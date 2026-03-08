use anyhow::{bail, Result};
use std::path::Path;

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::{PackageSource, Registry};
use crate::vendor;

pub async fn update(package: &str, version: Option<&str>) -> Result<()> {
    // Parse package@version syntax
    let (package, version) = match version {
        Some(_) => (package, version),
        None => match package.rsplit_once('@') {
            Some((pkg, ver)) if !pkg.is_empty() && !ver.is_empty() => (pkg, Some(ver)),
            _ => (package, None),
        },
    };

    let mut manifest = Manifest::load()?;
    let mut lockfile = Lockfile::load()?;
    let config = Config::load()?;
    let output_dir = Path::new(&config.output_dir);

    let dep = manifest
        .dependencies
        .get(package)
        .ok_or_else(|| anyhow::anyhow!("Package '{package}' not found in dependencies"))?;

    let locked = lockfile
        .dependencies
        .get(package)
        .ok_or_else(|| anyhow::anyhow!("Package '{package}' not found in lockfile"))?;

    let source = PackageSource::from_manifest(package, dep.source())?;
    let client = reqwest::Client::new();
    let registry = Registry::with_client(client.clone());
    let fetcher = Fetcher::with_client(client);

    let old_version = dep.version().to_string();

    // Resolve target version
    let new_version = match version {
        Some(v) => v.to_string(),
        None => {
            let pkg_info = registry.get_package(&source).await?;
            let latest = pkg_info
                .tags
                .latest
                .or_else(|| latest_stable(&pkg_info.versions))
                .ok_or_else(|| anyhow::anyhow!("No versions found for {source}"))?;
            latest
        }
    };

    if new_version == old_version {
        println!("{package} is already at {old_version}");
        return Ok(());
    }

    // Extract the file path from the old URL
    // URL format: https://cdn.jsdelivr.net/{npm|gh}/package@version/path/to/file
    let file_path = extract_file_path(&locked.url, &old_version)?;

    // Fetch the new file
    let url = Registry::file_url(&source, &new_version, &file_path);
    let result = fetcher.fetch(&url).await?;

    // Update manifest
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

    manifest
        .dependencies
        .insert(package.to_string(), new_dep);
    manifest.save()?;

    // Update lockfile
    lockfile.dependencies.insert(
        package.to_string(),
        LockedDependency {
            version: new_version.clone(),
            url: url.clone(),
            sha256: result.sha256,
            size: result.size,
            filename: filename.clone(),
        },
    );
    lockfile.save()?;

    // Replace vendored file
    vendor::place_file(output_dir, &filename, &result.bytes)?;

    println!("{package}: {old_version} -> {new_version}");

    Ok(())
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
/// e.g. "https://cdn.jsdelivr.net/npm/htmx.org@2.0.7/dist/htmx.min.js" -> "dist/htmx.min.js"
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
