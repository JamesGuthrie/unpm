use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    // r[impl lockfile.structure.top-level]
    #[serde(flatten)]
    pub dependencies: BTreeMap<String, LockedDependency>,
}

// r[impl lockfile.structure.dependency]
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct LockedDependency {
    pub version: String,
    // r[impl lockfile.structure.multi-file]
    pub files: Vec<LockedFile>,
}

// r[impl lockfile.structure.file-entry]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LockedFile {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub filename: String,
}

impl<'de> Deserialize<'de> for LockedDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            version: String,
            files: Option<Vec<LockedFile>>,
            url: Option<String>,
            sha256: Option<String>,
            size: Option<u64>,
            filename: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        let has_old = raw.url.is_some()
            || raw.sha256.is_some()
            || raw.size.is_some()
            || raw.filename.is_some();
        let has_new = raw.files.is_some();

        // r[impl lockfile.migration.conflict]
        if has_old && has_new {
            return Err(serde::de::Error::custom(
                "corrupt lockfile: contains both old flat fields and new files array",
            ));
        }

        if has_new {
            Ok(LockedDependency {
                version: raw.version,
                files: raw.files.unwrap(),
            })
        // r[impl lockfile.migration.old-format]
        } else if has_old {
            let url = raw
                .url
                .ok_or_else(|| serde::de::Error::missing_field("url"))?;
            let sha256 = raw
                .sha256
                .ok_or_else(|| serde::de::Error::missing_field("sha256"))?;
            let size = raw
                .size
                .ok_or_else(|| serde::de::Error::missing_field("size"))?;
            let filename = raw
                .filename
                .ok_or_else(|| serde::de::Error::missing_field("filename"))?;

            Ok(LockedDependency {
                version: raw.version,
                files: vec![LockedFile {
                    url,
                    sha256,
                    size,
                    filename,
                }],
            })
        // r[impl lockfile.migration.no-file-data]
        } else {
            Err(serde::de::Error::custom(
                "lockfile entry has neither files array nor legacy flat fields",
            ))
        }
    }
}

impl Lockfile {
    pub fn load() -> anyhow::Result<Self> {
        // r[impl lockfile.file.name]
        let path = std::path::Path::new("unpm.lock");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Self::from_json(&contents)
        } else {
            // r[impl lockfile.file.missing]
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let contents = self.to_json()?;
        std::fs::write("unpm.lock", contents)?;
        Ok(())
    }

    // r[impl lockfile.file.format]
    // r[impl lockfile.serialization.canonical]
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    // r[impl lockfile.serialization.roundtrip]
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}
