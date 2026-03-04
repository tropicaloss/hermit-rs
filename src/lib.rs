#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::lockfile::{Lockfile, PackageInfo};
    use crate::manager::{ManagerType, PackageManager};
    use std::collections::HashMap;

    #[test]
    fn test_config_add_remove() {
        let mut config = Config {
            manager: "npm".to_string(),
            packages: HashMap::new(),
        };

        config.add_package("react", "18.3.0").unwrap();
        assert_eq!(config.packages.get("react"), Some(&"18.3.0".to_string()));

        config.remove_package("react").unwrap();
        assert!(!config.packages.contains_key("react"));
    }

    #[test]
    fn test_lockfile_operations() {
        let mut lockfile = Lockfile {
            packages: HashMap::new(),
        };

        let package_info = PackageInfo {
            version: "18.3.0".to_string(),
            resolved: "https://registry.npmjs.org/react/-/react-18.3.0.tgz".to_string(),
            hash: "sha512-zB7H3n/n6/aJmmAf+K6tKD5B2j9HoDuLdpTHxqSPTONM4qWBnECbgGNjbhyMbf3HoHa/zDxFBfHnIqGxnFjA==".to_string(),
        };

        lockfile.add_package("react", package_info).unwrap();
        assert!(lockfile.packages.contains_key("react"));
        assert!(lockfile.get_package("react").is_some());
    }

    #[test]
    fn test_manager_type_from_name() {
        assert_eq!(ManagerType::from_name("bun").unwrap(), ManagerType::Bun);
        assert_eq!(ManagerType::from_name("npm").unwrap(), ManagerType::Npm);
        assert_eq!(ManagerType::from_name("cargo").unwrap(), ManagerType::Cargo);
        assert_eq!(ManagerType::from_name("pip").unwrap(), ManagerType::Pip);
    }

    #[test]
    fn test_package_manager_creation() {
        let config = Config {
            manager: "cargo".to_string(),
            packages: HashMap::new(),
        };

        let pm = PackageManager::from_config(&config).unwrap();
        assert_eq!(pm.manager_type, ManagerType::Cargo);
        assert_eq!(pm.name, "cargo");
    }

    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;

        let cli = crate::Cli::command();
        assert!(cli.get_name() == "hermit");

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
}
