use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // r[impl config.output-dir.custom]
    // r[impl config.format.empty]
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_canonical")]
    pub canonical: bool,
}

// r[impl config.output-dir.default]
fn default_output_dir() -> String {
    "static/vendor".to_string()
}

// r[impl config.canonical.default]
fn default_canonical() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            canonical: default_canonical(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // r[impl config.file.name]
        let path = std::path::Path::new(".unpm.toml");
        if path.exists() {
            // r[impl config.format.toml]
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            // r[impl config.file.missing]
            Ok(Self::default())
        }
    }
}
