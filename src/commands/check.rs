use crate::config::Config;
use crate::cve::CveChecker;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::registry::Registry;
use std::path::Path;

pub async fn check(allow_vulnerable: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);
    let cve_checker = CveChecker::new();
    let registry = Registry::new();

    if manifest.dependencies.is_empty() {
        println!("No dependencies to check.");
        return Ok(());
    }

    let mut has_errors = false;

    for (name, dep) in &manifest.dependencies {
        println!("Checking {name}...");

        let locked = match lockfile.dependencies.get(name) {
            Some(l) => l,
            None => {
                println!("  \u{2717} Not found in lockfile. Run `unpm add` first.");
                has_errors = true;
                println!();
                continue;
            }
        };

        // SHA verification
        let filename = locked.url.rsplit('/').next().unwrap_or(name);
        let file_path = output_dir.join(filename);

        match std::fs::read(&file_path) {
            Ok(bytes) => {
                if Fetcher::verify(&bytes, &locked.sha256) {
                    println!("  \u{2713} SHA verified");
                } else {
                    println!("  \u{2717} SHA mismatch for vendored file!");
                    has_errors = true;
                }
            }
            Err(_) => {
                println!("  \u{2717} Vendored file not found: {}", file_path.display());
                has_errors = true;
            }
        }

        // CVE checking
        let ignore_cves = dep.ignore_cves();
        match cve_checker.check(name, dep.version()).await {
            Ok(vulns) => {
                let unignored: Vec<_> = vulns
                    .iter()
                    .filter(|v| !ignore_cves.contains(&v.id))
                    .collect();

                if unignored.is_empty() {
                    println!("  \u{2713} No known vulnerabilities");
                } else {
                    for vuln in &unignored {
                        println!("  \u{26a0} {} \u{2014} {}", vuln.id, vuln.summary);
                    }
                    if !allow_vulnerable {
                        has_errors = true;
                    }
                }
            }
            Err(e) => {
                println!("  \u{26a0} Could not check CVEs: {e}");
            }
        }

        // Freshness check
        match registry.get_package(name).await {
            Ok(info) => {
                if let Some(latest) = &info.tags.latest {
                    if latest != dep.version() {
                        println!("  \u{2139} Newer version available: {latest}");
                    }
                }
            }
            Err(_) => {}
        }

        println!();
    }

    if has_errors {
        println!("Check failed.");
        std::process::exit(1);
    }

    println!("All checks passed.");
    Ok(())
}
