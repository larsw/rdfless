// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::{config, types::OwnedTriple, utils::resolve_uri_with_prefixes};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::io::Write;

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
    }
}

/// Format an owned predicate
pub fn format_predicate(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) -> String {
    resolve_uri_with_prefixes(&triple.predicate, prefixes)
        .color(colors.get_color("predicate"))
        .to_string()
}

/// Format an owned object
pub fn format_object(
    triple: &OwnedTriple,
    prefixes: Option<&HashMap<String, String>>,
    colors: &config::ColorConfig,
) -> String {
    match triple.object_type {
        crate::types::ObjectType::NamedNode => {
            resolve_uri_with_prefixes(&triple.object_value, prefixes)
                .color(colors.get_color("object"))
                .to_string()
        }
        crate::types::ObjectType::BlankNode => format!("_:{}", triple.object_value)
            .color(colors.get_color("object"))
            .to_string(),
        crate::types::ObjectType::Literal => {
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
                    match datatype.as_str() {
                        "http://www.w3.org/2001/XMLSchema#integer"
                        | "http://www.w3.org/2001/XMLSchema#decimal"
                        | "http://www.w3.org/2001/XMLSchema#float"
                        | "http://www.w3.org/2001/XMLSchema#double" => {
                            // Output numeric types without quotes
                            triple.object_value.to_string().color(literal_color).to_string()
                        }
                        "http://www.w3.org/2001/XMLSchema#boolean" => {
                            // Output boolean values without quotes
                            triple.object_value.to_string().color(literal_color).to_string()
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

/// Generic writer for formatted output
pub fn write_formatted_output<W: Write>(
    writer: &mut W,
    content: &str,
) -> Result<()> {
    writeln!(writer, "{}", content)?;
    Ok(())
}
