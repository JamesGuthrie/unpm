use crate::config::Config;
use crate::cve::CveChecker;
use crate::fetch::Fetcher;
use crate::lockfile::Lockfile;
use crate::manifest::Manifest;
use crate::registry::{PackageSource, Registry, latest_stable};
use futures::stream::{self, StreamExt};
use std::path::Path;

enum CveQuery {
    Npm { package: String, version: String },
    GitCommit { sha: String },
}

enum CheckTask {
    Cve {
        name: String,
        query: CveQuery,
        ignore_cves: Vec<String>,
    },
    CdnHash {
        name: String,
        source: PackageSource,
        version: String,
        local_sha256: String,
        url: String,
    },
    Outdated {
        name: String,
        current: String,
        source: PackageSource,
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
    Outdated {
        name: String,
        current: String,
        latest: Option<String>,
    },
}

pub async fn check(allow_vulnerable: bool, fail_on_outdated: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;
    let output_dir = Path::new(&config.output_dir);
    let client = reqwest::Client::new();
    let cve_checker = CveChecker::with_client(client.clone());
    let registry = Registry::with_client(client);

    // r[impl check.empty]
    if manifest.dependencies.is_empty() {
        println!("No dependencies to check.");
        return Ok(());
    }

    let mut integrity_errors: Vec<String> = Vec::new();
    let mut tasks: Vec<CheckTask> = Vec::new();

    // SHA verification against lockfile (local, synchronous)
    for (name, dep) in &manifest.dependencies {
        log::debug!("checking {name} v{}", dep.version());

        let locked = match lockfile.dependencies.get(name) {
            Some(l) => l,
            // r[impl check.integrity.lockfile-presence]
            None => {
                integrity_errors.push(format!("  {name}: not in lockfile, run `unpm add` first"));
                continue;
            }
        };
        for locked_file in &locked.files {
            log::debug!("  lockfile filename: {}", locked_file.filename);

            let file_path = output_dir.join(&locked_file.filename);

            let local_sha256 = match std::fs::read(&file_path) {
                Ok(bytes) => {
                    let hash = Fetcher::hash(&bytes);
                    // r[impl check.integrity.sha-match]
                    if hash != locked_file.sha256 {
                        integrity_errors.push(format!(
                            "  {name}: SHA mismatch for {}",
                            locked_file.filename
                        ));
                    }
                    hash
                }
                // r[impl check.integrity.file-exists]
                Err(_) => {
                    integrity_errors.push(format!(
                        "  {name}: vendored file not found ({})",
                        file_path.display()
                    ));
                    continue;
                }
            };

            // r[impl check.integrity.cdn-verify]
            // Queue CDN hash verification per file
            if let Ok(source) = PackageSource::from_manifest(name, dep.source()) {
                tasks.push(CheckTask::CdnHash {
                    name: name.clone(),
                    source,
                    version: locked.version.clone(),
                    local_sha256,
                    url: locked_file.url.clone(),
                });
            }
        }

        // Queue CVE check per package
        // r[impl check.cve.query]
        // r[impl check.cve.git-rev]
        let source = PackageSource::from_manifest(name, dep.source()).ok();
        let query = match &source {
            Some(PackageSource::Npm(n)) => CveQuery::Npm {
                package: n.clone(),
                version: dep.version().to_string(),
            },
            Some(PackageSource::GitHub { .. }) => CveQuery::GitCommit {
                sha: locked.version.clone(),
            },
            None => CveQuery::Npm {
                package: name.clone(),
                version: dep.version().to_string(),
            },
        };
        tasks.push(CheckTask::Cve {
            name: name.clone(),
            query,
            ignore_cves: dep.ignore_cves().to_vec(),
        });

        // Queue outdated check per package
        if let Ok(source) = PackageSource::from_manifest(name, dep.source()) {
            tasks.push(CheckTask::Outdated {
                name: name.clone(),
                current: dep.version().to_string(),
                source,
            });
        }
    }

    // Run CVE + CDN hash checks in parallel (up to 5 concurrent)
    let results: Vec<_> = stream::iter(tasks)
        .map(|task| {
            let cve_checker = &cve_checker;
            let registry = &registry;
            async move {
                match task {
                    CheckTask::Cve {
                        name,
                        query,
                        ignore_cves,
                    } => {
                        let result = match &query {
                            CveQuery::Npm { package, version } => {
                                cve_checker.check(package, version).await
                            }
                            CveQuery::GitCommit { sha } => {
                                cve_checker.check_commit(sha).await
                            }
                        };
                        CheckResult::Cve {
                            name,
                            result,
                            ignore_cves,
                        }
                    }
                    CheckTask::CdnHash {
                        name,
                        source,
                        version,
                        local_sha256,
                        url,
                    } => {
                        let result = async {
                            let file_path = crate::url::extract_file_path(&url, &version)?;
                            let pkg_files = registry.get_package_files(&source, &version).await?;
                            log::debug!("{name}: looking for path '{file_path}' in CDN file list");
                            let entry = pkg_files.files.iter().find(|f| f.path == file_path);
                            if entry.is_none() {
                                log::debug!(
                                    "{name}: no matching file found in {} CDN entries",
                                    pkg_files.files.len()
                                );
                            }
                            Ok(entry.map(|e| e.hash.clone()))
                        }
                        .await;
                        CheckResult::CdnHash {
                            name,
                            result,
                            local_sha256,
                        }
                    }
                    CheckTask::Outdated {
                        name,
                        current,
                        source,
                    } => {
                        let latest = registry.get_package(&source).await.ok().and_then(|info| {
                            info.tags.latest.or_else(|| latest_stable(&info.versions))
                        });
                        CheckResult::Outdated {
                            name,
                            current,
                            latest,
                        }
                    }
                }
            }
        })
        // r[impl check.concurrency]
        .buffer_unordered(5)
        .collect()
        .await;

    let mut vulnerabilities: Vec<String> = Vec::new();
    let mut outdated: Vec<String> = Vec::new();

    for result in results {
        match result {
            CheckResult::Cve {
                name,
                result,
                ignore_cves,
            } => match result {
                Ok(vulns) => {
                    let unignored: Vec<_> = vulns
                        .iter()
                        // r[impl check.cve.ignore]
                        .filter(|v| !ignore_cves.contains(&v.id))
                        .collect();

                    // r[impl check.cve.allow-vulnerable]
                    if !unignored.is_empty() && !allow_vulnerable {
                        for vuln in &unignored {
                            vulnerabilities
                                .push(format!("  {name}: {} \u{2014} {}", vuln.id, vuln.summary));
                        }
                    }
                }
                // r[impl check.cve.query-error]
                Err(e) => {
                    vulnerabilities.push(format!("  {name}: could not check CVEs: {e}"));
                }
            },
            CheckResult::CdnHash {
                name,
                result,
                local_sha256,
            } => match result {
                Ok(Some(cdn_hash)) => match base64_to_hex(&cdn_hash) {
                    Some(cdn_hex) if cdn_hex != local_sha256 => {
                        integrity_errors
                            .push(format!("  {name}: vendored file does not match CDN hash"));
                    }
                    // r[impl check.integrity.cdn-decode-failure]
                    None => {
                        integrity_errors.push(format!(
                            "  {name}: could not decode CDN hash (invalid base64)"
                        ));
                    }
                    _ => {}
                },
                // r[impl check.integrity.cdn-missing-file]
                Ok(None) => {
                    integrity_errors
                        .push(format!("  {name}: file not found on CDN for verification"));
                }
                // r[impl check.integrity.cdn-network-error]
                Err(e) => {
                    integrity_errors.push(format!("  {name}: could not verify against CDN: {e}"));
                }
            },
            CheckResult::Outdated {
                name,
                current,
                latest,
            } => {
                // r[impl check.freshness.compare]
                if let Some(latest) = latest
                    && latest != current
                {
                    outdated.push(format!("  {name}: {current} -> {latest}"));
                }
            }
        }
    }

    let mut has_errors = false;

    if !integrity_errors.is_empty() {
        println!("Integrity:");
        for msg in &integrity_errors {
            println!("{msg}");
        }
        has_errors = true;
    }

    if !vulnerabilities.is_empty() {
        println!("Vulnerabilities:");
        for msg in &vulnerabilities {
            println!("{msg}");
        }
        has_errors = true;
    }

    // r[impl check.freshness.print]
    if !outdated.is_empty() {
        println!("Outdated:");
        for msg in &outdated {
            println!("{msg}");
        }
        // r[impl check.freshness.fail-on-outdated]
        if fail_on_outdated {
            has_errors = true;
        }
    }

    // r[impl check.exit.failure]
    if has_errors {
        anyhow::bail!("Check failed.");
    }

    // r[impl check.exit.success]
    println!("All checks passed.");
    Ok(())
}

fn base64_to_hex(b64: &str) -> Option<String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .ok()
        .map(hex::encode)
}
