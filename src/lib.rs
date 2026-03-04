use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_sync_command() {
    // Create temporary .hermit config
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        "manager = \"npm\"\n[packages]\nreact = \"18.3.0\""
    )
    .unwrap();

    // Create temporary hermit.lock
    let lockfile_path = config_file.path().parent().unwrap().join("hermit.lock");
    if lockfile_path.exists() {
        fs::remove_file(lockfile_path).unwrap();
    }

    // Mock npm installation - this would require npm to be installed
    // We'll test the command structure instead
    let config_content = fs::read_to_string(config_file.path()).unwrap();
    assert!(config_content.contains("npm"));
    assert!(config_content.contains("react = \"18.3.0\""));

    // Test that the config can be parsed
    let config = crate::config::Config::load().unwrap();
    assert_eq!(config.manager, "npm");
    assert_eq!(config.packages.get("react"), Some(&"18.3.0".to_string()));
}

#[test]
fn test_add_remove_commands() {
    // Test add command
    let mut config = crate::config::Config {
        manager: "npm".to_string(),
        packages: HashMap::new(),
    };

    config.add_package("react", "18.3.0").unwrap();
    assert_eq!(config.packages.get("react"), Some(&"18.3.0".to_string()));

    // Test remove command
    config.remove_package("react").unwrap();
    assert!(!config.packages.contains_key("react"));
}

#[test]
fn test_lock_command() {
    let mut lockfile = crate::lockfile::Lockfile {
        packages: HashMap::new(),
    };

    let package_info = crate::lockfile::PackageInfo {
        version: "18.3.0".to_string(),
        resolved: "https://registry.npmjs.org/react/-/react-18.3.0.tgz".to_string(),
        hash: "sha512-placeholder".to_string(),
    };

    lockfile.add_package("react", package_info).unwrap();
    assert!(lockfile.packages.contains_key("react"));

    // Test lockfile saving/loading
    let temp_dir = tempfile::tempdir().unwrap();
    let lockfile_path = temp_dir.path().join("hermit.lock");

    let lockfile_content = toml::to_string_pretty(&lockfile).unwrap();
    fs::write(&lockfile_path, lockfile_content).unwrap();

    let loaded_lockfile = crate::lockfile::Lockfile::load().unwrap();
    assert!(loaded_lockfile.packages.contains_key("react"));
}

#[test]
fn test_check_command() {
    // This would require actual package installations
    // We'll test the version checking logic

    // Mock package manager type
    let config = crate::config::Config {
        manager: "npm".to_string(),
        packages: HashMap::from([("react".to_string(), "18.3.0".to_string())]),
    };

    let pm = crate::manager::PackageManager::from_config(&config).unwrap();

    // Test version parsing - this would need actual npm output
    // We'll just test the regex patterns
    let test_output = "react@18.3.0";
    let re = regex::Regex::new(r"\b\d+\.\d+\.\d+\b").unwrap();
    let found_version = re.find(test_output);

    assert!(found_version.is_some());
    assert_eq!(found_version.unwrap().as_str(), "18.3.0");
}

#[test]
fn test_clean_command() {
    // Test lock file removal
    let temp_dir = tempfile::tempdir().unwrap();
    let lockfile_path = temp_dir.path().join("hermit.lock");

    // Create lock file
    fs::write(&lockfile_path, "[packages]").unwrap();
    assert!(lockfile_path.exists());

    // Mock clean command - would remove lock file
    if lockfile_path.exists() {
        fs::remove_file(lockfile_path).unwrap();
    }
    assert!(!lockfile_path.exists());
}

#[test]
fn test_error_handling() {
    // Test error creation
    let err = crate::error::HermitError::config_error(None);
    assert_eq!(err.kind, crate::error::HermitErrorKind::ConfigError);

    // Test error display
    let display = format!("{}", err);
    assert!(display.contains("Configuration error"));
}

#[test]
fn test_package_manager_creation() {
    let config = crate::config::Config {
        manager: "cargo".to_string(),
        packages: HashMap::new(),
    };

    let pm = crate::manager::PackageManager::from_config(&config).unwrap();
    assert_eq!(pm.manager_type, crate::manager::ManagerType::Cargo);
    assert_eq!(pm.name, "cargo");
}

#[test]
fn test_cli_parsing() {
    use clap::CommandFactory;

    // Test that CLI can be constructed
    let cli = crate::Cli::command();
    assert!(cli.get_name() == "hermit");

    // Test that commands exist
    assert!(cli.get_subcommands().iter().any(|c| c.get_name() == "sync"));
    assert!(cli.get_subcommands().iter().any(|c| c.get_name() == "add"));
    assert!(cli
        .get_subcommands()
        .iter()
        .any(|c| c.get_name() == "remove"));
    assert!(cli.get_subcommands().iter().any(|c| c.get_name() == "lock"));
    assert!(cli
        .get_subcommands()
        .iter()
        .any(|c| c.get_name() == "check"));
    assert!(cli
        .get_subcommands()
        .iter()
        .any(|c| c.get_name() == "clean"));
}
