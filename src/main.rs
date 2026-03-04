use clap::Parser;
use log::LevelFilter;

mod cli;
mod config;
mod error;
mod lockfile;
mod manager;

#[derive(Parser)]
#[command(name = "hermit")]
#[command(bin_name = "hermit")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    /// Install all packages across all managers
    Sync {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Add a package to .hermit
    Add {
        package: String,
        version: String,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove a package from .hermit
    Remove {
        package: String,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Regenerate hermit.lock without installing
    Lock {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Verify installed versions match hermit.lock
    Check {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove all hermit-managed installs
    Clean {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let cli = Cli::parse();

    if let Err(e) = cli::run(cli.command) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = vec!["hermit", "sync"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Sync));
    }
}
