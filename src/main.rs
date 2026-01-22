// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.
//
// Main entry point for the rdfless CLI application

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};
use rdfless::{
    detect_format_from_path, get_effective_colors, load_config, ArgsConfig, ColorConfig, Config,
    InputFormat,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, IsTerminal};
use std::path::PathBuf;

/// Define the format options for the command-line
#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    Turtle,
    Trig,
    #[value(name = "ntriples")]
    NTriples,
    #[value(name = "nquads")]
    NQuads,
    #[value(name = "provn")]
    ProvN,
}

/// A pretty printer for RDF data with ANSI colors
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input files (Turtle, TriG, N-Triples, N-Quads, or PROV-N format)
    #[arg(name = "FILE")]
    files: Vec<PathBuf>,

    /// Expand prefixes instead of showing PREFIX declarations
    #[arg(long)]
    expand: bool,

    /// Compact mode (opposite of 'expand')
    #[arg(long)]
    compact: bool,

    /// Override the input format (auto-detected from file extension by default)
    /// Supported formats: turtle, trig, ntriples, nquads, provn
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

    /// Continue parsing even when encountering errors (skip invalid triples)
    #[arg(long)]
    continue_on_error: bool,

    /// Filter by subject (IRI or prefixed name)
    #[arg(long, short = 's')]
    subject: Option<String>,

    /// Filter by predicate (IRI or prefixed name)  
    #[arg(long, short = 'p')]
    predicate: Option<String>,

    /// Filter by object (IRI, prefixed name, or literal value)
    #[arg(long, short = 'o')]
    object: Option<String>,

    /// Output file (write to file instead of stdout)
    #[arg(short = 'O', long)]
    output: Option<PathBuf>,

    /// Generate shell completion script for bash, zsh, fish, elvish, or powershell
    #[arg(long, value_enum)]
    completion: Option<Shell>,

    /// Print pager keybindings and exit
    #[arg(long, short = 'k')]
    keybindings: bool,
}

