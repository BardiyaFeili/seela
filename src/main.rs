use clap::Parser;
use std::error::Error;

mod cli;
mod config;
mod fzf;
mod run;
mod tmux;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    if let Some(cmd) = args.run_command {
        return run::run_confirm(&cmd);
    }

    let config_path = config::get_config_path(args.config);

    if let Some(path) = config_path {
        let config_dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        match config::Config::load(path) {
            Ok(cfg) => run::run(&cfg, &config_dir, args.debug, args.headless)?,
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
