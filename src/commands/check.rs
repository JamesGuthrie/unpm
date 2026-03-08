use crate::config::Config;
use crate::cve::CveChecker;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::registry::{PackageSource, Registry};
use futures::stream::{self, StreamExt};
use std::path::Path;

enum CheckTask {
    Cve {
        name: String,
        cve_name: String,
        version: String,
        ignore_cves: Vec<String>,
    },
    CdnHash {
        name: String,
        source: PackageSource,
        version: String,
        local_sha256: String,
        filename: String,
    },
}

enum CheckResult {
    Cve {
        name: String,
        result: anyhow::Result<Vec<crate::cve::Vulnerability>>,
        ignore_cves: Vec<String>,
    },
    CdnHash {
        name: String,
        result: anyhow::Result<Option<String>>,
        local_sha256: String,
    },
}

pub async fn check(allow_vulnerable: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);
    let client = reqwest::Client::new();
    let cve_checker = CveChecker::with_client(client.clone());
    let registry = Registry::with_client(client);

    if manifest.dependencies.is_empty() {
        println!("No dependencies to check.");
        return Ok(());
    }

    let mut errors: Vec<String> = Vec::new();
    let mut tasks: Vec<CheckTask> = Vec::new();

    // SHA verification against lockfile (local, synchronous)
    for (name, dep) in &manifest.dependencies {
        let locked = match lockfile.dependencies.get(name) {
            Some(l) => l,
            None => {
                errors.push(format!("{name}: not in lockfile, run `unpm add` first"));
                continue;
            }
        };

        let file_path = output_dir.join(&locked.filename);

        let local_sha256 = match std::fs::read(&file_path) {
            Ok(bytes) => {
                let hash = Fetcher::hash(&bytes);
                if hash != locked.sha256 {
                    errors.push(format!("{name}: SHA mismatch for {}", locked.filename));
                }
                hash
            }
            Err(_) => {
                errors.push(format!("{name}: vendored file not found ({})", file_path.display()));
                continue;
            }
        };

        // Queue CDN hash verification
        if let Ok(source) = PackageSource::from_manifest(name, dep.source()) {
            tasks.push(CheckTask::CdnHash {
                name: name.clone(),
                source,
                version: dep.version().to_string(),
                local_sha256,
                filename: locked.filename.clone(),
            });
        }

        // Queue CVE check
        let source = PackageSource::from_manifest(name, dep.source()).ok();
        let cve_name = match &source {
            Some(PackageSource::Npm(n)) => n.clone(),
            _ => name.clone(),
        };
        tasks.push(CheckTask::Cve {
            name: name.clone(),
            cve_name,
            version: dep.version().to_string(),
            ignore_cves: dep.ignore_cves().to_vec(),
        });
    }

    // Run CVE + CDN hash checks in parallel (up to 5 concurrent)
    let results: Vec<_> = stream::iter(tasks)
        .map(|task| {
            let cve_checker = &cve_checker;
            let registry = &registry;
            async move {
                match task {
                    CheckTask::Cve { name, cve_name, version, ignore_cves } => {
                        let result = cve_checker.check(&cve_name, &version).await;
                        CheckResult::Cve { name, result, ignore_cves }
                    }
                    CheckTask::CdnHash { name, source, version, local_sha256, filename } => {
                        let result = async {
                            let pkg_files = registry.get_package_files(&source, &version).await?;
                            let entry = pkg_files.files.iter().find(|f| {
                                Path::new(&f.path)
                                    .file_name()
                                    .is_some_and(|n| n == filename.as_str())
                            });
                            Ok(entry.map(|e| e.hash.clone()))
                        }.await;
                        CheckResult::CdnHash { name, result, local_sha256 }
                    }
                }
            }
        })
        .buffer_unordered(5)
        .collect()
        .await;

    for result in results {
        match result {
            CheckResult::Cve { name, result, ignore_cves } => {
                match result {
                    Ok(vulns) => {
                        let unignored: Vec<_> = vulns
                            .iter()
                            .filter(|v| !ignore_cves.contains(&v.id))
                            .collect();

                        if !unignored.is_empty() && !allow_vulnerable {
                            for vuln in &unignored {
                                errors.push(format!("{name}: {} \u{2014} {}", vuln.id, vuln.summary));
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(format!("{name}: could not check CVEs: {e}"));
                    }
                }
            }
            CheckResult::CdnHash { name, result, local_sha256 } => {
                match result {
                    Ok(Some(cdn_hash)) => {
                        // jsdelivr returns base64-encoded SHA-256
                        let cdn_hex = base64_to_hex(&cdn_hash);
                        if cdn_hex != local_sha256 {
                            errors.push(format!("{name}: vendored file does not match CDN hash"));
                        }
                    }
                    Ok(None) => {
                        errors.push(format!("{name}: file not found on CDN for verification"));
                    }
                    Err(e) => {
                        errors.push(format!("{name}: could not verify against CDN: {e}"));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        println!("All checks passed.");
    } else {
        for err in &errors {
            println!("{err}");
        }
        anyhow::bail!("Check failed.");
    }

    Ok(())
}

fn base64_to_hex(b64: &str) -> String {
    use base64::Engine;
    match base64::engine::general_purpose::STANDARD.decode(b64) {
        Ok(bytes) => hex::encode(bytes),
        Err(_) => String::new(),
    }
}
