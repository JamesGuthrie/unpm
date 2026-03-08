use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    Short(String),
    Extended {
        version: String,
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
        let contents = toml::to_string_pretty(self)?;
        std::fs::write("unpm.toml", contents)?;
        Ok(())
    }
}
