use std::path::Path;

use anyhow::bail;

use crate::config::Config;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::vendor;

pub fn remove(package: &str) -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    let mut lockfile = Lockfile::load()?;
    let config = Config::load()?;

    // r[impl remove.manifest.exists]
    if manifest.dependencies.remove(package).is_none() {
        bail!("Package '{}' not found in dependencies", package);
    }

    // r[impl remove.lockfile.missing]
    // r[impl remove.files.delete]
    if let Some(locked) = lockfile.dependencies.remove(package) {
        for file in &locked.files {
            vendor::remove_file(Path::new(&config.output_dir), &file.filename)?;
        }
    }

    // r[impl remove.state.manifest]
    manifest.save()?;
    // r[impl remove.state.lockfile]
    lockfile.save()?;

    // r[impl remove.cleanup.canonical]
    vendor::clean_if_canonical(&config, &lockfile, Path::new(&config.output_dir))?;

    // r[impl remove.output.confirmation]
    println!("Removed {package}");
    Ok(())
}
