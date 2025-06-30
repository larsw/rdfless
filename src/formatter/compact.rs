// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::{
    config,
    formatter::writer::{format_object, format_predicate, format_subject},
    types::OwnedTriple,
    utils::{get_terminal_width, resolve_uri_with_prefixes},
};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;

/// Strip ANSI color codes to get actual text length
fn strip_ansi_codes(text: &str) -> String {
    // Simple regex-free approach to strip ANSI escape sequences
    let mut result = String::new();
    let mut chars = text.chars();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Found escape sequence, skip until 'm'
            for escape_ch in chars.by_ref() {
                if escape_ch == 'm' {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Print prefixes to a writer
pub fn print_prefixes_to_writer<W: Write>(
    prefixes: &HashMap<String, String>,
    colors: &config::ColorConfig,
    writer: &mut W,
) -> Result<()> {
    for (prefix, iri) in prefixes {
        writeln!(
            writer,
            "{} {}: <{}> .",
            colors.colorize("PREFIX", "prefix"),
            colors.colorize(prefix, "prefix"),
            iri
        )?;
    }

    if !prefixes.is_empty() {
        writeln!(writer)?; // Add a blank line after prefixes
    }

    Ok(())
}

/// Print triples with prefixes (compact mode) to a writer
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
            writeln!(writer, "{} {{", colors.colorize(&formatted_graph, "graph"))?;
        }

        // Group by subject within this graph
        let mut current_subject: Option<String> = None;
        let terminal_width = get_terminal_width();

        for triple in triples_in_graph {
            let subject = format_subject(triple, prefixes, colors);
            let predicate = format_predicate(triple, prefixes, colors);
            let object = format_object(triple, prefixes, colors);

            // Indent more if we're in a named graph
            let indent = if graph_key.is_some() { "  " } else { "" };

            // Check if we're continuing with the same subject
            if let Some(ref current) = current_subject {
                if current == &subject {
                    // Same subject, continue with predicate-object
                    // Calculate if we can fit predicate and object on the same line
                    let predicate_line = format!("{indent}    {predicate}");
                    let object_part = format!(" {object} .");
                    let full_line = format!("{predicate_line}{object_part}");
                    let line_length = strip_ansi_codes(&full_line).len();

                    if line_length <= terminal_width {
                        // Fits on one line
                        writeln!(writer, "{full_line}")?;
                    } else {
                        // Split across lines
                        writeln!(writer, "{predicate_line} ;")?;
                        writeln!(writer, "{indent}        {object} .")?;
                    }
                } else {
                    // Different subject, add blank line and start new triple
                    if graph_key.is_none() {
                        writeln!(writer)?; // Add a blank line between statements
                    }

                    // Calculate if subject, predicate, and object fit on one line
                    let subject_colored = colors.colorize(&subject, "subject");
                    let subject_line = format!("{indent}{subject_colored} {predicate}");
                    let object_part = format!(" {object} .");
                    let full_line = format!("{subject_line}{object_part}");
                    let line_length = strip_ansi_codes(&full_line).len();

                    if line_length <= terminal_width {
                        // Fits on one line
                        writeln!(writer, "{full_line}")?;
                    } else {
                        // Split across lines
                        writeln!(writer, "{subject_line} ;")?;
                        writeln!(writer, "{indent}    {object} .")?;
                    }
                    current_subject = Some(subject);
                }
            } else {
                // First triple
                let subject_colored = colors.colorize(&subject, "subject");
                let subject_line = format!("{indent}{subject_colored} {predicate}");
                let object_part = format!(" {object} .");
                let full_line = format!("{subject_line}{object_part}");
                let line_length = strip_ansi_codes(&full_line).len();

                if line_length <= terminal_width {
                    // Fits on one line
                    writeln!(writer, "{full_line}")?;
                } else {
                    // Split across lines
                    writeln!(writer, "{subject_line} ;")?;
                    writeln!(writer, "{indent}    {object} .")?;
                }
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
