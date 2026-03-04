use crate::cli::Commands;
use crate::config::Config;
use crate::error::Result;
use crate::lockfile::Lockfile;
use crate::manager::PackageManager;
use anyhow::Context;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn run(command: Commands) -> Result<()> {
    match command {
        Commands::Sync => sync()?,
        Commands::Add { package, version } => add_package(package, version)?,
        Commands::Remove { package } => remove_package(package)?,
        Commands::Lock => lock()?,
        Commands::Check => check()?,
        Commands::Clean => clean()?,
    }
    Ok(())
}

fn sync() -> Result<()> {
    let config = Config::load().context("Failed to load .hermit config")?;
    let mut lockfile = Lockfile::load().context("Failed to load hermit.lock")?;
    let package_manager =
        PackageManager::from_config(&config).context("Failed to create package manager")?;

    println!(
        "{} packages using {}...",
        "Syncing".green(),
        config.manager.blue()
    );

    let pb = indicatif::ProgressBar::new(config.packages.len() as u64);
    pb.set_message("Installing packages...");
    pb.enable_steady_tick(100);

    for (package, version) in config.packages.iter() {
        package_manager
            .install_package(package, version)
            .with_context(|| format!("Failed to install package {}@{}", package, version))?;

        // Update lockfile
        let package_info = super::lockfile::PackageInfo {
            version: version.clone(),
            resolved: format!("https://registry.npmjs.org/{}/-/{}.tgz", package, package),
            hash: "sha512-placeholder".to_string(),
        };
        lockfile.add_package(package, package_info)?;

        pb.inc(1);
    }

    pb.finish_and_clear();
    println!("{} All packages installed successfully", "Done".green());

    lockfile.save().context("Failed to save hermit.lock")?;
    Ok(())
}

fn add_package(package: String, version: String) -> Result<()> {
    let mut config = Config::load().context("Failed to load .hermit config")?;
    config.add_package(&package, &version)?;
    config.save().context("Failed to save .hermit config")?;

    println!(
        "{} {}@{} to .hermit",
        "Added".green(),
        package.blue(),
        version.blue()
    );

    Ok(())
}

fn remove_package(package: String) -> Result<()> {
    let mut config = Config::load().context("Failed to load .hermit config")?;
    config.remove_package(&package)?;
    config.save().context("Failed to save .hermit config")?;

    println!("{} {} from .hermit", "Removed".green(), package.blue());

    Ok(())
}

fn lock() -> Result<()> {
    let config = Config::load().context("Failed to load .hermit config")?;
    let mut lockfile = Lockfile::load().context("Failed to load hermit.lock")?;

    println!("{} hermit.lock...", "Regenerating".green());

    for (package, version) in config.packages.iter() {
        let package_info = super::lockfile::PackageInfo {
            version: version.clone(),
            resolved: format!("https://registry.npmjs.org/{}/-/{}.tgz", package, package),
            hash: "sha512-placeholder".to_string(),
        };
        lockfile.add_package(package, package_info)?;
    }

    lockfile.save().context("Failed to save hermit.lock")?;
    println!("{} hermit.lock", "Regenerated".green());

    Ok(())
}

fn check() -> Result<()> {
    let config = Config::load().context("Failed to load .hermit config")?;
    let lockfile = Lockfile::load().context("Failed to load hermit.lock")?;
    let package_manager =
        PackageManager::from_config(&config).context("Failed to create package manager")?;

    println!("{} package versions...", "Checking".green());

    let mut all_match = true;
    let pb = indicatif::ProgressBar::new(config.packages.len() as u64);
    pb.set_message("Checking packages...");
    pb.enable_steady_tick(100);

    for (package, expected_version) in config.packages.iter() {
        let installed = package_manager.check_installed_version(package, expected_version)?;
        if installed {
            println!(
                "{} {}@{} - ✓",
                "Package".green(),
                package.blue(),
                expected_version.blue()
            );
        } else {
            println!(
                "{} {}@{} - ✗",
                "Package".red(),
                package.blue(),
                expected_version.blue()
            );
            all_match = false;
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    if all_match {
        println!("{} All package versions match", "Success".green());
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some package versions do not match"))
    }
}

fn clean() -> Result<()> {
    println!("{} hermit-managed installs...", "Cleaning".yellow());

    // This is a placeholder - actual cleaning would depend on package manager
    println!(
        "{} Cleanup implementation needed for each package manager",
        "Warning".yellow()
    );
    println!("1. Remove package directories");
    println!("2. Clear caches");
    println!("3. Remove lock files");

    // Remove lock file
    let lockfile_path = PathBuf::from("hermit.lock");
    if lockfile_path.exists() {
        std::fs::remove_file(lockfile_path)?;
        println!("{} hermit.lock", "Removed".green());
    }

    println!("{} Cleanup completed", "Done".green());
    Ok(())
}
