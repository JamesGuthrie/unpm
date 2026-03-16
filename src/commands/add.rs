use std::io::IsTerminal;
use std::path::Path;

use anyhow::{Result, bail};
use dialoguer::{Confirm, MultiSelect, Select};

use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::{LockedDependency, LockedFile, Lockfile};
use crate::manifest::{Dependency, Manifest};
use crate::registry::{PackageSource, Registry, latest_stable};
use crate::vendor;

pub async fn add(package: &str, version: Option<&str>, files_flag: &[String]) -> Result<()> {
    let (package, version) = match version {
        Some(_) => (package, version),
        None => match package.rsplit_once('@') {
            Some((pkg, ver)) if !pkg.is_empty() && !ver.is_empty() => (pkg, Some(ver)),
            _ => (package, None),
        },
    };

    let interactive = std::io::stdin().is_terminal();

    if !interactive && (version.is_none() || files_flag.is_empty()) {
        bail!("Non-interactive mode requires both --version and --file flags");
    }

    let source = PackageSource::parse(package)?;
    let manifest_key = source.display_name();
    let client = reqwest::Client::new();
    let registry = Registry::with_client(client.clone());
    let fetcher = Fetcher::with_client(client);

    // Check for existing dep (merge mode)
    let mut manifest = Manifest::load()?;
    let mut lockfile = Lockfile::load()?;
    let existing = manifest.dependencies.get(&manifest_key);

    if let Some(existing_dep) = existing
        && let Some(v) = version
        && v != existing_dep.version()
    {
        bail!(
            "{manifest_key} already exists at version {}. \
             Cannot add files at version {v}.",
            existing_dep.version()
        );
    }

    // Step 1: Look up package
    println!("Looking up {source}...");
    let pkg_info = registry.get_package(&source).await?;

    // Step 2: Select version
    let selected_version = if let Some(existing_dep) = existing {
        existing_dep.version().to_string()
    } else {
        select_version(&pkg_info, version, interactive)?
    };

    // Step 3: Get file listing
    println!("Fetching file list for {source}@{selected_version}...");
    let pkg_files = registry
        .get_package_files(&source, &selected_version)
        .await?;

    // Resolve existing files from lockfile
    let existing_file_paths: Vec<String> = lockfile
        .dependencies
        .get(&manifest_key)
        .map(|l| {
            l.files
                .iter()
                .filter_map(|f| crate::url::extract_file_path(&f.url, &selected_version).ok())
                .collect()
        })
        .unwrap_or_default();

    // Step 4: Select file(s)
    let selected_files = select_files(&pkg_files, files_flag, interactive, &existing_file_paths)?;

    // Filter out files that already exist
    let new_files: Vec<String> = selected_files
        .into_iter()
        .filter(|f| !existing_file_paths.contains(f))
        .collect();

    if new_files.is_empty() && existing.is_some() {
        println!("All specified files are already vendored for {manifest_key}.");
        return Ok(());
    }

    // Step 5: Fetch all new files
    let config = Config::load()?;
    let output_dir = Path::new(&config.output_dir);
    let mut fetched_files: Vec<(String, LockedFile, Vec<u8>)> = Vec::new();

    for file_path in &new_files {
        let url = Registry::file_url(&source, &selected_version, file_path);
        println!("Fetching {url}...");
        let result = fetcher.fetch(&url).await?;

        let original_filename = Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let vendored_filename = resolve_filename(
            &original_filename,
            file_path,
            &manifest_key,
            &lockfile,
            &fetched_files,
        );

        fetched_files.push((
            file_path.clone(),
            LockedFile {
                url,
                sha256: result.sha256.clone(),
                size: result.size,
                filename: vendored_filename,
            },
            result.bytes.to_vec(),
        ));
    }

    // Step 6: Confirm (if interactive and version wasn't pre-specified)
    if interactive && version.is_none() && existing.is_none() {
        println!();
        println!("  Package:  {source}");
        println!("  Version:  {selected_version}");
        for (path, locked_file, _) in &fetched_files {
            println!("  File:     {path} -> {}", locked_file.filename);
            println!("    Size:   {} bytes", locked_file.size);
            println!("    SHA:    {}", locked_file.sha256);
        }
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

    // Step 7: Determine manifest form
    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d));

    let all_file_paths: Vec<&str> = existing_file_paths
        .iter()
        .map(|s| s.as_str())
        .chain(new_files.iter().map(|s| s.as_str()))
        .collect();

    let existing_source = existing.and_then(|d| d.source().map(|s| s.to_string()));
    let existing_cves = existing
        .map(|d| d.ignore_cves().to_vec())
        .unwrap_or_default();

    let dep = if all_file_paths.len() == 1 && default_path == Some(all_file_paths[0]) {
        Dependency::Short(selected_version.clone())
    } else if all_file_paths.len() == 1 {
        Dependency::Extended {
            version: selected_version.clone(),
            source: existing_source.clone(),
            file: Some(all_file_paths[0].to_string()),
            files: None,
            url: None,
            ignore_cves: existing_cves.clone(),
        }
    } else {
        Dependency::Extended {
            version: selected_version.clone(),
            source: existing_source.clone(),
            file: None,
            files: Some(all_file_paths.iter().map(|s| s.to_string()).collect()),
            url: None,
            ignore_cves: existing_cves.clone(),
        }
    };

    manifest.dependencies.insert(manifest_key.clone(), dep);
    manifest.save()?;

    // Step 8: Update lockfile
    let mut all_locked_files: Vec<LockedFile> = lockfile
        .dependencies
        .get(&manifest_key)
        .map(|l| l.files.clone())
        .unwrap_or_default();
    for (_, locked_file, _) in &fetched_files {
        all_locked_files.push(locked_file.clone());
    }

    lockfile.dependencies.insert(
        manifest_key.clone(),
        LockedDependency {
            version: selected_version.clone(),
            files: all_locked_files,
        },
    );
    lockfile.save()?;

    // Step 9: Place files
    for (_, locked_file, bytes) in &fetched_files {
        vendor::place_file(output_dir, &locked_file.filename, bytes)?;
    }

    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

    if new_files.len() == 1 {
        println!(
            "Added {source}@{selected_version} -> {}/{}",
            config.output_dir, fetched_files[0].1.filename
        );
    } else {
        println!("Added {source}@{selected_version}:");
        for (_, locked_file, _) in &fetched_files {
            println!("  {}/{}", config.output_dir, locked_file.filename);
        }
    }

    Ok(())
}

