use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(flatten)]
    pub dependencies: BTreeMap<String, LockedDependency>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LockedDependency {
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub filename: String,
}

impl Lockfile {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new("unpm.lock");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Self::from_json(&contents)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let contents = self.to_json()?;
        std::fs::write("unpm.lock", contents)?;
        Ok(())
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
