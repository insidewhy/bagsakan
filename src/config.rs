use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_validator_pattern")]
    pub validator_pattern: String,

    #[serde(default = "default_source_files")]
    pub source_files: String,

    #[serde(default = "default_validator_file")]
    pub validator_file: String,

    #[serde(default = "default_use_js_extensions")]
    pub use_js_extensions: bool,

    #[serde(default = "default_follow_external_imports")]
    pub follow_external_imports: bool,

    #[serde(default)]
    pub exclude_packages: Vec<String>,

    #[serde(default)]
    pub conditions: Vec<String>,
}

fn default_validator_pattern() -> String {
    "validate%(type)".to_string()
}

fn default_source_files() -> String {
    "src/**/*.ts".to_string()
}

fn default_validator_file() -> String {
    "src/validators.ts".to_string()
}

fn default_use_js_extensions() -> bool {
    false
}

fn default_follow_external_imports() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            validator_pattern: default_validator_pattern(),
            source_files: default_source_files(),
            validator_file: default_validator_file(),
            use_js_extensions: default_use_js_extensions(),
            follow_external_imports: default_follow_external_imports(),
            exclude_packages: Vec::new(),
            conditions: Vec::new(),
        }
    }
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn get_pattern_regex(&self) -> String {
        self.validator_pattern
            .replace("%(type)", r"([A-Z][a-zA-Z]+)")
    }
}
