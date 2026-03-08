use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::{PackageSource, Registry};
use crate::vendor;

pub async fn update(package: Option<&str>, version: Option<&str>, latest: bool) -> Result<()> {
    // Parse package@version syntax
    let (package, version) = match package {
        Some(pkg) => match version {
            Some(_) => (Some(pkg), version),
            None => match pkg.rsplit_once('@') {
                Some((p, v)) if !p.is_empty() && !v.is_empty() => (Some(p), Some(v)),
                _ => (Some(pkg), None),
            },
        },
        None => {
            if version.is_some() {
                bail!("--version requires a package name");
            }
            (None, None)
        }
    };

    let mut manifest = Manifest::load()?;
    let mut lockfile = Lockfile::load()?;
    let config = Config::load()?;
    let output_dir = Path::new(&config.output_dir);
    let client = reqwest::Client::new();
    let registry = Registry::with_client(client.clone());
    let fetcher = Fetcher::with_client(client);

    let packages: Vec<String> = match package {
        Some(pkg) => {
            if !manifest.dependencies.contains_key(pkg) {
                bail!("Package '{pkg}' not found in dependencies");
            }
            vec![pkg.to_string()]
        }
        None => manifest.dependencies.keys().cloned().collect(),
    };

    for name in &packages {
        let dep = &manifest.dependencies[name];
        let locked = lockfile
            .dependencies
            .get(name.as_str())
            .ok_or_else(|| anyhow::anyhow!("'{name}' not found in lockfile"))?;

        let source = PackageSource::from_manifest(name, dep.source())?;
        let old_version = dep.version().to_string();

        let new_version = match version {
            Some(v) => v.to_string(),
            None => {
                let pkg_info = registry.get_package(&source).await?;
                let current_major = semver::Version::parse(&old_version)
                    .ok()
                    .map(|v| v.major);

                match current_major {
                    Some(major) if !latest => {
                        match latest_compatible(&pkg_info.versions, major) {
                            Some(v) => {
                                // If already at latest compatible, check if a newer major exists
                                if v == old_version {
                                    if let Some(abs_latest) = latest_stable(&pkg_info.versions) {
                                        if abs_latest != old_version {
                                            println!(
                                                "{name}: {old_version} held back \
                                                 ({abs_latest} available, use --latest to update across major versions)"
                                            );
                                        }
                                    }
                                    continue;
                                }
                                v
                            }
                            None => continue,
                        }
                    }
                    _ => {
                        match pkg_info
                            .tags
                            .latest
                            .or_else(|| latest_stable(&pkg_info.versions))
                        {
                            Some(v) => v,
                            None => continue,
                        }
                    }
                }
            }
        };

        if new_version == old_version {
            if package.is_some() {
                println!("{name} is already at {old_version}");
            }
            continue;
        }

        let file_path = extract_file_path(&locked.url, &old_version)?;
        let url = Registry::file_url(&source, &new_version, &file_path);
        let result = fetcher.fetch(&url).await?;

        let new_dep = match dep {
            Dependency::Short(_) => Dependency::Short(new_version.clone()),
            Dependency::Extended {
                source,
                file,
                url: url_override,
                ignore_cves,
                ..
            } => Dependency::Extended {
                version: new_version.clone(),
                source: source.clone(),
                file: file.clone(),
                url: url_override.clone(),
                ignore_cves: ignore_cves.clone(),
            },
        };

        let filename = locked.filename.clone();

        manifest.dependencies.insert(name.clone(), new_dep);
        lockfile.dependencies.insert(
            name.clone(),
            LockedDependency {
                version: new_version.clone(),
                url,
                sha256: result.sha256,
                size: result.size,
                filename: filename.clone(),
            },
        );

        vendor::place_file(output_dir, &filename, &result.bytes)?;
        println!("{name}: {old_version} -> {new_version}");
    }

    manifest.save()?;
    lockfile.save()?;

    if config.canonical {
        let known: std::collections::HashSet<&str> = lockfile
            .dependencies
            .values()
            .map(|l| l.filename.as_str())
            .collect();
        vendor::clean(output_dir, &known)?;
    }

    Ok(())
}

/// Find the highest stable version with the same major version.
fn latest_compatible(versions: &[crate::registry::VersionInfo], major: u64) -> Option<String> {
    let mut compatible: Vec<(String, semver::Version)> = versions
        .iter()
        .filter_map(|v| {
            let sv = semver::Version::parse(&v.version).ok()?;
            if sv.pre.is_empty() && sv.major == major {
                Some((v.version.clone(), sv))
            } else {
                None
            }
        })
        .collect();

    compatible.sort_by(|a, b| b.1.cmp(&a.1));
    compatible.into_iter().next().map(|(s, _)| s)
}

/// Find the highest stable (non-prerelease) semver version.
fn latest_stable(versions: &[crate::registry::VersionInfo]) -> Option<String> {
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

/// Extract the file path portion from a jsdelivr CDN URL.
fn extract_file_path(url: &str, version: &str) -> Result<String> {
    let marker = format!("@{version}/");
    let idx = url
        .find(&marker)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse file path from lockfile URL: {url}"))?;
    let path = &url[idx + marker.len()..];
    if path.is_empty() {
        bail!("No file path found in lockfile URL: {url}");
    }
    Ok(path.to_string())
}
