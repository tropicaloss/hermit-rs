use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoManifest {
    pub package: Option<CargoPackage>,
    #[serde(default, deserialize_with = "deserialize_dependencies")]
    pub dependencies: HashMap<String, String>,
    #[serde(
        default,
        rename = "dev-dependencies",
        deserialize_with = "deserialize_dependencies"
    )]
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DependencySpec {
    pub version: Option<String>,
}

fn deserialize_dependencies<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value: toml::Value = Deserialize::deserialize(deserializer)?;
    let mut deps = HashMap::new();

    if let toml::Value::Table(table) = value {
        for (key, val) in table {
            let version = match val {
                toml::Value::String(s) => s,
                toml::Value::Table(t) => t
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "*".to_string()),
                _ => "*".to_string(),
            };
            deps.insert(key, version);
        }
    }

    Ok(deps)
}

impl CargoManifest {
    pub fn load() -> Result<Self, anyhow::Error> {
        let cargo_path = PathBuf::from("Cargo.toml");
        if !cargo_path.exists() {
            return Err(anyhow::anyhow!("Cargo.toml not found in current directory"));
        }

        let cargo_content = std::fs::read_to_string(&cargo_path)?;
        let manifest: CargoManifest = toml::from_str(&cargo_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse Cargo.toml: {}", e))?;

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

pub fn compute_sha256(path: &std::path::Path) -> Result<String, anyhow::Error> {
    use sha2::{Digest, Sha256};

    if path.is_file() {
        return compute_sha256_file(path);
    }

    if path.is_dir() {
        return compute_sha256_dir(path);
    }

    Err(anyhow::anyhow!(
        "Path is neither file nor directory: {:?}",
        path
    ))
}

fn compute_sha256_file(path: &std::path::Path) -> Result<String, anyhow::Error> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("sha256:{}", hex::encode(result)))
}

fn compute_sha256_dir(dir_path: &std::path::Path) -> Result<String, anyhow::Error> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    let mut entries: Vec<_> = std::fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        hasher.update(file_name.as_bytes());

        if path.is_file() {
            let content = std::fs::read(&path)?;
            hasher.update(&content);
        }
    }

    let result = hasher.finalize();
    Ok(format!("sha256:{}", hex::encode(result)))
}

pub fn find_crate_in_registry(crate_name: &str, version: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    let cache_path = PathBuf::from(&home)
        .join(".cargo")
        .join("registry")
        .join("cache")
        .join("index.crates.io-1949cf8c6b5b557f");

    if cache_path.exists() {
        let crate_file = cache_path.join(format!("{}-{}.crate", crate_name, version));
        if crate_file.exists() {
            return Some(crate_file);
        }
    }

    let src_path = PathBuf::from(&home)
        .join(".cargo")
        .join("registry")
        .join("src")
        .join("index.crates.io-1949cf8c6b5b557f");

    if src_path.exists() {
        let crate_dir = src_path.join(format!("{}-{}", crate_name, version));
        if crate_dir.exists() {
            return Some(crate_dir);
        }
    }

    None
}

pub fn compute_crate_hash(crate_name: &str, version: &str, verbose: bool) -> String {
    if let Some(crate_path) = find_crate_in_registry(crate_name, version) {
        if verbose {
            println!(
                "Computing hash for {}-{} from {:?}...",
                crate_name, version, crate_path
            );
        }
        if let Ok(hash) = compute_sha256(&crate_path) {
            return hash;
        }
    }

    if verbose {
        println!("Could not find crate in registry, using placeholder hash");
    }

    format!(
        "sha256:{}",
        "0000000000000000000000000000000000000000000000000000000000000000"
    )
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
    fn test_cargo_manifest_with_table_deps() -> Result<(), anyhow::Error> {
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
clap = { version = "4.5.60", features = ["derive"] }
toml = "0.8"
"#;
        let manifest: CargoManifest = toml::from_str(content)?;
        assert_eq!(
            manifest.dependencies.get("clap"),
            Some(&"4.5.60".to_string())
        );
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
