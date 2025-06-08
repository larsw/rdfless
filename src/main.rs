// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
// 
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use rio_api::model::{Triple, Term, Literal, Subject};
use rio_api::parser::TriplesParser;
use rio_turtle::{TurtleParser, TurtleError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, IsTerminal, Read};
use std::path::PathBuf;

mod config;
use config::{ColorConfig, load_config};

/// A TTL (Turtle) pretty printer with ANSI colors
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input files (TTL format)
    #[arg(name = "FILE")]
    files: Vec<PathBuf>,

    /// Expand prefixes instead of showing PREFIX declarations
    #[arg(long)]
    expand: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load color configuration
    let colors = load_config()?;

    // Check if we should read from stdin or files
    if args.files.is_empty() {
        // Read from stdin if no files are provided and stdin is not a terminal
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);
            process_input(reader, &args, &colors)?;
        } else {
            eprintln!("No input files provided and no input piped to stdin.");
            eprintln!("Usage: cat file.ttl | rdfless [--expand]");
            eprintln!("   or: rdfless [--expand] file1.ttl [file2.ttl ...]");
            eprintln!();
            eprintln!("Use --expand to expand prefixes instead of showing PREFIX declarations.");
            std::process::exit(1);
        }
    } else {
        // Process each file
        for file_path in &args.files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let reader = BufReader::new(file);
            process_input(reader, &args, &colors)?;
        }
    }

    Ok(())
}

// A struct to store triple data with owned strings
#[derive(Debug)]
struct OwnedTriple {
    subject_type: SubjectType,
    subject_value: String,
    predicate: String,
    object_type: ObjectType,
    object_value: String,
    object_datatype: Option<String>,
    object_language: Option<String>,
}

#[derive(Debug, PartialEq)]
enum SubjectType {
    NamedNode,
    BlankNode,
}

#[derive(Debug, PartialEq)]
enum ObjectType {
    NamedNode,
    BlankNode,
    Literal,
}

fn process_input<R: Read>(reader: BufReader<R>, args: &Args, colors: &ColorConfig) -> Result<()> {
    let mut parser = TurtleParser::new(reader, None);

    if !args.expand {
        // Collect triples and prefixes
        let mut triples = Vec::new();
        let mut prefixes = HashMap::new();

        // Process each triple
        let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
            // Convert to owned triple
            let owned_triple = triple_to_owned(&triple);
            triples.push(owned_triple);
            Ok(())
        };

        // Parse all triples
        parser.parse_all(&mut callback)?;

        // Get prefixes from parser
        for (prefix, iri) in parser.prefixes() {
            prefixes.insert(prefix.to_string(), iri.to_string());
        }

        // Print prefixes
        for (prefix, iri) in &prefixes {
            println!("{} {}: <{}> .", 
                "PREFIX".color(colors.get_color("prefix")),
                prefix.color(colors.get_color("prefix")),
                iri);
        }

        if !prefixes.is_empty() {
            println!(); // Add a blank line after prefixes
        }

        // Print triples with prefixes
        print_triples(&triples, Some(&prefixes), colors);
    } else {
        // If expanding, print directly as we parse
        let mut triples = Vec::new();

        // Process each triple
        let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
            // Convert to owned triple
            let owned_triple = triple_to_owned(&triple);
            triples.push(owned_triple);
            Ok(())
        };

        // Parse all triples
        parser.parse_all(&mut callback)?;

        // Print triples without prefixes
        print_triples(&triples, None, colors);
    }

    Ok(())
}

// Convert a Triple to an OwnedTriple
fn triple_to_owned(triple: &Triple) -> OwnedTriple {
    let (subject_type, subject_value) = match &triple.subject {
        Subject::NamedNode(node) => (SubjectType::NamedNode, node.iri.to_string()),
        Subject::BlankNode(node) => (SubjectType::BlankNode, node.id.to_string()),
        Subject::Triple(_) => (SubjectType::NamedNode, "".to_string()), // Not handling nested triples for simplicity
    };

    let predicate = triple.predicate.iri.to_string();

    let (object_type, object_value, object_datatype, object_language) = match &triple.object {
        Term::NamedNode(node) => (ObjectType::NamedNode, node.iri.to_string(), None, None),
        Term::BlankNode(node) => (ObjectType::BlankNode, node.id.to_string(), None, None),
        Term::Literal(literal) => match literal {
            Literal::Simple { value } => (ObjectType::Literal, value.to_string(), None, None),
            Literal::LanguageTaggedString { value, language } => (
                ObjectType::Literal,
                value.to_string(),
                None,
                Some(language.to_string()),
            ),
            Literal::Typed { value, datatype } => (
                ObjectType::Literal,
                value.to_string(),
                Some(datatype.iri.to_string()),
                None,
            ),
        },
        Term::Triple(_) => (ObjectType::NamedNode, "".to_string(), None, None), // Not handling nested triples for simplicity
    };

    OwnedTriple {
        subject_type,
        subject_value,
        predicate,
        object_type,
        object_value,
        object_datatype,
        object_language,
    }
}

