use anyhow::{Result, bail};
use serde::Deserialize;
use std::fmt;

/// Identifies where a package is hosted on jsdelivr.
#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Npm(String),
    GitHub { user: String, repo: String },
}

impl PackageSource {
    /// Parse a package specifier: "gh:user/repo" for GitHub, anything else for npm.
    pub fn parse(input: &str) -> Result<Self> {
        // r[impl manifest.source.github-prefix]
        if let Some(gh) = input.strip_prefix("gh:") {
            let (user, repo) = gh.split_once('/').ok_or_else(|| {
                anyhow::anyhow!("GitHub source must be 'gh:user/repo', got '{input}'")
            })?;
            // r[impl add.input.github-validation]
            if user.is_empty() || repo.is_empty() {
                bail!("GitHub source must be 'gh:user/repo', got '{input}'");
            }
            Ok(Self::GitHub {
                user: user.to_string(),
                repo: repo.to_string(),
            })
        } else {
            // r[impl manifest.source.default]
            // r[impl add.input.source]
            Ok(Self::Npm(input.to_string()))
        }
    }

    /// The API path segment: "npm/htmx.org" or "gh/user/repo"
    fn api_path(&self) -> String {
        match self {
            Self::Npm(name) => format!("npm/{name}"),
            Self::GitHub { user, repo } => format!("gh/{user}/{repo}"),
        }
    }

    /// The CDN path segment: "npm/htmx.org@2.0.4" or "gh/user/repo@1.0.0"
    fn cdn_path(&self, version: &str) -> String {
        match self {
            Self::Npm(name) => format!("npm/{name}@{version}"),
            Self::GitHub { user, repo } => format!("gh/{user}/{repo}@{version}"),
        }
    }

    /// A display name suitable for use as manifest key.
    pub fn display_name(&self) -> String {
        match self {
            Self::Npm(name) => name.clone(),
            Self::GitHub { user, repo } => format!("gh:{user}/{repo}"),
        }
    }

    /// The serialized source value for the manifest (None for npm since it's the default).
    pub fn manifest_source(&self) -> Option<String> {
        match self {
            Self::Npm(_) => None,
            Self::GitHub { user, repo } => Some(format!("gh:{user}/{repo}")),
        }
    }

    /// Reconstruct a PackageSource from the manifest key name.
    /// Keys starting with "gh:" are GitHub packages, everything else is npm.
    pub fn from_manifest(name: &str, source: Option<&str>) -> Result<Self> {
        // Explicit source field takes precedence (backwards compat)
        // r[impl manifest.source.field]
        if let Some(s) = source {
            return Self::parse(s);
        }
        // Otherwise infer from key name
        Self::parse(name)
    }
}

impl fmt::Display for PackageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// The result of resolving a GitHub ref (branch, SHA, or tag).
pub struct ResolvedVersion {
    /// What the manifest should store (user's original input).
    pub manifest_version: String,
    /// What the lockfile should store (resolved SHA or tag).
    pub lockfile_version: String,
}

pub struct Registry {
    client: reqwest::Client,
}

#[derive(Debug)]
pub struct PackageInfo {
    pub name: String,
    pub versions: Vec<VersionInfo>,
    pub tags: Tags,
}

#[derive(Debug)]
pub struct Tags {
    pub latest: Option<String>,
}

#[derive(Debug)]
pub struct VersionInfo {
    pub version: String,
}

#[derive(Debug)]
pub struct PackageFiles {
    pub default: Option<String>,
    pub files: Vec<FileEntry>,
}

#[derive(Debug)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Deserialize)]
struct ApiPackageInfo {
    name: String,
    versions: Vec<ApiVersionInfo>,
    tags: ApiTags,
}

#[derive(Deserialize)]
struct ApiTags {
    latest: Option<String>,
}

#[derive(Deserialize)]
struct ApiVersionInfo {
    version: String,
}

#[derive(Deserialize)]
struct ApiPackageFiles {
    default: Option<String>,
    files: Vec<ApiFileNode>,
}

#[derive(Deserialize)]
struct ApiFileNode {
    name: String,
    #[serde(rename = "type")]
    node_type: String,
    hash: Option<String>,
    size: Option<u64>,
    files: Option<Vec<ApiFileNode>>,
}

