use crate::lockfile::Lockfile;
use crate::manifest::Manifest;

pub fn list() -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;

    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return Ok(());
    }

    for (name, dep) in &manifest.dependencies {
        let version = dep.version();
        println!("{name}@{version}");
        match lockfile.dependencies.get(name) {
            Some(locked) => {
                for file in &locked.files {
                    println!("  {}", file.filename);
                }
            }
            None => println!("  (not installed)"),
        }
    }

    Ok(())
}