// Print triples with or without prefixes
fn print_triples(triples: &[OwnedTriple], prefixes: Option<&HashMap<String, String>>, colors: &ColorConfig) {
    let mut current_subject: Option<String> = None;

    for triple in triples {
        let subject = format_owned_subject(triple, prefixes, colors);
        let predicate = format_owned_predicate(triple, prefixes, colors);
        let object = format_owned_object(triple, prefixes, colors);

        // Check if we're continuing with the same subject
        if let Some(ref current) = current_subject {
            if *current == subject {
                // Same subject, print with semicolon
                println!("    {} ;", predicate);
                println!("        {} .", object);
            } else {
                // New subject
                if current_subject.is_some() {
                    println!();  // Add a blank line between statements
                }
                println!("{}", subject.color(colors.get_color("subject")).bold());
                println!("    {} ;", predicate);
                println!("        {} .", object);
                current_subject = Some(subject);
            }
        } else {
            // First subject
            println!("{}", subject.color(colors.get_color("subject")).bold());
            println!("    {} ;", predicate);
            println!("        {} .", object);
            current_subject = Some(subject);
        }
    }
}

// Format an owned subject
fn format_owned_subject(triple: &OwnedTriple, prefixes: Option<&HashMap<String, String>>, _colors: &ColorConfig) -> String {
    match triple.subject_type {
        SubjectType::NamedNode => {
            if let Some(prefixes) = prefixes {
                // Try to use a prefix if available
                for (prefix, iri) in prefixes {
                    if triple.subject_value.starts_with(iri) {
                        let local_part = &triple.subject_value[iri.len()..];
                        return format!("{}:{}", prefix, local_part);
                    }
                }
            }

            // No prefix found, use full URI
            format!("<{}>", triple.subject_value)
        },
        SubjectType::BlankNode => format!("_:{}", triple.subject_value),
    }
}

// Format an owned predicate
fn format_owned_predicate(triple: &OwnedTriple, prefixes: Option<&HashMap<String, String>>, colors: &ColorConfig) -> String {
    if let Some(prefixes) = prefixes {
        // Try to use a prefix if available
        for (prefix, iri) in prefixes {
            if triple.predicate.starts_with(iri) {
                let local_part = &triple.predicate[iri.len()..];
                return format!("{}:{}", prefix, local_part).color(colors.get_color("predicate")).to_string();
            }
        }
    }

    // No prefix found, use full URI
    format!("<{}>", triple.predicate).color(colors.get_color("predicate")).to_string()
}

// Format an owned object
fn format_owned_object(triple: &OwnedTriple, prefixes: Option<&HashMap<String, String>>, colors: &ColorConfig) -> String {
    match triple.object_type {
        ObjectType::NamedNode => {
            if let Some(prefixes) = prefixes {
                // Try to use a prefix if available
                for (prefix, iri) in prefixes {
                    if triple.object_value.starts_with(iri) {
                        let local_part = &triple.object_value[iri.len()..];
                        return format!("{}:{}", prefix, local_part).color(colors.get_color("object")).to_string();
                    }
                }
            }

            // No prefix found, use full URI
            format!("<{}>", triple.object_value).color(colors.get_color("object")).to_string()
        },
        ObjectType::BlankNode => format!("_:{}", triple.object_value).color(colors.get_color("object")).to_string(),
        ObjectType::Literal => {
            let literal_color = colors.get_color("literal");

            if let Some(language) = &triple.object_language {
                format!("\"{}\"@{}", triple.object_value, language).color(literal_color).to_string()
            } else if let Some(datatype) = &triple.object_datatype {
                let datatype_str = if let Some(prefixes) = prefixes {
                    // Try to use a prefix if available
                    let mut result = format!("<{}>", datatype);
                    for (prefix, iri) in prefixes {
                        if datatype.starts_with(iri) {
                            let local_part = &datatype[iri.len()..];
                            result = format!("{}:{}", prefix, local_part);
                            break;
                        }
                    }
                    result
                } else {
                    format!("<{}>", datatype)
                };

                format!("\"{}\"^^{}", triple.object_value, datatype_str).color(literal_color).to_string()
            } else {
                format!("\"{}\"", triple.object_value).color(literal_color).to_string()
            }
        },
    }
}
