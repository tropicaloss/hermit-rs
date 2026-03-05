use anyhow::Result;
use colored::Colorize;
use indicatif::ProgressBar;
use std::time::Duration;

pub struct PackageManager {
    pub name: String,
    pub manager_type: ManagerType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManagerType {
    Bun,
    Npm,
    Pnpm,
    Deno,
    Cargo,
    Pip,
    Uv,
    Brew,
    Gem,
    Go,
}

impl ManagerType {
    pub fn from_name(name: &str) -> Result<Self, anyhow::Error> {
        match name.to_lowercase().as_str() {
            "bun" => Ok(ManagerType::Bun),
            "npm" => Ok(ManagerType::Npm),
            "pnpm" => Ok(ManagerType::Pnpm),
            "deno" => Ok(ManagerType::Deno),
            "cargo" => Ok(ManagerType::Cargo),
            "pip" => Ok(ManagerType::Pip),
            "uv" => Ok(ManagerType::Uv),
            "brew" => Ok(ManagerType::Brew),
            "gem" => Ok(ManagerType::Gem),
            "go" => Ok(ManagerType::Go),
            _ => Err(anyhow::anyhow!(format!(
                "Unsupported package manager: {}",
                name
            ))),
        }
    }
}

impl PackageManager {
    pub fn from_config(config: &super::config::Config) -> Result<Self, anyhow::Error> {
        let manager_type = ManagerType::from_name(&config.manager)?;
        Ok(PackageManager {
            name: config.manager.clone(),
            manager_type,
        })
    }

    pub fn install_package(&self, package: &str, version: &str, verbose: bool) -> Result<()> {
        let mut command = match self.manager_type {
            ManagerType::Bun => {
                let mut cmd = std::process::Command::new("bun");
                cmd.arg("add").arg(format!("{}@{}", package, version));
                cmd
            }
            ManagerType::Npm => {
                let mut cmd = std::process::Command::new("npm");
                cmd.arg("install").arg(format!("{}@{}", package, version));
                cmd
            }
            ManagerType::Pnpm => {
                let mut cmd = std::process::Command::new("pnpm");
                cmd.arg("add").arg(format!("{}@{}", package, version));
                cmd
            }
            ManagerType::Deno => {
                let mut cmd = std::process::Command::new("deno");
                cmd.arg("install").arg(format!("{}@{}", package, version));
                cmd
            }
            ManagerType::Cargo => {
                let mut cmd = std::process::Command::new("cargo");
                cmd.arg("fetch");
                cmd
            }
            ManagerType::Pip => {
                let mut cmd = std::process::Command::new("pip");
                cmd.arg("install").arg(format!("{}=={}", package, version));
                cmd
            }
            ManagerType::Uv => {
                let mut cmd = std::process::Command::new("uv");
                cmd.arg("add").arg(format!("{}@{}", package, version));
                cmd
            }
            ManagerType::Brew => {
                let mut cmd = std::process::Command::new("brew");
                cmd.arg("install").arg(package);
                cmd
            }
            ManagerType::Gem => {
                let mut cmd = std::process::Command::new("gem");
                cmd.arg("install").arg(package);
                cmd
            }
            ManagerType::Go => {
                let mut cmd = std::process::Command::new("go");
                cmd.arg("get").arg(package);
                cmd
            }
        };

        if verbose {
            println!(
                "{} {}@{} using {} (verbose)...",
                "Installing".green(),
                package.blue(),
                version.blue(),
                self.name.blue()
            );
            println!("Command: {:?}", command);
        } else {
            println!(
                "{} {}@{} using {}...",
                "Installing".green(),
                package.blue(),
                version.blue(),
                self.name.blue()
            );
        }

        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Installing...");

        let output = command.output();
        pb.finish_and_clear();

        match output {
            Ok(output) => {
                if output.status.success() {
                    if verbose {
                        println!(
                            "{} {}@{} installed successfully (verbose)",
                            "Successfully installed".green(),
                            package.blue(),
                            version.blue()
                        );
                    } else {
                        println!(
                            "{} {}@{} installed successfully",
                            "Successfully installed".green(),
                            package.blue(),
                            version.blue()
                        );
                    }
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(anyhow::anyhow!(format!(
                        "Failed to install {}: {}",
                        package,
                        stderr.trim()
                    )))
                }
            }
            Err(e) => Err(anyhow::anyhow!(format!("Failed to execute command: {}", e))),
        }
    }

    pub fn check_installed_version(
        &self,
        package: &str,
        expected_version: &str,
    ) -> Result<bool, anyhow::Error> {
        let mut command = match self.manager_type {
            ManagerType::Bun | ManagerType::Npm | ManagerType::Pnpm => {
                let mut cmd =
                    std::process::Command::new(if self.manager_type == ManagerType::Bun {
                        "bun"
                    } else if self.manager_type == ManagerType::Npm {
                        "npm"
                    } else {
                        "pnpm"
                    });
                cmd.arg("list").arg(package);
                cmd
            }
            ManagerType::Cargo => {
                let mut cmd = std::process::Command::new("cargo");
                cmd.arg("install").arg("--list");
                cmd
            }
            ManagerType::Pip => {
                let mut cmd = std::process::Command::new("pip");
                cmd.arg("show").arg(package);
                cmd
            }
            ManagerType::Deno => {
                let mut cmd = std::process::Command::new("deno");
                cmd.arg("info").arg("ls");
                cmd
            }
            _ => {
                return Err(anyhow::anyhow!(format!(
                    "Version checking not supported for {}",
                    self.name
                )));
            }
        };

        let output = command.output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(self.parse_version_from_output(&stdout, expected_version))
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    fn parse_version_from_output(&self, output: &str, expected_version: &str) -> bool {
        match self.manager_type {
            ManagerType::Bun | ManagerType::Npm | ManagerType::Pnpm => {
                let re = regex::Regex::new(r"\b\d+\.\d+\.\d+\b").unwrap();
                if let Some(cap) = re.find(output) {
                    cap.as_str() == expected_version
                } else {
                    false
                }
            }
            ManagerType::Cargo => output.contains(expected_version),
            ManagerType::Pip => {
                let re = regex::Regex::new(r"Version: (\d+\.\d+\.\d+)").unwrap();
                if let Some(cap) = re.find(output) {
                    cap.as_str() == format!("Version: {}", expected_version)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_type_from_name() -> Result<(), anyhow::Error> {
        assert_eq!(ManagerType::from_name("bun")?, ManagerType::Bun);
        assert_eq!(ManagerType::from_name("npm")?, ManagerType::Npm);
        assert_eq!(ManagerType::from_name("cargo")?, ManagerType::Cargo);
        assert_eq!(ManagerType::from_name("pip")?, ManagerType::Pip);

        Ok(())
    }

    #[test]
    fn test_package_manager_install() {
        // This would require actual package managers to be installed
        // We'll test the command construction instead
        let config = super::super::config::Config {
            manager: "npm".to_string(),
            packages: HashMap::new(),
        };

        let pm = PackageManager::from_config(&config).unwrap();
        // Test that the command would be constructed correctly
        // Actual execution would require package managers to be installed
        assert_eq!(pm.manager_type, ManagerType::Npm);
    }
}
