// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::types::{ObjectType, OwnedTriple, SubjectType};
use oxrdf::{vocab::xsd, NamedOrBlankNode, Quad, Term, Triple};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// Convert a Triple to an OwnedTriple (oxrdf version with RDF-star support)
pub fn triple_to_owned(triple: &Triple) -> OwnedTriple {
    let (subject_type, subject_value, subject_triple) = match &triple.subject {
        NamedOrBlankNode::NamedNode(node) => {
            (SubjectType::NamedNode, node.as_str().to_string(), None)
        }
        NamedOrBlankNode::BlankNode(node) => {
            (SubjectType::BlankNode, node.as_str().to_string(), None)
        }
    };

    let predicate = triple.predicate.as_str().to_string();

    let (object_type, object_value, object_datatype, object_language, object_triple) =
        match &triple.object {
            Term::NamedNode(node) => (
                ObjectType::NamedNode,
                node.as_str().to_string(),
                None,
                None,
                None,
            ),
            Term::BlankNode(node) => (
                ObjectType::BlankNode,
                node.as_str().to_string(),
                None,
                None,
                None,
            ),
            Term::Literal(literal) => {
                let value = literal.value().to_string();
                if let Some(language) = literal.language() {
                    (
                        ObjectType::Literal,
                        value,
                        None,
                        Some(language.to_string()),
                        None,
                    )
                } else if literal.datatype() != xsd::STRING {
                    (
                        ObjectType::Literal,
                        value,
                        Some(literal.datatype().as_str().to_string()),
                        None,
                        None,
                    )
                } else {
                    (ObjectType::Literal, value, None, None, None)
                }
            }
            Term::Triple(embedded_triple) => {
                // RDF-star: embedded triple as object
                let owned_embedded = Box::new(triple_to_owned(embedded_triple));
                (
                    ObjectType::Triple,
                    format!("<< {} >>", format_embedded_triple(&owned_embedded)),
                    None,
                    None,
                    Some(owned_embedded),
                )
            }
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
        subject_triple,
        object_triple,
    }
}

/// Helper function to format an embedded triple for display
fn format_embedded_triple(triple: &OwnedTriple) -> String {
    format!(
        "{} {} {}",
        triple.subject_value, triple.predicate, triple.object_value
    )
}

/// Convert a Quad to an OwnedTriple with graph information
pub fn quad_to_owned(quad: &Quad) -> OwnedTriple {
    // First convert the triple part
    let mut owned_triple = triple_to_owned(&Triple {
        subject: quad.subject.clone(),
        predicate: quad.predicate.clone(),
        object: quad.object.clone(),
    });

    // Then add the graph information if available
    match &quad.graph_name {
        oxrdf::GraphName::NamedNode(graph_node) => {
            owned_triple.graph = Some(graph_node.as_str().to_string());
        }
        oxrdf::GraphName::BlankNode(graph_node) => {
            owned_triple.graph = Some(graph_node.as_str().to_string());
        }
        oxrdf::GraphName::DefaultGraph => {
            // Default graph, no graph name to set
        }
    }

    owned_triple
}

/// Extract prefix declarations from RDF content
/// Supports both Turtle (@prefix) and SPARQL (PREFIX) syntax
pub fn extract_prefixes<R: Read>(reader: R) -> HashMap<String, String> {
    let mut prefixes = HashMap::new();
    let buf_reader = BufReader::new(reader);

    for line in buf_reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse @prefix declarations (Turtle syntax)
        if let Some(prefix_decl) = parse_turtle_prefix(trimmed) {
            prefixes.insert(prefix_decl.0, prefix_decl.1);
            continue;
        }

        // Parse PREFIX declarations (SPARQL syntax)
        if let Some(prefix_decl) = parse_sparql_prefix(trimmed) {
            prefixes.insert(prefix_decl.0, prefix_decl.1);
            continue;
        }

        // Parse @base declarations
        if let Some(base_iri) = parse_base_declaration(trimmed) {
            prefixes.insert("".to_string(), base_iri);
            continue;
        }
    }

    prefixes
}

/// Parse Turtle @prefix declarations
/// Format: @prefix prefix: <iri> .
fn parse_turtle_prefix(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();

    if !trimmed.starts_with("@prefix") {
        return None;
    }

    // Remove @prefix and trim
    let without_prefix = trimmed[7..].trim();

    // Find the colon that separates prefix name from IRI
    let colon_pos = without_prefix.find(':')?;
    let prefix_name = without_prefix[..colon_pos].trim();

    // Find the IRI between angle brackets
    let iri_part = without_prefix[colon_pos + 1..].trim();
    let start_bracket = iri_part.find('<')?;
    let end_bracket = iri_part.find('>')?;

    if end_bracket <= start_bracket {
        return None;
    }

    let iri = iri_part[start_bracket + 1..end_bracket].to_string();

    Some((prefix_name.to_string(), iri))
}

/// Parse SPARQL PREFIX declarations
/// Format: PREFIX prefix: <iri>
fn parse_sparql_prefix(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();

    if !trimmed.to_uppercase().starts_with("PREFIX") {
        return None;
    }

    // Remove PREFIX and trim
    let without_prefix = trimmed[6..].trim();

    // Find the colon that separates prefix name from IRI
    let colon_pos = without_prefix.find(':')?;
    let prefix_name = without_prefix[..colon_pos].trim();

    // Find the IRI between angle brackets
    let iri_part = without_prefix[colon_pos + 1..].trim();
    let start_bracket = iri_part.find('<')?;
    let end_bracket = iri_part.find('>')?;

    if end_bracket <= start_bracket {
        return None;
    }

    let iri = iri_part[start_bracket + 1..end_bracket].to_string();

    Some((prefix_name.to_string(), iri))
}

/// Parse @base declarations
/// Format: @base <iri> .
fn parse_base_declaration(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if !trimmed.starts_with("@base") {
        return None;
    }

    // Remove @base and trim
    let without_base = trimmed[5..].trim();

    // Find the IRI between angle brackets
    let start_bracket = without_base.find('<')?;
    let end_bracket = without_base.find('>')?;

    if end_bracket <= start_bracket {
        return None;
    }

    Some(without_base[start_bracket + 1..end_bracket].to_string())
}
