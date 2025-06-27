// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::Result;
use colored::*;
// Rio imports for the implementation
use rio_api::model::{Literal, Quad, Subject, Term, Triple};
use rio_api::parser::{QuadsParser, TriplesParser};
use rio_turtle::{NQuadsParser, NTriplesParser, TriGParser, TurtleError, TurtleParser};
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use terminal_size::{terminal_size, Height, Width};

pub mod config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFormat {
    Turtle,
    TriG,
    NTriples,
    NQuads,
}

// Define a trait for the Args interface
pub trait ArgsConfig {
    // Determine if prefixes should be expanded based on args and config
    fn expand(&self, config: &config::Config) -> bool;

    // Get the input format (either specified by user or detected from file extension)
    fn format(&self) -> Option<InputFormat>;

    // Determine if paging should be used based on args and config (explicit user choice)
    fn use_pager(&self, config: &config::Config) -> bool;

    // Check if user explicitly disabled paging
    fn no_pager_explicit(&self) -> bool;

    // Get the effective color configuration
    fn get_colors(&self, config: &config::Config) -> config::ColorConfig;
}

// Helper function to detect 'format' from file extension
pub fn detect_format_from_path(path: &Path) -> Option<InputFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "ttl" => InputFormat::Turtle,
            "trig" => InputFormat::TriG,
            "nt" => InputFormat::NTriples,
            "nq" => InputFormat::NQuads,
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
        // The format!("{graph_name}") might include angle brackets, so we'll extract just the IRI
        let graph_str = format!("{graph_name}");

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
                return format!("{prefix}:{local_part}");
            }
        }
    }

    // No prefix found, use full URI
    format!("<{uri}>")
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
                let is_basic_datatype = matches!(
                    datatype.as_str(),
                    "http://www.w3.org/2001/XMLSchema#integer"
                        | "http://www.w3.org/2001/XMLSchema#string"
                        | "http://www.w3.org/2001/XMLSchema#boolean"
                        | "http://www.w3.org/2001/XMLSchema#decimal"
                        | "http://www.w3.org/2001/XMLSchema#float"
                        | "http://www.w3.org/2001/XMLSchema#double"
                        | "http://www.w3.org/2001/XMLSchema#date"
                        | "http://www.w3.org/2001/XMLSchema#time"
                        | "http://www.w3.org/2001/XMLSchema#dateTime"
                );

                if is_compact_mode && is_basic_datatype {
                    // In compact mode, don't expand basic data types
                    // Handle different literal types appropriately
                    match datatype.as_str() {
                        "http://www.w3.org/2001/XMLSchema#integer"
                        | "http://www.w3.org/2001/XMLSchema#decimal"
                        | "http://www.w3.org/2001/XMLSchema#float"
                        | "http://www.w3.org/2001/XMLSchema#double" => {
                            // Output numeric types without quotes
                            triple
                                .object_value
                                .to_string()
                                .color(literal_color)
                                .to_string()
                        }
                        "http://www.w3.org/2001/XMLSchema#boolean" => {
                            // Output boolean values without quotes
                            triple
                                .object_value
                                .to_string()
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
                    println!("{indent}    {predicate} ;");
                    println!("{indent}        {object} .");
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
                    println!("{indent}    {predicate} ;");
                    println!("{indent}        {object} .");
                    current_subject = Some(subject);
                }
            } else {
                // First subject
                println!(
                    "{}{}",
                    indent,
                    subject.color(colors.get_color("subject")).bold()
                );
                println!("{indent}    {predicate} ;");
                println!("{indent}        {object} .");
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

// Print triples with or without prefixes to a writer
pub fn print_triples_to_writer<W: Write>(
    triples: &[OwnedTriple],
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
    writer: &mut W,
) -> Result<()> {
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
            writeln!(
                writer,
                "{} {{",
                formatted_graph.color(colors.get_color("graph"))
            )?;
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
                if current == &subject {
                    // Same subject, continue with predicate-object
                    writeln!(writer, "{indent}    {predicate} ;")?;
                    writeln!(writer, "{indent}        {object} .")?;
                } else {
                    // Different subject, add blank line and start new triple
                    if graph_key.is_none() {
                        writeln!(writer)?; // Add a blank line between statements
                    }
                    writeln!(
                        writer,
                        "{}{} {} ;",
                        indent,
                        subject.color(colors.get_color("subject")),
                        predicate
                    )?;
                    writeln!(writer, "{indent}    {object} .")?;
                    current_subject = Some(subject);
                }
            } else {
                // First triple
                writeln!(
                    writer,
                    "{}{} {} ;",
                    indent,
                    subject.color(colors.get_color("subject")),
                    predicate
                )?;
                writeln!(writer, "{indent}    {object} .")?;
                current_subject = Some(subject);
            }
        }

        // Close the graph block if it's a named graph
        if graph_key.is_some() {
            writeln!(writer, "}}")?;
            writeln!(writer)?; // Add a blank line after each graph
        }
    }

    Ok(())
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

