use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_output_dir() -> String {
    "static/vendor".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::path::Path::new(".unpm.toml");
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }
}
