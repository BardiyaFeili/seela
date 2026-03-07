use std::error::Error;
use clap::Parser;
use seela::cli;
use seela::config;
use seela::run;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    if let Some(cmd) = args.run_command {
        return run::run_confirm(&cmd);
    }

    let config_path = config::get_config_path(args.config);

    if let Some(path) = config_path {
        match config::Config::load(path) {
            Ok(cfg) => run::run(&cfg, args.debug, args.headless)?,
            Err(e) => {
                eprintln!("Error loading config: {e}");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: No config file found in the search paths.");
        std::process::exit(1);
    }

    Ok(())
}
