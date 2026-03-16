use anyhow::{Result, bail};
use std::path::Path;

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, LockedFile, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::{PackageSource, Registry, latest_stable};
use crate::vendor;

pub async fn update(package: Option<&str>, version: Option<&str>, latest: bool) -> Result<()> {
    // r[impl update.target.at-syntax]
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
            // r[impl update.target.version-requires-package]
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
            // r[impl update.precondition.in-manifest]
            if !manifest.dependencies.contains_key(pkg) {
                bail!("Package '{pkg}' not found in dependencies");
            }
            vec![pkg.to_string()]
        }
        // r[impl update.target.all]
        None => manifest.dependencies.keys().cloned().collect(),
    };

    for name in &packages {
        let dep = &manifest.dependencies[name];
        let locked = lockfile
            .dependencies
            .get(name.as_str())
            // r[impl update.precondition.in-lockfile]
            .ok_or_else(|| anyhow::anyhow!("'{name}' not found in lockfile"))?
            .clone();

        let source = PackageSource::from_manifest(name, dep.source())?;
        let old_version = dep.version().to_string();

        let new_version = match version {
            // r[impl update.version.explicit]
            Some(v) => v.to_string(),
            None => {
                let pkg_info = registry.get_package(&source).await?;
                let current_major = semver::Version::parse(&old_version).ok().map(|v| v.major);

                match current_major {
                    Some(major) if !latest => {
                        // r[impl update.version.major-boundary]
                        match latest_compatible(&pkg_info.versions, major) {
                            Some(v) => {
                                // If already at latest compatible, check if a newer major exists
                                if v == old_version {
                                    if let Some(abs_latest) = latest_stable(&pkg_info.versions)
                                        && abs_latest != old_version
                                    {
                                        // r[impl update.version.held-back]
                                        println!(
                                            "{name}: {old_version} held back \
                                             ({abs_latest} available, use --latest to update across major versions)"
                                        );
                                    }
                                    continue;
                                }
                                v
                            }
                            None => continue,
                        }
                    }
                    // r[impl update.version.latest-flag]
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
            // r[impl update.version.already-current-single]
            // r[impl update.version.already-current-all]
            if package.is_some() {
                println!("{name} is already at {old_version}");
            }
            continue;
        }

        struct FetchedFile {
            locked: LockedFile,
            bytes: Vec<u8>,
        }

        // r[impl update.files.atomic]
        let mut fetched: Vec<FetchedFile> = Vec::new();
        // r[impl update.files.path-extraction]
        for locked_file in &locked.files {
            let file_path = crate::url::extract_file_path(&locked_file.url, &old_version)?;
            let url = Registry::file_url(&source, &new_version, &file_path);
            let result = match fetcher.fetch(&url).await {
                Ok(r) => r,
                // r[impl update.files.fetch-failure]
                Err(e) => bail!(
                    "{name}: failed to fetch '{file_path}' at version {new_version}: {e}\nAdjust the `files` list in unpm.toml and retry."
                ),
            };
            fetched.push(FetchedFile {
                locked: LockedFile {
                    url,
                    sha256: result.sha256,
                    size: result.size,
                    filename: locked_file.filename.clone(),
                },
                bytes: result.bytes.to_vec(),
            });
        }

        let new_dep = match dep {
            // r[impl update.manifest.short-form]
            Dependency::Short(_) => Dependency::Short(new_version.clone()),
            // r[impl update.manifest.extended-form]
            Dependency::Extended {
                source,
                file,
                files,
                ignore_cves,
                ..
            } => Dependency::Extended {
                version: new_version.clone(),
                source: source.clone(),
                file: file.clone(),
                files: files.clone(),
                ignore_cves: ignore_cves.clone(),
            },
        };

        manifest.dependencies.insert(name.clone(), new_dep);
        lockfile.dependencies.insert(
            name.clone(),
            LockedDependency {
                version: new_version.clone(),
                files: fetched.iter().map(|f| f.locked.clone()).collect(),
            },
        );

        for f in &fetched {
            // r[impl update.vendor.placement]
            vendor::place_file(output_dir, &f.locked.filename, &f.bytes)?;
        }
        // r[impl update.output.success]
        println!("{name}: {old_version} -> {new_version}");
    }

    // r[impl update.persist.manifest]
    manifest.save()?;
    // r[impl update.persist.lockfile]
    lockfile.save()?;

    // r[impl update.vendor.cleanup]
    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

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
