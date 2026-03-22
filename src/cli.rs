use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub headless: bool,

    #[arg(long, hide = true)]
    pub run_command: Option<String>,
}
