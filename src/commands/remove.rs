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
        vendor::remove_file(Path::new(&config.output_dir), &locked.filename)?;
    }

    manifest.save()?;
    lockfile.save()?;

    if config.canonical {
        let output_dir = Path::new(&config.output_dir);
        let known: std::collections::HashSet<&str> = lockfile
            .dependencies
            .values()
            .map(|l| l.filename.as_str())
            .collect();
        vendor::clean(output_dir, &known)?;
    }

    println!("Removed {package}");
    Ok(())
}
