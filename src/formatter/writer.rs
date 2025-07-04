// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::{config, types::OwnedTriple, utils::resolve_uri_with_prefixes};
use std::collections::HashMap;

/// Format an owned subject
pub fn format_subject(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    _colors: &config::ColorConfig,
) -> String {
    match triple.subject_type {
        crate::types::SubjectType::NamedNode => {
            resolve_uri_with_prefixes(&triple.subject_value, prefixes)
        }
        crate::types::SubjectType::BlankNode => format!("_:{}", triple.subject_value),
        crate::types::SubjectType::Triple => {
            // RDF-star: embedded triple as subject
            triple.subject_value.clone() // Already formatted in triple_to_owned
        }
    }
}

/// Format an owned predicate
pub fn format_predicate(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) -> String {
    let predicate_text = resolve_uri_with_prefixes(&triple.predicate, prefixes);
    colors.colorize(&predicate_text, "predicate")
}

/// Format an owned object
pub fn format_object(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) -> String {
    match triple.object_type {
        crate::types::ObjectType::NamedNode => {
            let object_text = resolve_uri_with_prefixes(&triple.object_value, prefixes);
            colors.colorize(&object_text, "object")
        }
        crate::types::ObjectType::BlankNode => {
            let blank_text = format!("_:{}", triple.object_value);
            colors.colorize(&blank_text, "object")
        }
        crate::types::ObjectType::Literal => {
            if let Some(language) = &triple.object_language {
                let literal_text = format!("\"{}\"@{}", triple.object_value, language);
                colors.colorize(&literal_text, "literal")
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
                    let literal_text = match datatype.as_str() {
                        "http://www.w3.org/2001/XMLSchema#integer"
                        | "http://www.w3.org/2001/XMLSchema#decimal"
                        | "http://www.w3.org/2001/XMLSchema#float"
                        | "http://www.w3.org/2001/XMLSchema#double" => {
                            // Output numeric types without quotes
                            triple.object_value.to_string()
                        }
                        "http://www.w3.org/2001/XMLSchema#boolean" => {
                            // Output boolean values without quotes
                            triple.object_value.to_string()
                        }
                        _ => {
                            // Keep other types (like strings, dates, etc.) in quotes
                            format!("\"{}\"", triple.object_value)
                        }
                    };
                    colors.colorize(&literal_text, "literal")
                } else {
                    // In expanded mode or for non-basic data types, show the full datatype
                    let datatype_str = resolve_uri_with_prefixes(datatype, prefixes);
                    let literal_text = format!("\"{}\"^^{}", triple.object_value, datatype_str);
                    colors.colorize(&literal_text, "literal")
                }
            } else {
                let literal_text = format!("\"{}\"", triple.object_value);
                colors.colorize(&literal_text, "literal")
            }
        }
        crate::types::ObjectType::Triple => {
            // RDF-star: embedded triple as object
            colors.colorize(&triple.object_value, "object")
        }
    }
}
