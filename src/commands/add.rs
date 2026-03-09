use std::io::IsTerminal;
use std::path::Path;

use anyhow::{bail, Result};
use dialoguer::{Confirm, Select};

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::{latest_stable, PackageSource, Registry};
use crate::vendor;

pub async fn add(package: &str, version: Option<&str>, file: Option<&str>) -> Result<()> {
    // Parse package@version syntax (last @ wins, to handle gh:user/repo@version)
    let (package, version) = match version {
        Some(_) => (package, version),
        None => match package.rsplit_once('@') {
            Some((pkg, ver)) if !pkg.is_empty() && !ver.is_empty() => (pkg, Some(ver)),
            _ => (package, None),
        },
    };

    let interactive = std::io::stdin().is_terminal();

    if !interactive && (version.is_none() || file.is_none()) {
        bail!("Non-interactive mode requires both --version and --file flags");
    }

    let source = PackageSource::parse(package)?;
    let client = reqwest::Client::new();
    let registry = Registry::with_client(client.clone());
    let fetcher = Fetcher::with_client(client);

    // Step 1: Look up package
    println!("Looking up {source}...");
    let pkg_info = registry.get_package(&source).await?;

    // Step 2: Select version
    let selected_version = select_version(&pkg_info, version, interactive)?;

    // Step 3: Get file listing for selected version
    println!("Fetching file list for {source}@{selected_version}...");
    let pkg_files = registry
        .get_package_files(&source, &selected_version)
        .await?;

    // Step 4: Select file
    let selected_file = select_file(&pkg_files, file, interactive)?;

    // Step 5: Minification preference
    let final_file = handle_minification(&pkg_files, &selected_file, interactive)?;

    // Step 6: Fetch the file
    let url = Registry::file_url(&source, &selected_version, &final_file);
    println!("Fetching {url}...");
    let result = fetcher.fetch(&url).await?;

    // Step 7: Build vendored filename (namespaced to avoid collisions)
    let original_filename = Path::new(&final_file)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let manifest_key = source.display_name();

    // Use plain filename unless it would collide with an existing vendored file
    let lockfile = Lockfile::load()?;
    let has_collision = lockfile.dependencies.values().any(|l| l.filename == original_filename);
    let vendored_filename = if has_collision {
        format!(
            "{}_{}",
            manifest_key.replace(['/', ':'], "-"),
            original_filename
        )
    } else {
        original_filename.clone()
    };

    // Step 8: Confirm
    if interactive && version.is_none() {
        println!();
        println!("  Package:  {source}");
        println!("  Version:  {selected_version}");
        println!("  File:     {final_file}");
        println!("  Size:     {} bytes", result.size);
        println!("  SHA-256:  {}", result.sha256);
        println!("  Saved as: {vendored_filename}");
        println!();

        let confirm = Confirm::new()
            .with_prompt("Add this dependency?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Step 9: Determine if selected file is the default entry point
    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d));
    let is_default = default_path == Some(final_file.as_str());

    // Step 10: Write manifest
    // Source is inferred from key name (gh: prefix), no explicit source field needed
    let mut manifest = Manifest::load()?;

    let dep = if is_default {
        Dependency::Short(selected_version.clone())
    } else {
        Dependency::Extended {
            version: selected_version.clone(),
            source: None,
            file: Some(final_file.clone()),
            url: None,
            ignore_cves: Vec::new(),
        }
    };
    manifest.dependencies.insert(manifest_key.clone(), dep);
    manifest.save()?;

    // Step 11: Write lockfile
    let mut lockfile = Lockfile::load()?;
    lockfile.dependencies.insert(
        manifest_key.clone(),
        LockedDependency {
            version: selected_version.clone(),
            url: url.clone(),
            sha256: result.sha256.clone(),
            size: result.size,
            filename: vendored_filename.clone(),
        },
    );
    lockfile.save()?;

    // Step 12: Place file
    let config = Config::load()?;
    let output_dir = Path::new(&config.output_dir);
    vendor::place_file(output_dir, &vendored_filename, &result.bytes)?;

    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

    println!("Added {source}@{selected_version} -> {}/{vendored_filename}", config.output_dir);

    Ok(())
}

/// Sort version strings by semver (highest first). Non-semver strings sort to the end.
fn sorted_versions(versions: &[crate::registry::VersionInfo]) -> Vec<&str> {
    let mut parsed: Vec<(&str, Option<semver::Version>)> = versions
        .iter()
        .map(|v| (v.version.as_str(), semver::Version::parse(&v.version).ok()))
        .collect();

    parsed.sort_by(|a, b| match (&b.1, &a.1) {
        (Some(bv), Some(av)) => bv.cmp(av),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.0.cmp(b.0),
    });

    parsed.into_iter().map(|(s, _)| s).collect()
}