// Helper function to print prefixes to a writer
fn print_prefixes_to_writer<W: Write>(
    prefixes: &HashMap<String, String>,
    colors: &config::ColorConfig,
    writer: &mut W,
) -> Result<()> {
    for (prefix, iri) in prefixes {
        writeln!(
            writer,
            "{} {}: <{}> .",
            "PREFIX".color(colors.get_color("prefix")),
            prefix.color(colors.get_color("prefix")),
            iri
        )?;
    }

    if !prefixes.is_empty() {
        writeln!(writer)?; // Add a blank line after prefixes
    }

    Ok(())
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

// Helper function to collect and print triples to a writer
fn collect_and_print_triples_to_writer<W: Write>(
    triples: &mut [OwnedTriple],
    prefixes: &mut HashMap<String, String>,
    parser_prefixes: impl Iterator<Item = (String, String)>,
    should_expand: bool,
    colors: &config::ColorConfig,
    writer: &mut W,
) -> Result<()> {
    // Get prefixes from parser
    for (prefix, iri) in parser_prefixes {
        prefixes.insert(prefix, iri);
    }

    if !should_expand {
        // Print prefixes
        print_prefixes_to_writer(prefixes, colors, writer)?;

        // Print triples with prefixes
        print_triples_to_writer(triples, Some(prefixes), colors, writer)?;
    } else {
        // Print triples without prefixes
        print_triples_to_writer(triples, None, colors, writer)?;
    }

    Ok(())
}

// Helper function to get terminal height
pub fn get_terminal_height() -> usize {
    if let Some((Width(_), Height(height))) = terminal_size() {
        height as usize
    } else {
        24 // Default fallback
    }
}

// Estimate the number of lines the output will take
pub fn estimate_output_lines(
    triples: &[OwnedTriple],
    prefixes: &HashMap<String, String>,
    should_expand: bool,
) -> usize {
    let mut lines = 0;

    // Count prefix lines if not expanding
    if !should_expand && !prefixes.is_empty() {
        lines += prefixes.len(); // PREFIX lines
        lines += 1; // Blank line after prefixes
    }

    // Group triples by graph to estimate lines more accurately
    let mut graph_groups: HashMap<Option<String>, Vec<&OwnedTriple>> = HashMap::new();
    for triple in triples {
        graph_groups
            .entry(triple.graph.clone())
            .or_default()
            .push(triple);
    }

    for (graph_key, triples_in_graph) in &graph_groups {
        // Graph opening line if it's a named graph
        if graph_key.is_some() {
            lines += 1;
        }

        // Group by subject to estimate lines per subject
        let mut subject_groups: HashMap<String, usize> = HashMap::new();
        for triple in triples_in_graph {
            let subject_key = format!(
                "{}:{}",
                match triple.subject_type {
                    SubjectType::NamedNode => "n",
                    SubjectType::BlankNode => "b",
                },
                triple.subject_value
            );
            *subject_groups.entry(subject_key).or_default() += 1;
        }

        // Each subject group takes at least 3 lines (subject line + predicate line + object line)
        // Plus blank lines between subjects
        for (i, (_subject, predicate_count)) in subject_groups.iter().enumerate() {
            lines += 3; // Minimum lines per subject
            lines += predicate_count.saturating_sub(1) * 2; // Additional predicate-object pairs

            if i > 0 {
                lines += 1; // Blank line between subjects
            }
        }

        // Graph closing line if it's a named graph
        if graph_key.is_some() {
            lines += 2; // Closing brace + blank line
        }
    }

    lines
}

// Determine if paging should be used based on content length
pub fn should_use_pager<A: ArgsConfig>(
    args: &A,
    config: &config::Config,
    estimated_lines: usize,
) -> bool {
    // If user explicitly disabled paging, respect that
    if args.no_pager_explicit() {
        return false;
    }

    // If user explicitly enabled paging, respect that
    if args.use_pager(config) {
        return true;
    }

    // Check if auto-paging is enabled
    if !config.output.auto_pager {
        return false;
    }

    // Determine threshold
    let threshold = if config.output.auto_pager_threshold > 0 {
        config.output.auto_pager_threshold
    } else {
        get_terminal_height().saturating_sub(2) // Leave some space for prompt
    };

    estimated_lines > threshold
}

// Process input based on format
pub fn process_input<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<()> {
    // Determine the format to use
    let format = args.format().unwrap_or(InputFormat::Turtle);

    // Use the rio implementation
    match format {
        InputFormat::Turtle => process_turtle(reader, args, colors, config),
        InputFormat::TriG => process_trig(reader, args, colors, config),
        InputFormat::NTriples => process_ntriples(reader, args, colors, config),
        InputFormat::NQuads => process_nquads(reader, args, colors, config),
    }
}

// Process Turtle input (rio version)
fn process_turtle<R: Read, A: ArgsConfig>(
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

// Process TriG input (rio version)
fn process_trig<R: Read, A: ArgsConfig>(
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

// Process N-Triples input (rio version)
fn process_ntriples<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<()> {
    let mut parser = NTriplesParser::new(reader);
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

    // N-Triples doesn't have prefixes, so we pass an empty iterator
    collect_and_print_triples(
        &mut triples,
        &mut prefixes,
        std::iter::empty(),
        args.expand(config),
        colors,
    );

    Ok(())
}

// Process N-Quads input (rio version)
fn process_nquads<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<()> {
    let mut parser = NQuadsParser::new(reader);
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

    // N-Quads doesn't have prefixes, so we pass an empty iterator
    collect_and_print_triples(
        &mut triples,
        &mut prefixes,
        std::iter::empty(),
        args.expand(config),
        colors,
    );

    Ok(())
}

// Process Turtle input to a writer (rio version)
fn process_turtle_to_writer<R: Read, W: Write, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
    writer: &mut W,
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
    collect_and_print_triples_to_writer(
        &mut triples,
        &mut prefixes,
        parser
            .prefixes()
            .iter()
            .map(|(p, i)| (p.to_string(), i.to_string())),
        args.expand(config),
        colors,
        writer,
    )?;

    Ok(())
}

// Process TriG input to a writer (rio version)
fn process_trig_to_writer<R: Read, W: Write, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
    writer: &mut W,
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
    collect_and_print_triples_to_writer(
        &mut triples,
        &mut prefixes,
        parser
            .prefixes()
            .iter()
            .map(|(p, i)| (p.to_string(), i.to_string())),
        args.expand(config),
        colors,
        writer,
    )?;

    Ok(())
}

// Process N-Triples input to a writer (rio version)
fn process_ntriples_to_writer<R: Read, W: Write, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
    writer: &mut W,
) -> Result<()> {
    let mut parser = NTriplesParser::new(reader);
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

    // N-Triples doesn't have prefixes, so we pass an empty iterator
    collect_and_print_triples_to_writer(
        &mut triples,
        &mut prefixes,
        std::iter::empty(),
        args.expand(config),
        colors,
        writer,
    )?;

    Ok(())
}

