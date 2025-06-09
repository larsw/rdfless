// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, ValueEnum};
use std::fs::File;
use std::io::{self, BufReader, IsTerminal};
use std::path::PathBuf;

use rdfless::{config::load_config, detect_format_from_path, InputFormat};

/// Define the format options for the command-line
#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    Turtle,
    Trig,
}

/// A pretty printer for RDF data with ANSI colors
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input files (Turtle or TriG format)
    #[arg(name = "FILE")]
    files: Vec<PathBuf>,

    /// Expand prefixes instead of showing PREFIX declarations
    #[arg(long)]
    expand: bool,

    /// Compact mode (opposite of 'expand')
    #[arg(long)]
    compact: bool,

    /// Override the input format (auto-detected from file extension by default)
    #[arg(long, value_enum)]
    format: Option<FormatArg>,
}

impl rdfless::Args for Args {
    fn expand(&self, config: &rdfless::config::Config) -> bool {
        // If both flags are provided, compact takes precedence
        if self.compact {
            false
        } else if self.expand {
            true
        } else {
            // If neither flag is provided, use config value
            config.output.expand
        }
    }

    fn format(&self) -> Option<InputFormat> {
        // If 'format' is explicitly specified, use it
        if let Some(format_arg) = self.format {
            return Some(match format_arg {
                FormatArg::Turtle => InputFormat::Turtle,
                FormatArg::Trig => InputFormat::TriG,
            });
        }

        // Otherwise, try to detect from the first file's extension
        if !self.files.is_empty() {
            return detect_format_from_path(&self.files[0]);
        }

        // Default to Turtle if no files or format specified
        Some(InputFormat::Turtle)
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = load_config()?;
    let colors = &config.colors;

    // Check if we should read from stdin or files
    if args.files.is_empty() {
        // Read from stdin if no files are provided and stdin is not a terminal
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);
            rdfless::process_input(reader, &args, colors, &config)?;
        } else {
            eprintln!("No input files provided and no input piped to stdin.");
            Args::command().print_help().expect("Failed to print help");
            std::process::exit(1);
        }
    } else {
        // Process each file
        for file_path in &args.files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let reader = BufReader::new(file);
            rdfless::process_input(reader, &args, colors, &config)?;
        }
    }

    Ok(())
}
