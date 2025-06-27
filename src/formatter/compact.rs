// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::{
    config,
    formatter::writer::{format_object, format_predicate, format_subject},
    types::OwnedTriple,
    utils::resolve_uri_with_prefixes,
};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::io::Write;

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
            writeln!(
                writer,
                "{} {{",
                formatted_graph.color(colors.get_color("graph"))
            )?;
        }

        // Group by subject within this graph
        let mut current_subject: Option<String> = None;

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
