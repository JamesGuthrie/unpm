use crate::config::Config;
use crate::cve::CveChecker;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::registry::PackageSource;
use std::path::Path;

pub async fn check(allow_vulnerable: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);
    let client = reqwest::Client::new();
    let cve_checker = CveChecker::with_client(client);

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
        let file_path = output_dir.join(&locked.filename);

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

        // CVE checking — use the npm package name for OSV queries
        // (GitHub packages won't have npm CVEs, but we check anyway)
        let source = PackageSource::from_manifest(name, dep.source()).ok();
        let cve_name = match &source {
            Some(PackageSource::Npm(n)) => n.as_str(),
            _ => name,
        };

        let ignore_cves = dep.ignore_cves();
        match cve_checker.check(cve_name, dep.version()).await {
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

        println!();
    }

    if has_errors {
        anyhow::bail!("Check failed.");
    }

    println!("All checks passed.");
    Ok(())
}
