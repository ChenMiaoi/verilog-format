use clap::Parser;
use std::path::PathBuf;

use crate::config::DEFAULT_CONFIG_FILE_NAME;

#[derive(Debug, Parser)]
#[command(
    name = "verilog-format",
    about = "Apply formatting to a Verilog file.",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'f', long = "format", value_name = "pathname")]
    pub format: Option<PathBuf>,

    #[arg(short = 'p', long = "print")]
    pub print: bool,

    #[arg(short = 's', long = "settings", value_name = DEFAULT_CONFIG_FILE_NAME)]
    pub settings: Option<PathBuf>,

    #[arg(short = 'v', long = "version")]
    pub version: bool,
}
