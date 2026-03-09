use crate::manifest::Manifest;
use crate::registry::{latest_stable, PackageSource, Registry};
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
                        info.tags.latest.or_else(|| latest_stable(&info.versions))
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
