use std::path::PathBuf;

use clap::Parser;
use tracing::Level;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, default_value = "INFO")]
    pub log_level: Level,

    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    pub address: String,

    #[arg(short = 'p', long, default_value = "3000")]
    pub port: u16,

    pub directory: Option<PathBuf>,
}
