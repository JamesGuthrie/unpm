use crate::manifest::Manifest;
use crate::registry::{PackageSource, Registry};
use futures::stream::{self, StreamExt};

pub async fn outdated() -> anyhow::Result<()> {
    let manifest = Manifest::load()?;

    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return Ok(());
    }

    let client = reqwest::Client::new();
    let registry = Registry::with_client(client);

    let entries: Vec<_> = manifest
        .dependencies
        .iter()
        .filter_map(|(name, dep)| {
            let source = PackageSource::from_manifest(name, dep.source()).ok()?;
            Some((name.clone(), dep.version().to_string(), source))
        })
        .collect();

    let results: Vec<_> = stream::iter(entries)
        .map(|(name, current, source)| {
            let registry = &registry;
            async move {
                let latest = registry
                    .get_package(&source)
                    .await
                    .ok()
                    .and_then(|info| {
                        info.tags.latest.or_else(|| {
                            let mut stable: Vec<_> = info.versions.iter()
                                .filter_map(|v| {
                                    let sv = semver::Version::parse(&v.version).ok()?;
                                    if sv.pre.is_empty() { Some((v.version.clone(), sv)) } else { None }
                                })
                                .collect();
                            stable.sort_by(|a, b| b.1.cmp(&a.1));
                            stable.into_iter().next().map(|(s, _)| s)
                        })
                    });
                (name, current, latest)
            }
        })
        .buffer_unordered(5)
        .collect()
        .await;

    let mut found = false;
    for (name, current, latest) in &results {
        if let Some(latest) = latest
            && latest != current {
            if !found {
                println!("Outdated dependencies:");
                found = true;
            }
            println!("  {name}: {current} -> {latest}");
        }
    }

    if !found {
        println!("All dependencies are up to date.");
    }

    Ok(())
}
