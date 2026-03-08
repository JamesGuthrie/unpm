use anyhow::Context;
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
