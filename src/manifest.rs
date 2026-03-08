use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    Short(String),
    Extended {
        version: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(
            default,
            rename = "ignore-cves",
            skip_serializing_if = "Vec::is_empty"
        )]
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

    pub fn url(&self) -> Option<&str> {
        match self {
            Dependency::Short(_) => None,
            Dependency::Extended { url, .. } => url.as_deref(),
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
fn toml_key(key: &str) -> String {
    if key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Escape a TOML string value.
fn toml_string(val: &str) -> String {
    format!("\"{}\"", val.replace('\\', "\\\\").replace('"', "\\\""))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub dependencies: BTreeMap<String, Dependency>,
}

impl Manifest {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new("unpm.toml");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self {
                dependencies: BTreeMap::new(),
            })
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut out = String::new();
        writeln!(out, "[dependencies]")?;

        for (name, dep) in &self.dependencies {
            let key = toml_key(name);
            match dep {
                Dependency::Short(version) => {
                    writeln!(out, "{key} = {}", toml_string(version))?;
                }
                Dependency::Extended {
                    version,
                    source,
                    file,
                    url,
                    ignore_cves,
                } => {
                    let mut fields = vec![format!("version = {}", toml_string(version))];
                    if let Some(s) = source {
                        fields.push(format!("source = {}", toml_string(s)));
                    }
                    if let Some(f) = file {
                        fields.push(format!("file = {}", toml_string(f)));
                    }
                    if let Some(u) = url {
                        fields.push(format!("url = {}", toml_string(u)));
                    }
                    if !ignore_cves.is_empty() {
                        let cves: Vec<String> = ignore_cves.iter().map(|c| toml_string(c)).collect();
                        fields.push(format!("ignore-cves = [{}]", cves.join(", ")));
                    }
                    writeln!(out, "{key} = {{ {} }}", fields.join(", "))?;
                }
            }
        }

        std::fs::write("unpm.toml", out)?;
        Ok(())
    }
}
