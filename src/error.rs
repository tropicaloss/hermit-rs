use colored::Colorize;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct HermitError {
    kind: HermitErrorKind,
    source: Option<Box<dyn Error + 'static>>,
}

#[derive(Debug)]
pub enum HermitErrorKind {
    ConfigError,
    LockfileError,
    PackageManagerError,
    CommandExecutionError,
    VersionMismatchError,
    CleanupError,
}

impl HermitError {
    pub fn new(kind: HermitErrorKind, source: Option<Box<dyn Error + 'static>>) -> Self {
        HermitError { kind, source }
    }

    pub fn config_error(source: Option<Box<dyn Error + 'static>>) -> Self {
        Self::new(HermitErrorKind::ConfigError, source)
    }

    pub fn lockfile_error(source: Option<Box<dyn Error + 'static>>) -> Self {
        Self::new(HermitErrorKind::LockfileError, source)
    }

    pub fn package_manager_error(source: Option<Box<dyn Error + 'static>>) -> Self {
        Self::new(HermitErrorKind::PackageManagerError, source)
    }

    pub fn command_execution_error(source: Option<Box<dyn Error + 'static>>) -> Self {
        Self::new(HermitErrorKind::CommandExecutionError, source)
    }

    pub fn version_mismatch_error() -> Self {
        Self::new(HermitErrorKind::VersionMismatchError, None)
    }

    pub fn cleanup_error(source: Option<Box<dyn Error + 'static>>) -> Self {
        Self::new(HermitErrorKind::CleanupError, source)
    }
}

impl fmt::Display for HermitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            HermitErrorKind::ConfigError => write!(f, "Configuration error"),
            HermitErrorKind::LockfileError => write!(f, "Lockfile error"),
            HermitErrorKind::PackageManagerError => write!(f, "Package manager error"),
            HermitErrorKind::CommandExecutionError => write!(f, "Command execution error"),
            HermitErrorKind::VersionMismatchError => write!(f, "Version mismatch error"),
            HermitErrorKind::CleanupError => write!(f, "Cleanup error"),
        }?;

        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        }

        Ok(())
    }
}

impl Error for HermitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|boxed| &**boxed)
    }
}

pub type Result<T> = std::result::Result<T, HermitError>;

pub fn log_error(err: HermitError) {
    eprintln!("{}: {}", "Error".red().bold(), err);

    if let Some(source) = err.source() {
        eprintln!("Caused by: {}", source);
    }

    match err.kind {
        HermitErrorKind::ConfigError => {
            eprintln!("Check your .hermit configuration file");
            eprintln!("Ensure the manager is supported and packages are properly formatted");
        }
        HermitErrorKind::LockfileError => {
            eprintln!("Check your hermit.lock file");
            eprintln!("Ensure it's properly formatted and accessible");
        }
        HermitErrorKind::PackageManagerError => {
            eprintln!("Check if your package manager is installed");
            eprintln!("Run: {}", "which <manager>".yellow());
        }
        HermitErrorKind::CommandExecutionError => {
            eprintln!("Command execution failed");
            eprintln!("Check your system's PATH and permissions");
        }
        HermitErrorKind::VersionMismatchError => {
            eprintln!("Package versions do not match");
            eprintln!(
                "Run: {} to install correct versions",
                "hermit sync".yellow()
            );
        }
        HermitErrorKind::CleanupError => {
            eprintln!("Cleanup failed");
            eprintln!("Manual cleanup may be required");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermit_error_display() {
        let err = HermitError::new(
            HermitErrorKind::ConfigError,
            Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))),
        );

        let display = format!("{}", err);
        assert!(display.contains("Configuration error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_hermit_error_kind() {
        let err = HermitError::config_error(None);
        assert_eq!(err.kind, HermitErrorKind::ConfigError);

        let err = HermitError::package_manager_error(None);
        assert_eq!(err.kind, HermitErrorKind::PackageManagerError);
    }

    #[test]
    fn test_log_error() {
        let err = HermitError::new(
            HermitErrorKind::CommandExecutionError,
            Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "permission denied",
            ))),
        );

        // This would print to stderr, we just check it doesn't panic
        log_error(err);
    }
}
