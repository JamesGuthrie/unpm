use anyhow::Context;
use std::collections::HashSet;
use std::path::Path;

pub fn place_file(output_dir: &Path, filename: &str, content: &[u8]) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create directory: {}", output_dir.display()))?;
    let dest = output_dir.join(filename);
    std::fs::write(&dest, content)
        .with_context(|| format!("Failed to write: {}", dest.display()))?;
    Ok(())
}

pub fn remove_file(output_dir: &Path, filename: &str) -> anyhow::Result<()> {
    let dest = output_dir.join(filename);
    if dest.exists() {
        std::fs::remove_file(&dest)
            .with_context(|| format!("Failed to remove: {}", dest.display()))?;
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
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if !known_filenames.contains(name) {
                std::fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove: {}", path.display()))?;
                println!("Removed untracked file: {name}");
            }
        }
    }

    Ok(())
}
