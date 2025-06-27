// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::types::{ObjectType, OwnedTriple, SubjectType};
use rio_api::model::{Literal, Quad, Subject, Term, Triple};

/// Convert a Triple to an OwnedTriple (rio version)
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

/// Convert a Quad to an OwnedTriple with graph information
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
