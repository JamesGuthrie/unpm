use anyhow::{bail, Result};
use serde::Deserialize;

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

// Raw API response types for deserialization

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

const BASE_URL: &str = "https://data.jsdelivr.com/v1/packages/npm";

impl Registry {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn get_package(&self, name: &str) -> Result<PackageInfo> {
        let url = format!("{BASE_URL}/{name}");
        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            bail!("Package not found: {name}");
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

    pub async fn get_package_files(&self, name: &str, version: &str) -> Result<PackageFiles> {
        let url = format!("{BASE_URL}/{name}@{version}");
        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            bail!("Package not found: {name}@{version}");
        }

        let api: ApiPackageFiles = resp.error_for_status()?.json().await?;

        let mut files = Vec::new();
        flatten_files(&api.files, "", &mut files);

        Ok(PackageFiles {
            default: api.default,
            files,
        })
    }

    pub fn file_url(name: &str, version: &str, file_path: &str) -> String {
        format!("https://cdn.jsdelivr.net/npm/{name}@{version}/{file_path}")
    }
}
