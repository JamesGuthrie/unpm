use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    // r[impl manifest.dep.short]
    Short(String),
    // r[impl manifest.dep.extended]
    Extended {
        version: String,
        // r[impl manifest.serial.omit-empty]
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        files: Option<Vec<String>>,
        // r[impl manifest.field.ignore-cves]
        #[serde(default, rename = "ignore-cves", skip_serializing_if = "Vec::is_empty")]
        ignore_cves: Vec<String>,
    },
}

impl Dependency {
    pub fn version(&self) -> &str {
        match self {
            Dependency::Short(v) => v,
            Dependency::Extended { version, .. } => version,
        }
    }

    pub fn source(&self) -> Option<&str> {
        match self {
            Dependency::Short(_) => None,
            Dependency::Extended { source, .. } => source.as_deref(),
        }
    }

    pub fn file(&self) -> Option<&str> {
        match self {
            Dependency::Short(_) => None,
            Dependency::Extended { file, .. } => file.as_deref(),
        }
    }

    pub fn files(&self) -> Option<&[String]> {
        match self {
            Dependency::Short(_) => None,
            Dependency::Extended { files, .. } => files.as_deref(),
        }
    }

    pub fn ignore_cves(&self) -> &[String] {
        match self {
            Dependency::Short(_) => &[],
            Dependency::Extended { ignore_cves, .. } => ignore_cves,
        }
    }
}

/// Quote a TOML key if it contains characters that require quoting.
// r[impl manifest.serial.key-quoting]
fn toml_key(key: &str) -> String {
    if key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Escape a TOML string value.
// r[impl manifest.serial.escaping]
fn toml_string(val: &str) -> String {
    format!("\"{}\"", val.replace('\\', "\\\\").replace('"', "\\\""))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    // r[impl manifest.serial.order]
    pub dependencies: BTreeMap<String, Dependency>,
}

impl Manifest {
    pub fn load() -> anyhow::Result<Self> {
        // r[impl manifest.file]
        Self::load_from(std::path::Path::new("unpm.toml"))
    }

    pub fn load_from(path: &std::path::Path) -> anyhow::Result<Self> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            let manifest: Self = toml::from_str(&contents)?;
            manifest.validate()?;
            Ok(manifest)
        } else {
            // r[impl manifest.file.missing]
            Ok(Self {
                dependencies: BTreeMap::new(),
            })
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for (name, dep) in &self.dependencies {
            if let Dependency::Extended { file, files, .. } = dep {
                // r[impl manifest.validation.file-files]
                if file.is_some() && files.is_some() {
                    anyhow::bail!("{name}: `file` and `files` are mutually exclusive");
                }
                // r[impl manifest.validation.files-empty]
                if let Some(fs) = files
                    && fs.is_empty()
                {
                    anyhow::bail!("{name}: `files` must not be empty");
                }
            }
        }
        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(std::path::Path::new("unpm.toml"))
    }

    pub fn save_to(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let mut out = String::new();
        writeln!(out, "[dependencies]")?;

        for (name, dep) in &self.dependencies {
            let key = toml_key(name);
            match dep {
                // r[impl manifest.serial.short]
                Dependency::Short(version) => {
                    writeln!(out, "{key} = {}", toml_string(version))?;
                }
                // r[impl manifest.serial.extended]
                Dependency::Extended {
                    version,
                    source,
                    file,
                    files,
                    ignore_cves,
                } => {
                    let mut fields = vec![format!("version = {}", toml_string(version))];
                    if let Some(s) = source {
                        fields.push(format!("source = {}", toml_string(s)));
                    }
                    if let Some(f) = file {
                        fields.push(format!("file = {}", toml_string(f)));
                    }
                    if let Some(fs) = files {
                        let items: Vec<String> = fs.iter().map(|f| toml_string(f)).collect();
                        fields.push(format!("files = [{}]", items.join(", ")));
                    }
                    if !ignore_cves.is_empty() {
                        let cves: Vec<String> =
                            ignore_cves.iter().map(|c| toml_string(c)).collect();
                        fields.push(format!("ignore-cves = [{}]", cves.join(", ")));
                    }
                    writeln!(out, "{key} = {{ {} }}", fields.join(", "))?;
                }
            }
        }

        std::fs::write(path, out)?;
        Ok(())
    }
}