fn flatten_files(nodes: &[ApiFileNode], prefix: &str, out: &mut Vec<FileEntry>) {
    for node in nodes {
        let path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{}/{}", prefix, node.name)
        };

        if node.node_type == "file" {
            out.push(FileEntry {
                path,
                hash: node.hash.clone().unwrap_or_default(),
                size: node.size.unwrap_or(0),
            });
        } else if let Some(children) = &node.files {
            flatten_files(children, &path, out);
        }
    }
}

/// Find the highest stable (non-prerelease) semver version.
pub fn latest_stable(versions: &[VersionInfo]) -> Option<String> {
    let mut stable: Vec<(String, semver::Version)> = versions
        .iter()
        .filter_map(|v| {
            let sv = semver::Version::parse(&v.version).ok()?;
            if sv.pre.is_empty() {
                Some((v.version.clone(), sv))
            } else {
                None
            }
        })
        .collect();

    stable.sort_by(|a, b| b.1.cmp(&a.1));
    stable.into_iter().next().map(|(s, _)| s)
}

const API_BASE: &str = "https://data.jsdelivr.com/v1/packages";
const CDN_BASE: &str = "https://cdn.jsdelivr.net";
const GITHUB_API_BASE: &str = "https://api.github.com";

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn get_package(&self, source: &PackageSource) -> Result<PackageInfo> {
        let url = format!("{API_BASE}/{}", source.api_path());
        log::debug!("GET {url}");
        let resp = self.client.get(&url).send().await?;
        log::debug!("  -> {}", resp.status());

        // r[impl add.resolve.not-found]
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            bail!("Package not found: {source}");
        }

        let api: ApiPackageInfo = resp.error_for_status()?.json().await?;

        Ok(PackageInfo {
            name: api.name,
            versions: api
                .versions
                .into_iter()
                .map(|v| VersionInfo { version: v.version })
                .collect(),
            tags: Tags {
                latest: api.tags.latest,
            },
        })
    }

    pub async fn get_package_files(
        &self,
        source: &PackageSource,
        version: &str,
    ) -> Result<PackageFiles> {
        let url = format!("{API_BASE}/{}@{version}", source.api_path());
        log::debug!("GET {url}");
        let resp = self.client.get(&url).send().await?;
        log::debug!("  -> {}", resp.status());

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            bail!("Package not found: {source}@{version}");
        }

        let api: ApiPackageFiles = resp.error_for_status()?.json().await?;

        let mut files = Vec::new();
        flatten_files(&api.files, "", &mut files);

        log::debug!("  default: {:?}", api.default);
        log::debug!("  {} files returned", files.len());
        for f in &files {
            log::debug!("    {} (hash: {})", f.path, f.hash);
        }

        Ok(PackageFiles {
            default: api.default,
            files,
        })
    }

    pub fn file_url(source: &PackageSource, version: &str, file_path: &str) -> String {
        format!("{CDN_BASE}/{}/{file_path}", source.cdn_path(version))
    }

    // r[impl add.version.github-ref]
    // r[impl add.version.github-resolve]
    /// Resolve a GitHub version (tag, branch, or SHA) to a commit SHA.
    pub async fn resolve_github_ref(
        &self,
        source: &PackageSource,
        version: &str,
    ) -> Result<ResolvedVersion> {
        let PackageSource::GitHub { user, repo } = source else {
            bail!("resolve_github_ref called on non-GitHub source");
        };

        let gh_url = format!("{GITHUB_API_BASE}/repos/{user}/{repo}/commits/{version}");
        log::debug!("GET {gh_url}");
        let resp = self
            .client
            .get(&gh_url)
            .header("Accept", "application/vnd.github.sha")
            .header("User-Agent", "unpm")
            .send()
            .await?;
        log::debug!("  -> {}", resp.status());

        if resp.status() == reqwest::StatusCode::FORBIDDEN {
            bail!(
                "GitHub API rate limit exceeded (60 requests/hour for unauthenticated requests). \
                 Please wait and try again."
            );
        }

        if resp.status() == reqwest::StatusCode::NOT_FOUND
            || resp.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY
        {
            bail!("Version '{version}' not found for {source}");
        }

        let sha = resp.error_for_status()?.text().await?.trim().to_string();

        Ok(ResolvedVersion {
            manifest_version: version.to_string(),
            lockfile_version: sha,
        })
    }
}
