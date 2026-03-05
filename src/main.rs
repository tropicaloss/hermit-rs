use clap::Parser;
use log::LevelFilter;

mod cargo;
mod cli;
mod config;
mod error;
mod lockfile;
mod manager;

use cli::Commands;

#[derive(Parser)]
#[command(name = "hermit")]
#[command(bin_name = "hermit")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        assert!(matches!(cli.command, Commands::Sync { verbose: _ }));
    }
}
