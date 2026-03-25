pub mod cli;
pub mod config;
pub mod formatter;

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser};
use std::fs;

use crate::cli::Cli;
use crate::config::resolve_settings;
use crate::formatter::Formatter;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let format_path = match cli.format {
        Some(path) => path,
        None => {
            let mut command = Cli::command();
            command.print_help()?;
            println!();
            return Ok(());
        }
    };

    if !format_path.exists() || format_path.is_dir() {
        bail!("file not found: {}", format_path.display());
    }

    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let loaded_settings = resolve_settings(cli.settings.as_deref(), &cwd)?;

    let input = fs::read_to_string(&format_path)
        .with_context(|| format!("failed to read {}", format_path.display()))?;
    let output = Formatter::new(loaded_settings.settings).format(&input);

    if cli.print {
        print!("{output}");
    } else {
        fs::write(&format_path, output)
            .with_context(|| format!("failed to write {}", format_path.display()))?;
    }

    Ok(())
}
