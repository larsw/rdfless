// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::types::{ObjectType, OwnedTriple, SubjectType};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// Parse PROV-N input and convert to RDF triples
pub fn parse_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut triples = Vec::new();
    let mut prefixes = HashMap::new();
    let mut lines_buffer = Vec::new();

    // First pass: collect all lines and extract prefixes
    for line in reader.lines() {
        let line = line?;
        lines_buffer.push(line.clone());
        
        // Extract prefix declarations
        if let Some((prefix, iri)) = parse_provn_prefix(&line) {
            prefixes.insert(prefix, iri);
        }
    }

    // Second pass: parse PROV-N statements and convert to triples
    for line in lines_buffer {
        let trimmed = line.trim();
        
        // Skip empty lines, comments, document markers, and prefix declarations
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("document")
            || trimmed.starts_with("endDocument")
            || trimmed.starts_with("prefix")
        {
            continue;
        }

        // Parse PROV-N statements
        if let Some(mut statement_triples) = parse_provn_statement(trimmed, &prefixes)? {
            triples.append(&mut statement_triples);
        }
    }

    Ok((triples, prefixes))
}

/// Parse PROV-N prefix declaration
/// Format: prefix ex <http://example.org/>
fn parse_provn_prefix(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    
    if !trimmed.starts_with("prefix") {
        return None;
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let prefix_name = parts[1].to_string();
    let iri = parts[2].trim_matches('<').trim_matches('>').to_string();

    Some((prefix_name, iri))
}

/// Expand a qualified name using prefix map
fn expand_qname(qname: &str, prefixes: &HashMap<String, String>) -> String {
    if qname.starts_with('<') && qname.ends_with('>') {
        // Already a full IRI
        return qname.trim_matches('<').trim_matches('>').to_string();
    }

    if let Some(colon_pos) = qname.find(':') {
        let prefix = &qname[..colon_pos];
        let local = &qname[colon_pos + 1..];
        
        if let Some(base_iri) = prefixes.get(prefix) {
            return format!("{}{}", base_iri, local);
        }
    }

    qname.to_string()
}

/// Parse a PROV-N statement and convert to RDF triples
fn parse_provn_statement(
    statement: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let statement = statement.trim();
    
    // Remove trailing parenthesis if present
    let statement = statement.trim_end_matches(')');
    
    // Find the opening parenthesis
    let paren_pos = statement.find('(').ok_or_else(|| anyhow!("Invalid PROV-N statement"))?;
    let relation_type = statement[..paren_pos].trim();
    let args = statement[paren_pos + 1..].trim();

    match relation_type {
        "entity" => parse_entity_statement(args, prefixes),
        "activity" => parse_activity_statement(args, prefixes),
        "agent" => parse_agent_statement(args, prefixes),
        "wasGeneratedBy" => parse_was_generated_by_statement(args, prefixes),
        "used" => parse_used_statement(args, prefixes),
        "wasAssociatedWith" => parse_was_associated_with_statement(args, prefixes),
        "wasAttributedTo" => parse_was_attributed_to_statement(args, prefixes),
        "wasDerivedFrom" => parse_was_derived_from_statement(args, prefixes),
        "wasInformedBy" => parse_was_informed_by_statement(args, prefixes),
        "actedOnBehalfOf" => parse_acted_on_behalf_of_statement(args, prefixes),
        _ => {
            // Unknown relation type, skip
            Ok(None)
        }
    }
}

/// Parse entity statement: entity(id, [attributes])
fn parse_entity_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.is_empty() {
        return Ok(None);
    }

    let entity_id = expand_qname(&parts[0], prefixes);
    let mut triples = Vec::new();

    // Add rdf:type prov:Entity triple
    triples.push(create_type_triple(
        &entity_id,
        "http://www.w3.org/ns/prov#Entity",
    ));

    // Parse attributes if present
    if parts.len() > 1 {
        let attributes = parse_attributes(&parts[1], prefixes)?;
        for (attr_name, attr_value) in attributes {
            triples.push(create_triple(
                &entity_id,
                &attr_name,
                &attr_value,
                ObjectType::Literal,
            ));
        }
    }

    Ok(Some(triples))
}

/// Parse activity statement: activity(id, startTime, endTime, [attributes])
fn parse_activity_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.is_empty() {
        return Ok(None);
    }

    let activity_id = expand_qname(&parts[0], prefixes);
    let mut triples = Vec::new();

    // Add rdf:type prov:Activity triple
    triples.push(create_type_triple(
        &activity_id,
        "http://www.w3.org/ns/prov#Activity",
    ));

    // Add start time if present
    if parts.len() > 1 && !parts[1].is_empty() && parts[1] != "-" {
        triples.push(create_triple(
            &activity_id,
            "http://www.w3.org/ns/prov#startedAtTime",
            &parts[1],
            ObjectType::Literal,
        ));
    }

    // Add end time if present
    if parts.len() > 2 && !parts[2].is_empty() && parts[2] != "-" {
        triples.push(create_triple(
            &activity_id,
            "http://www.w3.org/ns/prov#endedAtTime",
            &parts[2],
            ObjectType::Literal,
        ));
    }

    Ok(Some(triples))
}

