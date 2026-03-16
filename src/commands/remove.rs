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

    if manifest.dependencies.remove(package).is_none() {
        bail!("Package '{}' not found in dependencies", package);
    }

    if let Some(locked) = lockfile.dependencies.remove(package) {
        for file in &locked.files {
            vendor::remove_file(Path::new(&config.output_dir), &file.filename)?;
        }
    }

    manifest.save()?;
    lockfile.save()?;

    vendor::clean_if_canonical(&config, &lockfile, Path::new(&config.output_dir))?;

    println!("Removed {package}");
    Ok(())
}
