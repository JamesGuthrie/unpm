use crate::lockfile::Lockfile;
use crate::manifest::Manifest;

pub fn list() -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let lockfile = Lockfile::load()?;

    // r[impl list.empty]
    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return Ok(());
    }

    for (name, dep) in &manifest.dependencies {
        let version = dep.version();
        // r[impl list.output.entry]
        println!("{name}@{version}");
        match lockfile.dependencies.get(name) {
            Some(locked) => {
                // r[impl list.output.files]
                for file in &locked.files {
                    println!("  {}", file.filename);
                }
            }
            // r[impl list.output.not-installed]
            None => println!("  (not installed)"),
        }
    }

    Ok(())
}