fn select_version(
    pkg_info: &crate::registry::PackageInfo,
    version_flag: Option<&str>,
    interactive: bool,
) -> Result<String> {
    if let Some(v) = version_flag {
        if !pkg_info.versions.iter().any(|vi| vi.version == v) {
            bail!("Version {v} not found for {}", pkg_info.name);
        }
        return Ok(v.to_string());
    }

    // Prefer: latest tag > highest stable semver > first in sorted list
    let stable = latest_stable(&pkg_info.versions);
    let default_version = pkg_info
        .tags
        .latest
        .as_deref()
        .or(stable.as_deref())
        .or_else(|| sorted_versions(&pkg_info.versions).first().copied())
        .ok_or_else(|| anyhow::anyhow!("No versions found for {}", pkg_info.name))?;

    if !interactive {
        return Ok(default_version.to_string());
    }

    let label = if pkg_info.tags.latest.is_some() {
        format!("Use latest version ({default_version})?")
    } else {
        format!("Use latest stable version ({default_version})?")
    };

    let use_default = Confirm::new()
        .with_prompt(label)
        .default(true)
        .interact()?;

    if use_default {
        return Ok(default_version.to_string());
    }

    let versions = sorted_versions(&pkg_info.versions);

    let selection = Select::new()
        .with_prompt("Select version")
        .items(&versions)
        .default(0)
        .interact()?;

    Ok(versions[selection].to_string())
}

fn select_file(
    pkg_files: &crate::registry::PackageFiles,
    file_flag: Option<&str>,
    interactive: bool,
) -> Result<String> {
    if let Some(f) = file_flag {
        let path = f.strip_prefix('/').unwrap_or(f);
        if !pkg_files.files.iter().any(|fe| fe.path == path) {
            bail!("File {f} not found in package");
        }
        return Ok(path.to_string());
    }

    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d).to_string());

    if !interactive {
        return default_path.ok_or_else(|| anyhow::anyhow!("No default entry point; use --file"));
    }

    if let Some(ref default) = default_path {
        let items = &[
            format!("Use default entry point ({default})"),
            "Select file manually".to_string(),
        ];
        let selection = Select::new()
            .with_prompt("File selection")
            .items(&items[..])
            .default(0)
            .interact()?;

        if selection == 0 {
            return Ok(default.clone());
        }
    }

    let file_labels: Vec<String> = pkg_files
        .files
        .iter()
        .map(|f| format!("{} ({} bytes)", f.path, f.size))
        .collect();

    let selection = Select::new()
        .with_prompt("Select file")
        .items(&file_labels)
        .default(0)
        .interact()?;

    Ok(pkg_files.files[selection].path.clone())
}

fn handle_minification(
    pkg_files: &crate::registry::PackageFiles,
    selected_file: &str,
    interactive: bool,
) -> Result<String> {
    if !interactive {
        return Ok(selected_file.to_string());
    }

    let file_paths: Vec<&str> = pkg_files.files.iter().map(|f| f.path.as_str()).collect();

    let counterpart = find_min_counterpart(selected_file, &file_paths);

    if let Some((min_file, full_file)) = counterpart {
        let items = &[
            format!("{min_file} (minified)"),
            format!("{full_file} (unminified)"),
        ];
        let default_idx = if selected_file == min_file { 0 } else { 1 };

        let selection = Select::new()
            .with_prompt("Both minified and unminified versions exist")
            .items(items)
            .default(default_idx)
            .interact()?;

        return Ok(if selection == 0 {
            min_file.to_string()
        } else {
            full_file.to_string()
        });
    }

    Ok(selected_file.to_string())
}

fn find_min_counterpart(
    selected: &str,
    all_files: &[&str],
) -> Option<(String, String)> {
    for ext in &[".js", ".css"] {
        let min_ext = format!(".min{ext}");

        if selected.ends_with(&min_ext) {
            let unminified = format!(
                "{}{ext}",
                &selected[..selected.len() - min_ext.len()]
            );
            if all_files.contains(&unminified.as_str()) {
                return Some((selected.to_string(), unminified));
            }
        } else if let Some(stripped) = selected.strip_suffix(ext) {
            let minified = format!(
                "{}{min_ext}",
                stripped
            );
            if all_files.contains(&minified.as_str()) {
                return Some((minified, selected.to_string()));
            }
        }
    }
    None
}