/// Resolve vendored filename, avoiding collisions with existing lockfile entries,
/// other files being added in the same batch, and intra-package collisions.
fn resolve_filename(
    original: &str,
    file_path: &str,
    manifest_key: &str,
    lockfile: &Lockfile,
    batch: &[(String, LockedFile, Vec<u8>)],
) -> String {
    let existing_filenames: Vec<&str> = lockfile
        .dependencies
        .values()
        .flat_map(|l| l.files.iter().map(|f| f.filename.as_str()))
        .chain(batch.iter().map(|(_, f, _)| f.filename.as_str()))
        .collect();

    if !existing_filenames.contains(&original) {
        return original.to_string();
    }

    // Check if collision is intra-package (same batch has same basename)
    let batch_has_same_basename = batch.iter().any(|(_, f, _)| {
        Path::new(&f.filename)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            == Some(original.to_string())
    });

    if batch_has_same_basename {
        // Intra-package collision: prefix with parent directory segments
        let parts: Vec<&str> = file_path.split('/').collect();
        for depth in 1..parts.len() {
            let prefix = parts[parts.len() - 1 - depth..parts.len() - 1].join("_");
            let candidate = format!("{prefix}_{original}");
            if !existing_filenames.contains(&candidate.as_str()) {
                return candidate;
            }
        }
    }

    // Cross-package collision: namespace with package name
    format!("{}_{}", manifest_key.replace(['/', ':'], "-"), original)
}

