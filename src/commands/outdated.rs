use crate::manifest::Manifest;
use crate::registry::{PackageSource, Registry, latest_stable};
use futures::stream::{self, StreamExt};

pub async fn outdated() -> anyhow::Result<()> {
    let manifest = Manifest::load()?;

    // r[impl outdated.empty]
    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return Ok(());
    }

    let client = reqwest::Client::new();
    let registry = Registry::with_client(client);

    let entries: Vec<_> = manifest
        .dependencies
        .iter()
        // r[impl outdated.resolution.source]
        .filter_map(|(name, dep)| {
            let source = PackageSource::from_manifest(name, dep.source()).ok()?;
            Some((name.clone(), dep.version().to_string(), source))
        })
        .collect();

    let results: Vec<_> = stream::iter(entries)
        .map(|(name, current, source)| {
            let registry = &registry;
            async move {
                let latest = registry.get_package(&source).await.ok().and_then(|info| {
                    // r[impl outdated.resolution.latest]
                    info.tags.latest.or_else(|| latest_stable(&info.versions))
                });
                (name, current, latest)
            }
        })
        // r[impl outdated.concurrency]
        .buffer_unordered(5)
        .collect()
        .await;

    let mut found = false;
    for (name, current, latest) in &results {
        // r[impl outdated.comparison]
        if let Some(latest) = latest
            && latest != current
        {
            if !found {
                // r[impl outdated.output.header]
                println!("Outdated dependencies:");
                found = true;
            }
            // r[impl outdated.output.entry]
            println!("  {name}: {current} -> {latest}");
        }
    }

    if !found {
        // r[impl outdated.output.up-to-date]
        println!("All dependencies are up to date.");
    }

    Ok(())
}
