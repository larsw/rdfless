// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
// 
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use colored::*;
use rio_api::model::{Triple, Term, Literal, Subject, Quad};
use rio_api::parser::{TriplesParser, QuadsParser};
use rio_turtle::{TurtleParser, TurtleError, TriGParser};
use std::collections::HashMap;
use std::io::{BufReader, Read};
use anyhow::Result;
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

// Convert a Triple to an OwnedTriple
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
            graph_str[1..graph_str.len()-1].to_string()
        } else {
            graph_str
        };

        owned_triple.graph = Some(clean_graph);
    }

    owned_triple
}

// Format an owned subject
pub fn format_owned_subject(triple: &OwnedTriple, prefixes: Option<&HashMap<String, String>>, _colors: &config::ColorConfig) -> String {
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
pub fn format_owned_predicate(triple: &OwnedTriple, prefixes: Option<&HashMap<String, String>>, colors: &config::ColorConfig) -> String {
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
pub fn format_owned_object(triple: &OwnedTriple, prefixes: Option<&HashMap<String, String>>, colors: &config::ColorConfig) -> String {
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

// Print triples with or without prefixes
pub fn print_triples(triples: &[OwnedTriple], prefixes: Option<&HashMap<String, String>>, colors: &config::ColorConfig) {
    // Group triples by graph
    let mut graph_groups: HashMap<Option<String>, Vec<&OwnedTriple>> = HashMap::new();

    for triple in triples {
        graph_groups.entry(triple.graph.clone()).or_default().push(triple);
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
            // Try to use a prefix if available
            let formatted_graph = if let Some(prefixes) = prefixes {
                let mut result = format!("<{}>", graph_name);
                for (prefix, iri) in prefixes {
                    if graph_name.starts_with(iri) {
                        let local_part = &graph_name[iri.len()..];
                        result = format!("{}:{}", prefix, local_part);
                        break;
                    }
                }
                result
            } else {
                format!("<{}>", graph_name)
            };

            println!("{} {{", formatted_graph.color(colors.get_color("graph")).bold());
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
                        println!();  // Add a blank line between statements
                    }
                    println!("{}{}", indent, subject.color(colors.get_color("subject")).bold());
                    println!("{}    {} ;", indent, predicate);
                    println!("{}        {} .", indent, object);
                    current_subject = Some(subject);
                }
            } else {
                // First subject
                println!("{}{}", indent, subject.color(colors.get_color("subject")).bold());
                println!("{}    {} ;", indent, predicate);
                println!("{}        {} .", indent, object);
                current_subject = Some(subject);
            }
        }

        // Close the graph block if it's a named graph
        if graph_key.is_some() {
            println!("}}");
            println!();  // Add a blank line after each graph
        }
    }
}

// Process input based on format
pub fn process_input<R: Read, A: Args>(reader: BufReader<R>, args: &A, colors: &config::ColorConfig, config: &config::Config) -> Result<()> {
    // Determine the format to use
    let format = args.format().unwrap_or(InputFormat::Turtle);

    match format {
        InputFormat::Turtle => process_turtle(reader, args, colors, config),
        InputFormat::TriG => process_trig(reader, args, colors, config),
    }
}

// Process Turtle input
fn process_turtle<R: Read, A: Args>(reader: BufReader<R>, args: &A, colors: &config::ColorConfig, config: &config::Config) -> Result<()> {
    let mut parser = TurtleParser::new(reader, None);

    if !args.expand(config) {
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

// Process TriG input
fn process_trig<R: Read, A: Args>(reader: BufReader<R>, args: &A, colors: &config::ColorConfig, config: &config::Config) -> Result<()> {
    let mut parser = TriGParser::new(reader, None);

    if !args.expand(config) {
        // Collect triples and prefixes
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

        // Process each quad
        let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
            // Convert to owned triple with graph information
            let owned_triple = quad_to_owned(&quad);
            triples.push(owned_triple);
            Ok(())
        };

        // Parse all quads
        parser.parse_all(&mut callback)?;

        // Print triples without prefixes
        print_triples(&triples, None, colors);
    }

    Ok(())
}
