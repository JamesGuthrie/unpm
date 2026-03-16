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

    // r[impl install.preconditions.empty-manifest]
    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    // r[impl install.fetch.progress-total]
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

    for name in manifest.dependencies.keys() {
        // r[impl install.preconditions.missing-lock-entry]
        let locked = lockfile.dependencies.get(name).ok_or_else(|| {
            anyhow::anyhow!("'{name}' is in unpm.toml but not in unpm.lock. Run `unpm add` first.")
        })?;

        for locked_file in &locked.files {
            let url = &locked_file.url;

            let result = fetcher.fetch(url).await?;

            // r[impl install.integrity.sha256]
            if !Fetcher::verify(&result.bytes, &locked_file.sha256) {
                // r[impl install.integrity.mismatch]
                anyhow::bail!(
                    "SHA mismatch for {name} ({})!\nExpected: {}\nGot:      {}",
                    locked_file.filename,
                    locked_file.sha256,
                    result.sha256
                );
            }

            // r[impl install.vendor.placement]
            vendor::place_file(output_dir, &locked_file.filename, &result.bytes)?;
            pb.inc(1);
        }
    }

    pb.finish_with_message("Done");

    // r[impl install.vendor.canonical-cleanup]
    vendor::clean_if_canonical(&config, &lockfile, output_dir)?;

    // r[impl install.vendor.success-message]
    println!(
        "Installed {} dependencies to {}",
        manifest.dependencies.len(),
        config.output_dir
    );
    Ok(())
}
