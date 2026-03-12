use anyhow::{Context, bail};
use std::collections::HashSet;
use std::path::Path;

use crate::config::Config;
use crate::lockfile::Lockfile;

/// Resolve a filename within output_dir, rejecting path traversal attempts.
fn safe_path(output_dir: &Path, filename: &str) -> anyhow::Result<std::path::PathBuf> {
    let safe_name = Path::new(filename)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {filename}"))?;
    let dest = output_dir.join(safe_name);
    if dest.file_name() != Some(safe_name) {
        bail!("Path traversal rejected: {filename}");
    }
    Ok(dest)
}

pub fn place_file(output_dir: &Path, filename: &str, content: &[u8]) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create directory: {}", output_dir.display()))?;
    let dest = safe_path(output_dir, filename)?;
    std::fs::write(&dest, content)
        .with_context(|| format!("Failed to write: {}", dest.display()))?;
    Ok(())
}

pub fn remove_file(output_dir: &Path, filename: &str) -> anyhow::Result<()> {
    let dest = safe_path(output_dir, filename)?;
    if dest.exists() {
        std::fs::remove_file(&dest)
            .with_context(|| format!("Failed to remove: {}", dest.display()))?;
    }
    Ok(())
}

/// If canonical mode is enabled, remove untracked files from output_dir.
pub fn clean_if_canonical(
    config: &Config,
    lockfile: &Lockfile,
    output_dir: &Path,
) -> anyhow::Result<()> {
    if config.canonical {
        let known: HashSet<&str> = lockfile
            .dependencies
            .values()
            .map(|l| l.filename.as_str())
            .collect();
        clean(output_dir, &known)?;
    }
    Ok(())
}

/// Remove all files in output_dir that aren't in the known set.
pub fn clean(output_dir: &Path, known_filenames: &HashSet<&str>) -> anyhow::Result<()> {
    let entries = match std::fs::read_dir(output_dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && !known_filenames.contains(name)
        {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove: {}", path.display()))?;
            println!("Removed untracked file: {name}");
        }
    }

    Ok(())
}
