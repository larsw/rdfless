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
    // First, check if there's a base IRI (stored with empty string key)
    if let Some(base_iri) = prefixes.get("") {
        writeln!(
            writer,
            "{} <{}> .",
            colors.colorize_bold("@base", "base"),
            base_iri
        )?;
    }

    // Then print all other prefixes (skip the empty string key if present)
    for (prefix, iri) in prefixes {
        if prefix.is_empty() {
            continue; // Skip the base IRI, already printed above
        }
        writeln!(
            writer,
            "{} {}: <{}> .",
            colors.colorize("@prefix", "prefix"),
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

        // Group by subject within this graph while preserving input order.
        let mut subject_groups: Vec<(String, Vec<&OwnedTriple>)> = Vec::new();
        for triple in triples_in_graph {
            let subject = format_subject(triple, prefixes, colors);
            if let Some((_, subject_triples)) = subject_groups
                .iter_mut()
                .find(|(group_subject, _)| group_subject == &subject)
            {
                subject_triples.push(triple);
            } else {
                subject_groups.push((subject, vec![triple]));
            }
        }

        let terminal_width = get_terminal_width();

        for (subject_index, (subject, subject_triples)) in subject_groups.iter().enumerate() {
            // Indent more if we're in a named graph
            let indent = if graph_key.is_some() { "  " } else { "" };

            if subject_index > 0 && graph_key.is_none() {
                writeln!(writer)?; // Add a blank line between statements
            }

            for (triple_index, triple) in subject_triples.iter().enumerate() {
                let predicate = format_predicate(triple, prefixes, colors);
                let object = format_object(triple, prefixes, colors);
                let terminator = if triple_index + 1 == subject_triples.len() {
                    "."
                } else {
                    ";"
                };

                let predicate_line = if triple_index == 0 {
                    let subject_colored = colors.colorize(subject, "subject");
                    format!("{indent}{subject_colored} {predicate}")
                } else {
                    format!("{indent}    {predicate}")
                };
                let object_part = format!(" {object} {terminator}");
                let full_line = format!("{predicate_line}{object_part}");
                let line_length = strip_ansi_codes(&full_line).len();

                if line_length <= terminal_width {
                    writeln!(writer, "{full_line}")?;
                } else {
                    writeln!(writer, "{predicate_line}")?;
                    let object_indent = if triple_index == 0 {
                        format!("{indent}    ")
                    } else {
                        format!("{indent}        ")
                    };
                    writeln!(writer, "{object_indent}{object} {terminator}")?;
                }
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
