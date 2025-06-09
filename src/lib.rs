// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::Result;
use colored::*;
use rio_api::model::{Literal, Quad, Subject, Term, Triple};
use rio_api::parser::{QuadsParser, TriplesParser};
use rio_turtle::{TriGParser, TurtleError, TurtleParser};
// Sophia imports will be used in the future implementation
// Currently keeping the dependencies for future migration
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::Path;

pub mod config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFormat {
    Turtle,
    TriG,
}

// Define a trait for the Args interface
pub trait Args {
    // Determine if prefixes should be expanded based on args and config
    fn expand(&self, config: &config::Config) -> bool;

    // Get the input format (either specified by user or detected from file extension)
    fn format(&self) -> Option<InputFormat>;
}

// Helper function to detect format from file extension
pub fn detect_format_from_path(path: &Path) -> Option<InputFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "ttl" => InputFormat::Turtle,
            "trig" => InputFormat::TriG,
            _ => InputFormat::Turtle, // Default to Turtle for unknown extensions
        })
}

// Re-export the types and functions needed for testing
#[derive(Debug)]
pub struct OwnedTriple {
    pub subject_type: SubjectType,
    pub subject_value: String,
    pub predicate: String,
    pub object_type: ObjectType,
    pub object_value: String,
    pub object_datatype: Option<String>,
    pub object_language: Option<String>,
    pub graph: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum SubjectType {
    NamedNode,
    BlankNode,
}

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    NamedNode,
    BlankNode,
    Literal,
}

// Convert a Triple to an OwnedTriple (rio version)
pub fn triple_to_owned(triple: &Triple) -> OwnedTriple {
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
        graph: None,
    }
}

// TODO: Implement sophia_triple_to_owned function
// This will be implemented later when we migrate to sophia

// Convert a Quad to an OwnedTriple with graph information
pub fn quad_to_owned(quad: &Quad) -> OwnedTriple {
    // First convert the triple part
    let mut owned_triple = triple_to_owned(&Triple {
        subject: quad.subject,
        predicate: quad.predicate,
        object: quad.object,
    });

    // Then add the graph information if available
    if let Some(graph_name) = &quad.graph_name {
        // Extract the graph name without angle brackets
        // The format!("{}", graph_name) might include angle brackets, so we'll extract just the IRI
        let graph_str = format!("{}", graph_name);

        // Remove angle brackets if present
        let clean_graph = if graph_str.starts_with('<') && graph_str.ends_with('>') {
            graph_str[1..graph_str.len() - 1].to_string()
        } else {
            graph_str
        };

        owned_triple.graph = Some(clean_graph);
    }

    owned_triple
}

// Helper function to resolve a URI using prefixes
fn resolve_uri_with_prefixes(uri: &str, prefixes: Option<&HashMap<String, String>>) -> String {
    if let Some(prefixes) = prefixes {
        // Try to use a prefix if available
        for (prefix, iri) in prefixes {
            if uri.starts_with(iri) {
                let local_part = &uri[iri.len()..];
                return format!("{}:{}", prefix, local_part);
            }
        }
    }

    // No prefix found, use full URI
    format!("<{}>", uri)
}

// Format an owned subject
pub fn format_owned_subject(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    _colors: &config::ColorConfig,
) -> String {
    match triple.subject_type {
        SubjectType::NamedNode => resolve_uri_with_prefixes(&triple.subject_value, prefixes),
        SubjectType::BlankNode => format!("_:{}", triple.subject_value),
    }
}

// Format an owned predicate
pub fn format_owned_predicate(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) -> String {
    resolve_uri_with_prefixes(&triple.predicate, prefixes)
        .color(colors.get_color("predicate"))
        .to_string()
}

