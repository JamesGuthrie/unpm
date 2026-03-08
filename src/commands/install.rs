use crate::config::Config;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::vendor;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub async fn install() -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);
    let fetcher = Fetcher::new();

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    let pb = ProgressBar::new(manifest.dependencies.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:30}] {pos}/{len}")
            .unwrap(),
    );
    pb.set_message("Installing");

    for (name, _dep) in &manifest.dependencies {
        let locked = lockfile
            .dependencies
            .get(name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "'{name}' is in unpm.toml but not in unpm.lock. Run `unpm add` first."
                )
            })?;

        let result = fetcher.fetch(&locked.url).await?;

        if !Fetcher::verify(&result.bytes, &locked.sha256) {
            anyhow::bail!(
                "SHA mismatch for {name}!\nExpected: {}\nGot:      {}\nThe remote file may have been tampered with.",
                locked.sha256,
                result.sha256
            );
        }

        let filename = locked.url.rsplit('/').next().unwrap_or(name);
        vendor::place_file(output_dir, filename, &result.bytes)?;
        pb.inc(1);
    }

    pb.finish_with_message("Done");
    println!(
        "Installed {} dependencies to {}",
        manifest.dependencies.len(),
        config.output_dir
    );
    Ok(())
}
