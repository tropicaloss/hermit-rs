use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoManifest {
    pub package: Option<CargoPackage>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
}

impl CargoManifest {
    pub fn load() -> Result<Self, anyhow::Error> {
        let cargo_path = PathBuf::from("Cargo.toml");
        if !cargo_path.exists() {
            return Err(anyhow::anyhow!("Cargo.toml not found"));
        }

        let cargo_content = std::fs::read_to_string(&cargo_path)?;
        let manifest: CargoManifest = toml::from_str(&cargo_content)?;

        Ok(manifest)
    }

    pub fn get_all_dependencies(&self) -> HashMap<String, String> {
        let mut all_deps = self.dependencies.clone();
        for (key, value) in self.dev_dependencies.iter() {
            if !all_deps.contains_key(key) {
                all_deps.insert(key.clone(), value.clone());
            }
        }
        all_deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cargo_manifest_load() -> Result<(), anyhow::Error> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\ntoml = \"0.8\""
        )?;

        let manifest = CargoManifest::load()?;
        assert_eq!(manifest.package.unwrap().name, "test");
        assert_eq!(manifest.dependencies.get("toml"), Some(&"0.8".to_string()));

        Ok(())
    }

    #[test]
    fn test_get_all_dependencies() -> Result<(), anyhow::Error> {
        let manifest = CargoManifest {
            package: None,
            dependencies: HashMap::from([("toml".to_string(), "0.8".to_string())]),
            dev_dependencies: HashMap::from([("tempfile".to_string(), "3.8".to_string())]),
        };

        let all_deps = manifest.get_all_dependencies();
        assert_eq!(all_deps.len(), 2);
        assert!(all_deps.contains_key("toml"));
        assert!(all_deps.contains_key("tempfile"));

        Ok(())
    }
}