// Format an owned object
pub fn format_owned_object(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) -> String {
    match triple.object_type {
        ObjectType::NamedNode => resolve_uri_with_prefixes(&triple.object_value, prefixes)
            .color(colors.get_color("object"))
            .to_string(),
        ObjectType::BlankNode => format!("_:{}", triple.object_value)
            .color(colors.get_color("object"))
            .to_string(),
        ObjectType::Literal => {
            let literal_color = colors.get_color("literal");

            if let Some(language) = &triple.object_language {
                format!("\"{}\"@{}", triple.object_value, language)
                    .color(literal_color)
                    .to_string()
            } else if let Some(datatype) = &triple.object_datatype {
                // In compact mode (prefixes is Some), don't expand basic data types
                // In expanded mode (prefixes is None), always expand data types
                let is_compact_mode = prefixes.is_some();
                let is_basic_datatype = match datatype.as_str() {
                    "http://www.w3.org/2001/XMLSchema#integer" |
                    "http://www.w3.org/2001/XMLSchema#string" |
                    "http://www.w3.org/2001/XMLSchema#boolean" |
                    "http://www.w3.org/2001/XMLSchema#decimal" |
                    "http://www.w3.org/2001/XMLSchema#float" |
                    "http://www.w3.org/2001/XMLSchema#double" |
                    "http://www.w3.org/2001/XMLSchema#date" |
                    "http://www.w3.org/2001/XMLSchema#time" |
                    "http://www.w3.org/2001/XMLSchema#dateTime" => true,
                    _ => false,
                };

                if is_compact_mode && is_basic_datatype {
                    // In compact mode, don't expand basic data types
                    // Handle different literal types appropriately
                    match datatype.as_str() {
                        "http://www.w3.org/2001/XMLSchema#integer" |
                        "http://www.w3.org/2001/XMLSchema#decimal" |
                        "http://www.w3.org/2001/XMLSchema#float" |
                        "http://www.w3.org/2001/XMLSchema#double" => {
                            // Output numeric types without quotes
                            format!("{}", triple.object_value)
                                .color(literal_color)
                                .to_string()
                        }
                        "http://www.w3.org/2001/XMLSchema#boolean" => {
                            // Output boolean values without quotes
                            format!("{}", triple.object_value)
                                .color(literal_color)
                                .to_string()
                        }
                        _ => {
                            // Keep other types (like strings, dates, etc.) in quotes
                            format!("\"{}\"", triple.object_value)
                                .color(literal_color)
                                .to_string()
                        }
                    }
                } else {
                    // In expanded mode or for non-basic data types, show the full datatype
                    let datatype_str = resolve_uri_with_prefixes(datatype, prefixes);
                    format!("\"{}\"^^{}", triple.object_value, datatype_str)
                        .color(literal_color)
                        .to_string()
                }
            } else {
                format!("\"{}\"", triple.object_value)
                    .color(literal_color)
                    .to_string()
            }
        }
    }
}

// Print triples with or without prefixes
pub fn print_triples(
    triples: &[OwnedTriple],
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) {
    // Group triples by graph
    let mut graph_groups: HashMap<Option<String>, Vec<&OwnedTriple>> = HashMap::new();

    for triple in triples {
        graph_groups
            .entry(triple.graph.clone())
            .or_default()
            .push(triple);
    }

    // Sort graphs to ensure consistent output (None/default graph first)
    let mut graph_keys: Vec<_> = graph_groups.keys().collect();
    graph_keys.sort_by(|a, b| match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, _) => std::cmp::Ordering::Less,
        (_, None) => std::cmp::Ordering::Greater,
        (Some(a_str), Some(b_str)) => a_str.cmp(b_str),
    });

    // Process each graph
    for graph_key in graph_keys {
        let triples_in_graph = &graph_groups[graph_key];

        // Print graph name if it exists (for TriG format)
        if let Some(graph_name) = graph_key {
            let formatted_graph = resolve_uri_with_prefixes(graph_name, prefixes);

            println!(
                "{} {{",
                formatted_graph.color(colors.get_color("graph")).bold()
            );
        }

        // Group by subject within this graph
        let mut current_subject: Option<String> = None;

        for triple in triples_in_graph {
            let subject = format_owned_subject(triple, prefixes, colors);
            let predicate = format_owned_predicate(triple, prefixes, colors);
            let object = format_owned_object(triple, prefixes, colors);

            // Indent more if we're in a named graph
            let indent = if graph_key.is_some() { "  " } else { "" };

            // Check if we're continuing with the same subject
            if let Some(ref current) = current_subject {
                if *current == subject {
                    // Same subject, print with semicolon
                    println!("{}    {} ;", indent, predicate);
                    println!("{}        {} .", indent, object);
                } else {
                    // New subject
                    if current_subject.is_some() {
                        println!(); // Add a blank line between statements
                    }
                    println!(
                        "{}{}",
                        indent,
                        subject.color(colors.get_color("subject")).bold()
                    );
                    println!("{}    {} ;", indent, predicate);
                    println!("{}        {} .", indent, object);
                    current_subject = Some(subject);
                }
            } else {
                // First subject
                println!(
                    "{}{}",
                    indent,
                    subject.color(colors.get_color("subject")).bold()
                );
                println!("{}    {} ;", indent, predicate);
                println!("{}        {} .", indent, object);
                current_subject = Some(subject);
            }
        }

        // Close the graph block if it's a named graph
        if graph_key.is_some() {
            println!("}}");
            println!(); // Add a blank line after each graph
        }
    }
}