impl rdfless::ArgsConfig for Args {
    fn expand(&self, config: &Config) -> bool {
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

    fn use_pager(&self, config: &Config) -> bool {
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

    fn get_colors(&self, config: &Config) -> ColorConfig {
        // If outputting to a file, disable colors unless explicitly forced with theme flags
        if self.is_output_to_file() && !self.dark_theme && !self.light_theme {
            // Return a "no color" configuration
            return ColorConfig::no_color();
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
            get_effective_colors(config)
        }
    }

    fn is_output_to_file(&self) -> bool {
        self.output.is_some()
    }

    fn continue_on_error(&self) -> bool {
        self.continue_on_error
    }

    fn filter_subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    fn filter_predicate(&self) -> Option<&str> {
        self.predicate.as_deref()
    }

    fn filter_object(&self) -> Option<&str> {
        self.object.as_deref()
    }

    fn format(&self) -> Option<InputFormat> {
        // If 'format' is explicitly specified, use it
        if let Some(format_arg) = self.format {
            return Some(match format_arg {
                FormatArg::Turtle => InputFormat::Turtle,
                FormatArg::Trig => InputFormat::TriG,
                FormatArg::NTriples => InputFormat::NTriples,
                FormatArg::NQuads => InputFormat::NQuads,
                FormatArg::ProvN => InputFormat::ProvN,
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

/// Print the pager keybindings help
fn print_keybindings() {
    println!("rdfless pager keybindings");
    println!();
    println!("When paging is enabled, rdfless uses the minus pager which provides");
    println!("powerful navigation and search capabilities. Here are the key bindings:");
    println!();
    println!("Navigation:");
    println!("  Ctrl+C/q            Quit the pager");
    println!("  [n] Arrow Up/k      Scroll up by n line(s) (default: 1)");
    println!("  [n] Arrow Down/j    Scroll down by n line(s) (default: 1)");
    println!("  [n] Arrow Left/h    Scroll left by n line(s) (default: 1)");
    println!("  [n] Arrow Right/l   Scroll right by n line(s) (default: 1)");
    println!("  Page Up             Scroll up by entire page");
    println!("  Page Down           Scroll down by entire page");
    println!("  [n] Enter           Scroll down by n line(s)");
    println!("  Space               Scroll down by one page");
    println!("  Ctrl+U/u            Scroll up by half a screen");
    println!("  Ctrl+D/d            Scroll down by half a screen");
    println!("  g                   Go to the very top of the output");
    println!("  [n] G               Go to the very bottom (or line n if specified)");
    println!();
    println!("Display:");
    println!("  Ctrl+h              Turn off line wrapping and allow horizontal scrolling");
    println!("  Ctrl+L              Toggle line numbers if not forced enabled/disabled");
    println!("  Ctrl+f              Toggle follow-mode");
    println!();
    println!("Search:");
    println!("  /                   Start forward search");
    println!("  ?                   Start backward search");
    println!("  Esc                 Cancel search input");
    println!("  n                   Go to the next search match");
    println!("  p                   Go to the previous search match");
    println!();
    println!("Mouse:");
    println!("  Mouse scroll Up     Scroll up by 5 lines");
    println!("  Mouse scroll Down   Scroll down by 5 lines");
    println!();
    println!("Search Prompt:");
    println!("  Enter               Confirm the search query");
    println!("  Backspace           Remove the character before the cursor");
    println!("  Delete              Remove the character under the cursor");
    println!("  Arrow Left/Right    Move cursor within search query");
    println!("  Ctrl+Arrow L/R      Move cursor word by word");
    println!("  Home/End            Move cursor to beginning/end of search query");
    println!();
    println!(
        "Note: [n] means you can precede the key with an integer to repeat the action n times."
    );
    println!();
    println!("For more detailed information, see PAGER_KEYBINDINGS.md in the repository.");
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle shell completion generation
    if let Some(shell) = args.completion {
        let mut cmd = Args::command();
        generate(shell, &mut cmd, "rdfless", &mut io::stdout());
        return Ok(());
    }

    // Handle keybindings display
    if args.keybindings {
        print_keybindings();
        return Ok(());
    }

    // Load configuration
    let config = load_config()?;

    // Helper function to parse input with robust error handling
    fn parse_input_generic<R: std::io::Read>(
        reader: BufReader<R>,
        format: rdfless::InputFormat,
        continue_on_error: bool,
    ) -> Result<(Vec<rdfless::OwnedTriple>, HashMap<String, String>)> {
        if continue_on_error {
            let parse_result = rdfless::parse_robust(reader, format, true)?;

            // Report errors to stderr if any
            if parse_result.has_errors() {
                eprintln!(
                    "Warning: {} parse errors encountered:",
                    parse_result.error_count()
                );
                for error in &parse_result.errors {
                    eprintln!("  Line {}: {}", error.line, error.message);
                }
                eprintln!(
                    "Successfully parsed {} triples",
                    parse_result.triple_count()
                );
            }

            Ok((parse_result.triples, parse_result.prefixes))
        } else {
            // Use standard parsing
            rdfless::parse_for_estimation(reader, format)
        }
    }

    // Helper function to process and output data
    let process_and_output =
        |triples: &[rdfless::OwnedTriple], prefixes: &HashMap<String, String>| -> Result<()> {
            let colors = &args.get_colors(&config);
            let should_expand = args.expand(&config);

            // Apply filtering if any filters are specified
            let filter = rdfless::TripleFilter::new(
                args.filter_subject(),
                args.filter_predicate(),
                args.filter_object(),
            );

            let filtered_triples = filter.filter_triples(triples, prefixes);
            let triples_to_process = &filtered_triples;

            if let Some(output_path) = &args.output {
                // Write to file
                let mut file = File::create(output_path).with_context(|| {
                    format!("Failed to create output file: {}", output_path.display())
                })?;
                rdfless::render_output(
                    triples_to_process,
                    prefixes,
                    should_expand,
                    colors,
                    &mut file,
                )?;
            } else {
                // Write to stdout (with potential paging)
                let estimated_lines =
                    rdfless::estimate_output_lines(triples_to_process, prefixes, should_expand);
                let use_paging = rdfless::should_use_pager(&args, &config, estimated_lines);

                if use_paging && io::stdout().is_terminal() {
                    // Use pager
                    let mut output = Vec::new();
                    rdfless::render_output(
                        triples_to_process,
                        prefixes,
                        should_expand,
                        colors,
                        &mut output,
                    )?;
                    let output_str = String::from_utf8(output)?;

                    let pager = minus::Pager::new();
                    pager.set_text(output_str)?;
                    minus::page_all(pager)?;
                } else {
                    // Direct output to stdout
                    rdfless::render_output(
                        triples_to_process,
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
            let (triples, prefixes) =
                parse_input_generic(reader, format, args.continue_on_error())?;

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
            let (mut triples, prefixes) =
                parse_input_generic(reader, format, args.continue_on_error())?;

            all_triples.append(&mut triples);
            for (prefix, iri) in prefixes {
                all_prefixes.insert(prefix, iri);
            }
        }

        process_and_output(&all_triples, &all_prefixes)?;
    }

    Ok(())
}
