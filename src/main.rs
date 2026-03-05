mod cli;
mod config;
mod run;

use std::error::Error;

use clap::Parser;

use crate::run::run;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let config_path = config::get_config_path(args.config);

    if let Some(path) = config_path {
        match config::Config::load(path) {
            Ok(cfg) => run(cfg)?,
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: No config file found in search paths.");
        std::process::exit(1);
    }

    Ok(())
}