/// Parse agent statement: agent(id, [attributes])
fn parse_agent_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.is_empty() {
        return Ok(None);
    }

    let agent_id = expand_qname(&parts[0], prefixes);
    let mut triples = Vec::new();

    // Add rdf:type prov:Agent triple
    triples.push(create_type_triple(
        &agent_id,
        "http://www.w3.org/ns/prov#Agent",
    ));

    // Parse attributes if present
    if parts.len() > 1 {
        let attributes = parse_attributes(&parts[1], prefixes)?;
        for (attr_name, attr_value) in attributes {
            triples.push(create_triple(
                &agent_id,
                &attr_name,
                &attr_value,
                ObjectType::Literal,
            ));
        }
    }

    Ok(Some(triples))
}

/// Parse wasGeneratedBy statement: wasGeneratedBy(entity, activity, time, [attributes])
fn parse_was_generated_by_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let entity_id = expand_qname(&parts[0], prefixes);
    let activity_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &entity_id,
        "http://www.w3.org/ns/prov#wasGeneratedBy",
        &activity_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse used statement: used(activity, entity, time, [attributes])
fn parse_used_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let activity_id = expand_qname(&parts[0], prefixes);
    let entity_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &activity_id,
        "http://www.w3.org/ns/prov#used",
        &entity_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse wasAssociatedWith statement: wasAssociatedWith(activity, agent, [plan], [attributes])
fn parse_was_associated_with_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let activity_id = expand_qname(&parts[0], prefixes);
    let agent_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &activity_id,
        "http://www.w3.org/ns/prov#wasAssociatedWith",
        &agent_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse wasAttributedTo statement: wasAttributedTo(entity, agent, [attributes])
fn parse_was_attributed_to_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let entity_id = expand_qname(&parts[0], prefixes);
    let agent_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &entity_id,
        "http://www.w3.org/ns/prov#wasAttributedTo",
        &agent_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse wasDerivedFrom statement: wasDerivedFrom(entity2, entity1, [attributes])
fn parse_was_derived_from_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let entity2_id = expand_qname(&parts[0], prefixes);
    let entity1_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &entity2_id,
        "http://www.w3.org/ns/prov#wasDerivedFrom",
        &entity1_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse wasInformedBy statement: wasInformedBy(activity2, activity1, [attributes])
fn parse_was_informed_by_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let activity2_id = expand_qname(&parts[0], prefixes);
    let activity1_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &activity2_id,
        "http://www.w3.org/ns/prov#wasInformedBy",
        &activity1_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse actedOnBehalfOf statement: actedOnBehalfOf(agent2, agent1, [activity], [attributes])
fn parse_acted_on_behalf_of_statement(
    args: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Option<Vec<OwnedTriple>>> {
    let parts = parse_statement_args(args)?;
    if parts.len() < 2 {
        return Ok(None);
    }

    let agent2_id = expand_qname(&parts[0], prefixes);
    let agent1_id = expand_qname(&parts[1], prefixes);

    let triple = create_triple(
        &agent2_id,
        "http://www.w3.org/ns/prov#actedOnBehalfOf",
        &agent1_id,
        ObjectType::NamedNode,
    );

    Ok(Some(vec![triple]))
}

/// Parse statement arguments, handling commas and brackets
fn parse_statement_args(args: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_quotes = false;

    for ch in args.chars() {
        match ch {
            '[' if !in_quotes => {
                depth += 1;
                current.push(ch);
            }
            ']' if !in_quotes => {
                depth -= 1;
                current.push(ch);
            }
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            ',' if depth == 0 && !in_quotes => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current.trim().to_string());
    }

    Ok(parts)
}

/// Parse attribute list: [attr1=value1, attr2=value2, ...]
fn parse_attributes(
    attrs_str: &str,
    prefixes: &HashMap<String, String>,
) -> Result<Vec<(String, String)>> {
    let mut attributes = Vec::new();
    
    let attrs_str = attrs_str.trim();
    if !attrs_str.starts_with('[') || !attrs_str.ends_with(']') {
        return Ok(attributes);
    }

    let attrs_content = &attrs_str[1..attrs_str.len() - 1];
    let attr_parts = attrs_content.split(',');

    for attr_part in attr_parts {
        let attr_part = attr_part.trim();
        if let Some(eq_pos) = attr_part.find('=') {
            let attr_name = attr_part[..eq_pos].trim();
            let attr_value = attr_part[eq_pos + 1..].trim().trim_matches('"');
            
            let expanded_name = expand_qname(attr_name, prefixes);
            attributes.push((expanded_name, attr_value.to_string()));
        }
    }

    Ok(attributes)
}

/// Create an RDF type triple
fn create_type_triple(subject: &str, type_iri: &str) -> OwnedTriple {
    OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: subject.to_string(),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
        object_type: ObjectType::NamedNode,
        object_value: type_iri.to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
        subject_triple: None,
        object_triple: None,
    }
}

/// Create an RDF triple
fn create_triple(
    subject: &str,
    predicate: &str,
    object: &str,
    object_type: ObjectType,
) -> OwnedTriple {
    OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: subject.to_string(),
        predicate: predicate.to_string(),
        object_type,
        object_value: object.to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
        subject_triple: None,
        object_triple: None,
    }
}