// Process N-Quads input to a writer (rio version)
fn process_nquads_to_writer<R: Read, W: Write, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
    writer: &mut W,
) -> Result<()> {
    let mut parser = NQuadsParser::new(reader);
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

    // N-Quads doesn't have prefixes, so we pass an empty iterator
    collect_and_print_triples_to_writer(
        &mut triples,
        &mut prefixes,
        std::iter::empty(),
        args.expand(config),
        colors,
        writer,
    )?;

    Ok(())
}

// Process input and return formatted output as a string
pub fn process_input_to_string<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<String> {
    let mut output = Vec::new();
    process_input_to_writer(reader, args, colors, config, &mut output)?;
    Ok(String::from_utf8(output)?)
}

// Process input and write to any writer
pub fn process_input_to_writer<R: Read, W: Write, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
    writer: &mut W,
) -> Result<()> {
    // Determine the format to use
    let format = args.format().unwrap_or(InputFormat::Turtle);

    // Use the rio implementation
    match format {
        InputFormat::Turtle => process_turtle_to_writer(reader, args, colors, config, writer),
        InputFormat::TriG => process_trig_to_writer(reader, args, colors, config, writer),
        InputFormat::NTriples => process_ntriples_to_writer(reader, args, colors, config, writer),
        InputFormat::NQuads => process_nquads_to_writer(reader, args, colors, config, writer),
    }
}

