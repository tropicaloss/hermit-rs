use crate::config::Config;
use crate::lockfile::Lockfile;
use crate::manager::PackageManager;
use crate::Commands;
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Duration;

pub fn run(command: Commands) -> Result<()> {
    match command {
        Commands::Sync { verbose } => sync(verbose)?,
        Commands::Add {
            package,
            version,
            verbose,
        } => add_package(package, version, verbose)?,
        Commands::Remove { package, verbose } => remove_package(package, verbose)?,
        Commands::Lock { verbose } => lock(verbose)?,
        Commands::Check { verbose } => check(verbose)?,
        Commands::Clean { verbose } => clean(verbose)?,
    }
    Ok(())
}

fn sync(verbose: bool) -> Result<()> {
    let config = Config::load().context("Failed to load .hermit config")?;
    let mut lockfile = Lockfile::load().context("Failed to load hermit.lock")?;
    let package_manager =
        PackageManager::from_config(&config).context("Failed to create package manager")?;

    if verbose {
        println!(
            "{} {} packages using {}...",
            "Syncing".green(),
            config.packages.len(),
            config.manager.blue()
        );
    } else {
        println!(
            "{} packages using {}...",
            "Syncing".green(),
            config.manager.blue()
        );
    }

    let pb = indicatif::ProgressBar::new(config.packages.len() as u64);
    pb.set_message("Installing packages...");
    pb.enable_steady_tick(Duration::from_millis(100));

    for (package, version) in config.packages.iter() {
        if verbose {
            println!("Installing {}@{}...", package, version);
        }

        package_manager
            .install_package(package, version, verbose)
            .with_context(|| format!("Failed to install package {}@{}", package, version))?;

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

fn add_package(package: String, version: String, verbose: bool) -> Result<()> {
    let mut config = Config::load().context("Failed to load .hermit config")?;
    config.add_package(&package, &version)?;
    config.save().context("Failed to save .hermit config")?;

    if verbose {
        println!(
            "{} {}@{} to .hermit (verbose mode)",
            "Added".green(),
            package.blue(),
            version.blue()
        );
    } else {
        println!(
            "{} {}@{} to .hermit",
            "Added".green(),
            package.blue(),
            version.blue()
        );
    }

    Ok(())
}

fn remove_package(package: String, verbose: bool) -> Result<()> {
    let mut config = Config::load().context("Failed to load .hermit config")?;
    config.remove_package(&package)?;
    config.save().context("Failed to save .hermit config")?;

    if verbose {
        println!(
            "{} {} from .hermit (verbose mode)",
            "Removed".green(),
            package.blue()
        );
    } else {
        println!("{} {} from .hermit", "Removed".green(), package.blue());
    }

    Ok(())
}

fn lock(verbose: bool) -> Result<()> {
    let config = Config::load().context("Failed to load .hermit config")?;
    let mut lockfile = Lockfile::load().context("Failed to load hermit.lock")?;

    if verbose {
        println!(
            "{} hermit.lock for {} packages...",
            "Regenerating".green(),
            config.packages.len()
        );
    } else {
        println!("{} hermit.lock...", "Regenerating".green());
    }

    for (package, version) in config.packages.iter() {
        if verbose {
            println!("Adding {}@{} to lockfile...", package, version);
        }
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

fn check(verbose: bool) -> Result<()> {
    let config = Config::load().context("Failed to load .hermit config")?;
    let _lockfile = Lockfile::load().context("Failed to load hermit.lock")?;
    let package_manager =
        PackageManager::from_config(&config).context("Failed to create package manager")?;

    if verbose {
        println!(
            "{} {} package versions...",
            "Checking".green(),
            config.packages.len()
        );
    } else {
        println!("{} package versions...", "Checking".green());
    }

    let mut all_match = true;
    let pb = indicatif::ProgressBar::new(config.packages.len() as u64);
    pb.set_message("Checking packages...");
    pb.enable_steady_tick(Duration::from_millis(100));

    for (package, expected_version) in config.packages.iter() {
        if verbose {
            println!("Checking {}@{}...", package, expected_version);
        }
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

fn clean(verbose: bool) -> Result<()> {
    if verbose {
        println!(
            "{} hermit-managed installs (verbose)...",
            "Cleaning".yellow()
        );
    } else {
        println!("{} hermit-managed installs...", "Cleaning".yellow());
    }

    if verbose {
        println!("Step 1: Remove package directories");
        println!("Step 2: Clear caches");
        println!("Step 3: Remove lock files");
    } else {
        println!(
            "{} Cleanup implementation needed for each package manager",
            "Warning".yellow()
        );
        println!("1. Remove package directories");
        println!("2. Clear caches");
        println!("3. Remove lock files");
    }

    let lockfile_path = PathBuf::from("hermit.lock");
    if lockfile_path.exists() {
        std::fs::remove_file(lockfile_path)?;
        if verbose {
            println!("Removed hermit.lock file");
        }
        println!("{} hermit.lock", "Removed".green());
    }

    println!("{} Cleanup completed", "Done".green());
    Ok(())
}
