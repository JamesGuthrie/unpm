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
    let client = reqwest::Client::new();
    let fetcher = Fetcher::with_client(client);

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    let total_files: u64 = lockfile
        .dependencies
        .values()
        .map(|l| l.files.len() as u64)
        .sum();
    let pb = ProgressBar::new(total_files);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:30}] {pos}/{len}")
            .unwrap(),
    );
    pb.set_message("Installing");

    for (name, dep) in &manifest.dependencies {
        let locked = lockfile.dependencies.get(name).ok_or_else(|| {
            anyhow::anyhow!("'{name}' is in unpm.toml but not in unpm.lock. Run `unpm add` first.")
        })?;

        for locked_file in &locked.files {
            // Use custom URL from manifest if specified (single-file deps only)
            let url = if locked.files.len() == 1 {
                dep.url().unwrap_or(&locked_file.url)
            } else {
                &locked_file.url
            };

            let result = fetcher.fetch(url).await?;

            if !Fetcher::verify(&result.bytes, &locked_file.sha256) {
                anyhow::bail!(
                    "SHA mismatch for {name} ({})!\nExpected: {}\nGot:      {}",
                    locked_file.filename,
                    locked_file.sha256,
                    result.sha256
                );
            }

            vendor::place_file(output_dir, &locked_file.filename, &result.bytes)?;
            pb.inc(1);
        }
    }

    pb.finish_with_message("Done");

    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

    println!(
        "Installed {} dependencies to {}",
        manifest.dependencies.len(),
        config.output_dir
    );
    Ok(())
}
