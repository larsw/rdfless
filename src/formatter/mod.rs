// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

pub mod writer;
pub mod compact;
pub mod expanded;

use crate::{config, types::OwnedTriple, utils};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;

/// Render output to any writer
pub fn render_output<W: Write>(
    triples: &[OwnedTriple],
    prefixes: &HashMap<String, String>,
    should_expand: bool,
    colors: &config::ColorConfig,
    writer: &mut W,
) -> Result<()> {
    if !should_expand {
        // Print prefixes and triples with prefixes (compact mode)
        compact::print_prefixes_to_writer(prefixes, colors, writer)?;
        compact::print_triples_to_writer(triples, Some(prefixes), colors, writer)?;
    } else {
        // Print triples without prefixes (expanded mode)
        expanded::print_triples_to_writer(triples, None, colors, writer)?;
    }
    Ok(())
}

/// Estimate the number of lines the output will take
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
                    crate::types::SubjectType::NamedNode => "n",
                    crate::types::SubjectType::BlankNode => "b",
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