fn select_files(
    pkg_files: &crate::registry::PackageFiles,
    files_flag: &[String],
    interactive: bool,
    existing_files: &[String],
) -> Result<Vec<String>> {
    if !files_flag.is_empty() {
        // Validate all specified files exist
        for f in files_flag {
            let path = f.strip_prefix('/').unwrap_or(f);
            if !pkg_files.files.iter().any(|fe| fe.path == path) {
                bail!("File {f} not found in package");
            }
        }
        return Ok(files_flag
            .iter()
            .map(|f| f.strip_prefix('/').unwrap_or(f).to_string())
            .collect());
    }

    let default_path = pkg_files
        .default
        .as_deref()
        .map(|d| d.strip_prefix('/').unwrap_or(d).to_string())
        .map(|d| resolve_default_path(&d, &pkg_files.files));

    if !interactive {
        return default_path
            .map(|d| vec![d])
            .ok_or_else(|| anyhow::anyhow!("No default entry point; use --file"));
    }

    // If there are existing files, skip the default prompt and go to multi-select
    if !existing_files.is_empty() {
        return interactive_multi_select(pkg_files, existing_files);
    }

    if let Some(ref default) = default_path {
        let items = &[
            format!("Use default entry point ({default})"),
            "Select file(s) manually".to_string(),
        ];
        let selection = Select::new()
            .with_prompt("File selection")
            .items(&items[..])
            .default(0)
            .interact()?;

        if selection == 0 {
            // Check for min counterpart
            let file_paths: Vec<&str> = pkg_files.files.iter().map(|f| f.path.as_str()).collect();
            let final_file =
                if let Some((min_file, full_file)) = find_min_counterpart(default, &file_paths) {
                    let items = &[
                        format!("{min_file} (minified)"),
                        format!("{full_file} (unminified)"),
                    ];
                    let default_idx = if *default == min_file { 0 } else { 1 };
                    let selection = Select::new()
                        .with_prompt("Both minified and unminified versions exist")
                        .items(items)
                        .default(default_idx)
                        .interact()?;
                    if selection == 0 { min_file } else { full_file }
                } else {
                    default.clone()
                };
            return Ok(vec![final_file]);
        }
    }

    interactive_multi_select(pkg_files, existing_files)
}

fn interactive_multi_select(
    pkg_files: &crate::registry::PackageFiles,
    existing_files: &[String],
) -> Result<Vec<String>> {
    let file_labels: Vec<String> = pkg_files
        .files
        .iter()
        .map(|f| {
            let marker = if existing_files.contains(&f.path) {
                " (already added)"
            } else {
                ""
            };
            format!("{} ({} bytes){marker}", f.path, f.size)
        })
        .collect();

    let defaults: Vec<bool> = pkg_files
        .files
        .iter()
        .map(|f| existing_files.contains(&f.path))
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("Select file(s) (space to toggle, enter to confirm)")
        .items(&file_labels)
        .defaults(&defaults)
        .interact()?;

    if selections.is_empty() {
        bail!("No files selected");
    }

    Ok(selections
        .into_iter()
        .map(|i| pkg_files.files[i].path.clone())
        .collect())
}

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

    let use_default = Confirm::new().with_prompt(label).default(true).interact()?;

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

fn resolve_default_path(default: &str, files: &[crate::registry::FileEntry]) -> String {
    if files.iter().any(|f| f.path == default) {
        return default.to_string();
    }

    for ext in &[".js", ".css"] {
        let min_ext = format!(".min{ext}");
        if let Some(stem) = default.strip_suffix(&min_ext) {
            let unminified = format!("{stem}{ext}");
            if files.iter().any(|f| f.path == unminified) {
                log::debug!(
                    "default path '{default}' not in file listing, using '{unminified}' instead"
                );
                return unminified;
            }
        }
    }

    default.to_string()
}

fn find_min_counterpart(selected: &str, all_files: &[&str]) -> Option<(String, String)> {
    for ext in &[".js", ".css"] {
        let min_ext = format!(".min{ext}");

        if selected.ends_with(&min_ext) {
            let unminified = format!("{}{ext}", &selected[..selected.len() - min_ext.len()]);
            if all_files.contains(&unminified.as_str()) {
                return Some((selected.to_string(), unminified));
            }
        } else if let Some(stripped) = selected.strip_suffix(ext) {
            let minified = format!("{}{min_ext}", stripped);
            if all_files.contains(&minified.as_str()) {
                return Some((minified, selected.to_string()));
            }
        }
    }
    None
}