// Helper function to print prefixes
fn print_prefixes(prefixes: &HashMap<String, String>, colors: &config::ColorConfig) {
    for (prefix, iri) in prefixes {
        println!(
            "{} {}: <{}> .",
            "PREFIX".color(colors.get_color("prefix")),
            prefix.color(colors.get_color("prefix")),
            iri
        );
    }

    if !prefixes.is_empty() {
        println!(); // Add a blank line after prefixes
    }
}

// Helper function to collect and print triples
fn collect_and_print_triples(
    triples: &mut [OwnedTriple],
    prefixes: &mut HashMap<String, String>,
    parser_prefixes: impl Iterator<Item = (String, String)>,
    should_expand: bool,
    colors: &config::ColorConfig,
) {
    // Get prefixes from parser
    for (prefix, iri) in parser_prefixes {
        prefixes.insert(prefix, iri);
    }

    if !should_expand {
        // Print prefixes
        print_prefixes(prefixes, colors);

        // Print triples with prefixes
        print_triples(triples, Some(prefixes), colors);
    } else {
        // Print triples without prefixes
        print_triples(triples, None, colors);
    }
}

// Process input based on format
pub fn process_input<R: Read, A: Args>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<()> {
    // Determine the format to use
    let format = args.format().unwrap_or(InputFormat::Turtle);

    match format {
        InputFormat::Turtle => process_turtle(reader, args, colors, config),
        InputFormat::TriG => process_trig(reader, args, colors, config),
    }
}

// Process Turtle input (rio version)
fn process_turtle<R: Read, A: Args>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<()> {
    let mut parser = TurtleParser::new(reader, None);
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

    // Collect prefixes and print triples
    collect_and_print_triples(
        &mut triples,
        &mut prefixes,
        parser
            .prefixes()
            .iter()
            .map(|(p, i)| (p.to_string(), i.to_string())),
        args.expand(config),
        colors,
    );

    Ok(())
}

// TODO: Implement process_turtle_sophia function
// This will be implemented later when we migrate to sophia

// Process TriG input
fn process_trig<R: Read, A: Args>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<()> {
    let mut parser = TriGParser::new(reader, None);
    let mut triples = Vec::new();
    let mut prefixes = HashMap::new();

    // Process each quad
    let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
        // Convert to owned triple with graph information
        let owned_triple = quad_to_owned(&quad);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all quads
    parser.parse_all(&mut callback)?;

    // Collect prefixes and print triples
    collect_and_print_triples(
        &mut triples,
        &mut prefixes,
        parser
            .prefixes()
            .iter()
            .map(|(p, i)| (p.to_string(), i.to_string())),
        args.expand(config),
        colors,
    );

    Ok(())
}
