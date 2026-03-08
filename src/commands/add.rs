use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use dialoguer::{Confirm, Select};

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::Registry;
use crate::vendor;

pub async fn add(package: &str, version: Option<&str>, file: Option<&str>) -> Result<()> {
    let interactive = atty::is(atty::Stream::Stdin);

    if !interactive && (version.is_none() || file.is_none()) {
        bail!("Non-interactive mode requires both --version and --file flags");
    }

    let registry = Registry::new();

    // Step 1: Look up package
    println!("Looking up {package}...");
    let pkg_info = registry.get_package(package).await?;

    // Step 2: Select version
    let selected_version = select_version(&pkg_info, version, interactive)?;

    // Step 3: Get file listing for selected version
    println!("Fetching file list for {package}@{selected_version}...");
    let pkg_files = registry
        .get_package_files(package, &selected_version)
        .await?;

    // Step 4: Select file
    let selected_file = select_file(&pkg_files, file, interactive)?;

    // Step 5: Minification preference
    let final_file = handle_minification(&pkg_files, &selected_file, interactive)?;

    // Step 6: Fetch the file
    let url = Registry::file_url(package, &selected_version, &final_file);
    println!("Fetching {url}...");
    let fetcher = Fetcher::new();
    let result = fetcher.fetch(&url).await?;

    // Step 7: Confirm
    let filename = Path::new(&final_file)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if interactive && version.is_none() {
        println!();
        println!("  Package:  {package}");
        println!("  Version:  {selected_version}");
        println!("  File:     {final_file}");
        println!("  Size:     {} bytes", result.size);
        println!("  SHA-256:  {}", result.sha256);
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

    // Step 8: Determine if selected file is the default entry point
    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d));
    let is_default = default_path == Some(final_file.as_str());

    // Step 9: Write manifest
    let mut manifest = Manifest::load().unwrap_or_else(|_| Manifest {
        dependencies: BTreeMap::new(),
    });

    let dep = if is_default {
        Dependency::Short(selected_version.clone())
    } else {
        Dependency::Extended {
            version: selected_version.clone(),
            file: Some(final_file.clone()),
            url: None,
            ignore_cves: Vec::new(),
        }
    };
    manifest.dependencies.insert(package.to_string(), dep);
    manifest.save()?;

    // Step 10: Write lockfile
    let mut lockfile = Lockfile::load()?;
    lockfile.dependencies.insert(
        package.to_string(),
        LockedDependency {
            version: selected_version.clone(),
            url: url.clone(),
            sha256: result.sha256.clone(),
            size: result.size,
        },
    );
    lockfile.save()?;

    // Step 11: Place file
    let config = Config::load()?;
    vendor::place_file(Path::new(&config.output_dir), &filename, &result.bytes)?;

    println!("Added {package}@{selected_version} -> {}/{filename}", config.output_dir);

    Ok(())
}

fn select_version(
    pkg_info: &crate::registry::PackageInfo,
    version_flag: Option<&str>,
    interactive: bool,
) -> Result<String> {
    if let Some(v) = version_flag {
        // Validate the version exists
        if !pkg_info.versions.iter().any(|vi| vi.version == v) {
            bail!("Version {v} not found for {}", pkg_info.name);
        }
        return Ok(v.to_string());
    }

    let latest = pkg_info
        .tags
        .latest
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No latest tag found for {}", pkg_info.name))?;

    if !interactive {
        return Ok(latest.to_string());
    }

    let use_latest = Confirm::new()
        .with_prompt(format!("Use latest version ({latest})?"))
        .default(true)
        .interact()?;

    if use_latest {
        return Ok(latest.to_string());
    }

    // Show version picker (most recent first, jsdelivr returns oldest-first)
    let versions: Vec<&str> = pkg_info
        .versions
        .iter()
        .rev()
        .map(|v| v.version.as_str())
        .collect();

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
        let items = &["Use default entry point", "Select file manually"];
        let selection = Select::new()
            .with_prompt(format!("File selection (default: {default})"))
            .items(items)
            .default(0)
            .interact()?;

        if selection == 0 {
            return Ok(default.clone());
        }
    }

    // Manual file picker
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

    // Check if the selected file has a minified/unminified counterpart
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

/// Given a file path, checks if both `.min.ext` and `.ext` versions exist.
/// Returns Some((minified_path, unminified_path)) if both exist.
fn find_min_counterpart<'a>(
    selected: &str,
    all_files: &[&'a str],
) -> Option<(String, String)> {
    for ext in &[".js", ".css"] {
        let min_ext = format!(".min{ext}");

        if selected.ends_with(&min_ext) {
            // Selected is minified — check if unminified exists
            let unminified = format!(
                "{}{ext}",
                &selected[..selected.len() - min_ext.len()]
            );
            if all_files.contains(&unminified.as_str()) {
                return Some((selected.to_string(), unminified));
            }
        } else if selected.ends_with(ext) {
            // Selected is unminified — check if minified exists
            let minified = format!(
                "{}{min_ext}",
                &selected[..selected.len() - ext.len()]
            );
            if all_files.contains(&minified.as_str()) {
                return Some((minified, selected.to_string()));
            }
        }
    }
    None
}
