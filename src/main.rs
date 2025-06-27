// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, ValueEnum};
use rdfless::{config::load_config, detect_format_from_path, ArgsConfig, InputFormat};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, IsTerminal};
use std::path::PathBuf;

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

    /// Enable paging for large outputs
    #[arg(long)]
    pager: bool,

    /// Disable paging (useful when paging is enabled by default in config)
    #[arg(long)]
    no_pager: bool,

    /// Force dark theme colors
    #[arg(long)]
    dark_theme: bool,

    /// Force light theme colors
    #[arg(long)]
    light_theme: bool,

    /// Disable automatic background detection
    #[arg(long)]
    no_auto_theme: bool,

    /// Output file (write to file instead of stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

impl rdfless::ArgsConfig for Args {
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

    fn use_pager(&self, config: &rdfless::config::Config) -> bool {
        // If no_pager is specified, disable paging
        if self.no_pager {
            false
        } else if self.pager {
            true
        } else {
            // If neither flag is provided, use config value
            config.output.pager
        }
    }

    fn no_pager_explicit(&self) -> bool {
        self.no_pager
    }

    fn get_colors(&self, config: &rdfless::config::Config) -> rdfless::config::ColorConfig {
        // If outputting to a file, disable colors unless explicitly forced with theme flags
        if self.is_output_to_file() && !self.dark_theme && !self.light_theme {
            // Return a "no color" configuration
            return rdfless::config::ColorConfig::no_color();
        }

        // Check for explicit theme flags first
        if self.dark_theme {
            config.theme.dark_theme.clone()
        } else if self.light_theme {
            config.theme.light_theme.clone()
        } else if self.no_auto_theme {
            // Use explicitly configured colors without auto-detection
            config.colors.clone()
        } else {
            // Use auto-detection
            rdfless::config::get_effective_colors(config)
        }
    }

    fn is_output_to_file(&self) -> bool {
        self.output.is_some()
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

    // Helper function to process and output data
    let process_and_output = |triples: &[rdfless::OwnedTriple],
                              prefixes: &HashMap<String, String>|
     -> Result<()> {
        let colors = &args.get_colors(&config);
        let should_expand = args.expand(&config);

        if let Some(output_path) = &args.output {
            // Write to file
            let mut file = File::create(output_path).with_context(|| {
                format!("Failed to create output file: {}", output_path.display())
            })?;
            rdfless::render_output(triples, prefixes, should_expand, colors, &mut file)?;
        } else {
            // Write to stdout (with potential paging)
            let estimated_lines = rdfless::estimate_output_lines(triples, prefixes, should_expand);
            let use_paging = rdfless::should_use_pager(&args, &config, estimated_lines);

            if use_paging && io::stdout().is_terminal() {
                // Use pager
                let mut output = Vec::new();
                rdfless::render_output(triples, prefixes, should_expand, colors, &mut output)?;
                let output_str = String::from_utf8(output)?;

                let pager = minus::Pager::new();
                pager.set_text(output_str)?;
                minus::page_all(pager)?;
            } else {
                // Direct output to stdout
                rdfless::render_output(
                    triples,
                    prefixes,
                    should_expand,
                    colors,
                    &mut io::stdout(),
                )?;
            }
        }
        Ok(())
    };

    // Check if we should read from stdin or files
    if args.files.is_empty() {
        // Read from stdin if no files are provided and stdin is not a terminal
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);

            let format = args.format().unwrap_or(rdfless::InputFormat::Turtle);
            let (triples, prefixes) = match format {
                rdfless::InputFormat::Turtle => rdfless::parse_turtle_for_estimation(reader)?,
                rdfless::InputFormat::TriG => rdfless::parse_trig_for_estimation(reader)?,
                rdfless::InputFormat::NTriples => rdfless::parse_ntriples_for_estimation(reader)?,
                rdfless::InputFormat::NQuads => rdfless::parse_nquads_for_estimation(reader)?,
            };

            process_and_output(&triples, &prefixes)?;
        } else {
            eprintln!("No input files provided and no input piped to stdin.");
            Args::command().print_help().expect("Failed to print help");
            std::process::exit(1);
        }
    } else {
        // Process files
        let mut all_triples = Vec::new();
        let mut all_prefixes = HashMap::new();

        for file_path in &args.files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let reader = BufReader::new(file);

            let format = args.format().unwrap_or(rdfless::InputFormat::Turtle);
            let (mut triples, prefixes) = match format {
                rdfless::InputFormat::Turtle => rdfless::parse_turtle_for_estimation(reader)?,
                rdfless::InputFormat::TriG => rdfless::parse_trig_for_estimation(reader)?,
                rdfless::InputFormat::NTriples => rdfless::parse_ntriples_for_estimation(reader)?,
                rdfless::InputFormat::NQuads => rdfless::parse_nquads_for_estimation(reader)?,
            };

            all_triples.append(&mut triples);
            for (prefix, iri) in prefixes {
                all_prefixes.insert(prefix, iri);
            }
        }

        process_and_output(&all_triples, &all_prefixes)?;
    }

    Ok(())
}
