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
        let file_info = lockfile
            .dependencies
            .get(name)
            .map(|l| l.filename.as_str())
            .unwrap_or("(not installed)");
        println!("{name}@{version}  {file_info}");
    }

    Ok(())
}
