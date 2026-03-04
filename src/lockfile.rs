use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lockfile {
    pub packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageInfo {
    pub version: String,
    pub resolved: String,
    pub hash: String,
}

impl Lockfile {
    pub fn load() -> Result<Self, anyhow::Error> {
        let lockfile_path = PathBuf::from("hermit.lock");
        if !lockfile_path.exists() {
            return Ok(Lockfile {
                packages: HashMap::new(),
            });
        }

        let lockfile_content = std::fs::read_to_string(lockfile_path)?;
        let lockfile: Lockfile = toml::from_str(&lockfile_content)?;

        Ok(lockfile)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let lockfile_path = PathBuf::from("hermit.lock");
        let lockfile_content = toml::to_string_pretty(self)?;
        std::fs::write(lockfile_path, lockfile_content)?;
        Ok(())
    }

    pub fn add_package(&mut self, name: &str, info: PackageInfo) -> Result<(), anyhow::Error> {
        self.packages.insert(name.to_string(), info);
        Ok(())
    }

    pub fn get_package(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lockfile_load() -> Result<(), anyhow::Error> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "[packages.react]\nversion = \"18.3.0\"\nresolved = \"https://registry.npmjs.org/react/-/react-18.3.0.tgz\"\nhash = \"sha512-zB7H3n/n6/aJmmAf+K6tKD5B2j9HoDuLdpTHxqSPTONM4qWBnECbgGNjbhyMbf3HoHa/zDxFBfHnIqGxnFjA==\"")?;

        let lockfile = Lockfile::load()?;
        assert!(lockfile.packages.contains_key("react"));

        Ok(())
    }

    #[test]
    fn test_lockfile_add_package() -> Result<(), anyhow::Error> {
        let mut lockfile = Lockfile {
            packages: HashMap::new(),
        };

        let package_info = PackageInfo {
            version: "18.3.0".to_string(),
            resolved: "https://registry.npmjs.org/react/-/react-18.3.0.tgz".to_string(),
            hash: "sha512-zB7H3n/n6/aJmmAf+K6tKD5B2j9HoDuLdpTHxqSPTONM4qWBnECbgGNjbhyMbf3HoHa/zDxFBfHnIqGxnFjA==".to_string(),
        };

        lockfile.add_package("react", package_info)?;
        assert!(lockfile.packages.contains_key("react"));

        Ok(())
    }

    #[test]
    fn test_lockfile_get_package() -> Result<(), anyhow::Error> {
        let mut lockfile = Lockfile {
            packages: HashMap::new(),
        };

        let package_info = PackageInfo {
            version: "18.3.0".to_string(),
            resolved: "https://registry.npmjs.org/react/-/react-18.3.0.tgz".to_string(),
            hash: "sha512-zB7H3n/n6/aJmmAf+K6tKD5B2j9HoDuLdpTHxqSPTONM4qWBnECbgGNjbhyMbf3HoHa/zDxFBfHnIqGxnFjA==".to_string(),
        };

        lockfile.add_package("react", package_info)?;
        assert!(lockfile.get_package("react").is_some());

        Ok(())
    }
}
