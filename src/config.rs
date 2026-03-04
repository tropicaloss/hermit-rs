use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub manager: String,
    pub packages: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let config_path = PathBuf::from(".hermit");
        if !config_path.exists() {
            return Err(anyhow::anyhow!(".hermit file not found"));
        }

        let config_content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_content)?;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_path = PathBuf::from(".hermit");
        let config_content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, config_content)?;
        Ok(())
    }

    pub fn add_package(&mut self, package: &str, version: &str) -> Result<(), anyhow::Error> {
        if self.packages.contains_key(package) {
            return Err(anyhow::anyhow!(format!(
                "Package {} already exists",
                package
            )));
        }
        self.packages
            .insert(package.to_string(), version.to_string());
        Ok(())
    }

    pub fn remove_package(&mut self, package: &str) -> Result<(), anyhow::Error> {
        if !self.packages.contains_key(package) {
            return Err(anyhow::anyhow!(format!("Package {} not found", package)));
        }
        self.packages.remove(package);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load() -> Result<(), anyhow::Error> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "manager = \"bun\"\n[packages]\nreact = \"18.3.0\"")?;

        let config = Config::load()?;
        assert_eq!(config.manager, "bun");
        assert_eq!(config.packages.get("react"), Some(&"18.3.0".to_string()));

        Ok(())
    }

    #[test]
    fn test_config_add_package() -> Result<(), anyhow::Error> {
        let mut config = Config {
            manager: "bun".to_string(),
            packages: HashMap::new(),
        };

        config.add_package("react", "18.3.0")?;
        assert_eq!(config.packages.get("react"), Some(&"18.3.0".to_string()));

        Ok(())
    }

    #[test]
    fn test_config_remove_package() -> Result<(), anyhow::Error> {
        let mut config = Config {
            manager: "bun".to_string(),
            packages: HashMap::from([("react".to_string(), "18.3.0".to_string())]),
        };

        config.remove_package("react")?;
        assert!(!config.packages.contains_key("react"));

        Ok(())
    }
}