// Process input with automatic paging detection
pub fn process_input_auto_pager<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    config: &config::Config,
) -> Result<()> {
    let colors = &args.get_colors(config);

    // First, we need to parse the input to estimate the output size
    // We'll collect the triples and then decide whether to use paging
    let format = args.format().unwrap_or(InputFormat::Turtle);

    let (triples, prefixes) = match format {
        InputFormat::Turtle => parse_turtle_for_estimation(reader)?,
        InputFormat::TriG => parse_trig_for_estimation(reader)?,
        InputFormat::NTriples => parse_ntriples_for_estimation(reader)?,
        InputFormat::NQuads => parse_nquads_for_estimation(reader)?,
    };

    // Estimate output lines
    let should_expand = args.expand(config);
    let estimated_lines = estimate_output_lines(&triples, &prefixes, should_expand);

    // Determine if we should use paging
    let use_paging = should_use_pager(args, config, estimated_lines);

    if use_paging && std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        // Use pager
        let mut output = Vec::new();
        render_output(&triples, &prefixes, should_expand, colors, &mut output)?;
        let output_str = String::from_utf8(output)?;

        let pager = minus::Pager::new();
        pager.set_text(output_str)?;
        minus::page_all(pager)?;
    } else {
        // Direct output
        render_output(
            &triples,
            &prefixes,
            should_expand,
            colors,
            &mut std::io::stdout(),
        )?;
    }

    Ok(())
}

// Helper function to render output to any writer
pub fn render_output<W: Write>(
    triples: &[OwnedTriple],
    prefixes: &HashMap<String, String>,
    should_expand: bool,
    colors: &config::ColorConfig,
    writer: &mut W,
) -> Result<()> {
    if !should_expand {
        // Print prefixes
        print_prefixes_to_writer(prefixes, colors, writer)?;
        // Print triples with prefixes
        print_triples_to_writer(triples, Some(prefixes), colors, writer)?;
    } else {
        // Print triples without prefixes
        print_triples_to_writer(triples, None, colors, writer)?;
    }
    Ok(())
}

// Parse Turtle input and return triples and prefixes for estimation
pub fn parse_turtle_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut parser = TurtleParser::new(reader, None);
    let mut triples = Vec::new();
    let mut prefixes = HashMap::new();

    // Process each triple
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all triples
    parser.parse_all(&mut callback)?;

    // Get prefixes from parser
    for (prefix, iri) in parser.prefixes().iter() {
        prefixes.insert(prefix.to_string(), iri.to_string());
    }

    Ok((triples, prefixes))
}

// Parse TriG input and return triples and prefixes for estimation
pub fn parse_trig_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut parser = TriGParser::new(reader, None);
    let mut triples = Vec::new();
    let mut prefixes = HashMap::new();

    // Process each quad
    let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
        let owned_triple = quad_to_owned(&quad);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all quads
    parser.parse_all(&mut callback)?;

    // Get prefixes from parser
    for (prefix, iri) in parser.prefixes().iter() {
        prefixes.insert(prefix.to_string(), iri.to_string());
    }

    Ok((triples, prefixes))
}

// Parse N-Triples input and return triples for estimation
pub fn parse_ntriples_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut parser = NTriplesParser::new(reader);
    let mut triples = Vec::new();

    // Process each triple
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all triples
    parser.parse_all(&mut callback)?;

    // N-Triples doesn't have prefixes
    Ok((triples, HashMap::new()))
}

// Parse N-Quads input and return triples for estimation
pub fn parse_nquads_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut parser = NQuadsParser::new(reader);
    let mut triples = Vec::new();

    // Process each quad
    let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
        let owned_triple = quad_to_owned(&quad);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all quads
    parser.parse_all(&mut callback)?;

    // N-Quads doesn't have prefixes
    Ok((triples, HashMap::new()))
}
