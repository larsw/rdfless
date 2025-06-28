// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::parser::common::{quad_to_owned, triple_to_owned};
use crate::types::OwnedTriple;
use anyhow::Result;
use rio_api::model::{Quad, Triple};
use rio_api::parser::{QuadsParser, TriplesParser};
use rio_turtle::{NQuadsParser, NTriplesParser, TriGParser, TurtleError, TurtleParser};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug)]
pub struct ParseResult {
    pub triples: Vec<OwnedTriple>,
    pub prefixes: HashMap<String, String>,
    pub errors: Vec<ParseError>,
}

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub position: usize,
    pub message: String,
}

impl Default for ParseResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseResult {
    pub fn new() -> Self {
        ParseResult {
            triples: Vec::new(),
            prefixes: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn triple_count(&self) -> usize {
        self.triples.len()
    }
}

/// Parse Turtle input with robust error handling
pub fn parse_turtle_robust<R: Read>(
    reader: BufReader<R>,
    continue_on_error: bool,
) -> Result<ParseResult> {
    if continue_on_error {
        parse_turtle_line_by_line(reader)
    } else {
        // Use the standard parser for strict mode
        let mut parser = TurtleParser::new(reader, None);
        let mut result = ParseResult::new();

        let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
            let owned_triple = triple_to_owned(&triple);
            result.triples.push(owned_triple);
            Ok(())
        };

        match parser.parse_all(&mut callback) {
            Ok(_) => {
                // Get prefixes from parser
                for (prefix, iri) in parser.prefixes().iter() {
                    result.prefixes.insert(prefix.to_string(), iri.to_string());
                }
                Ok(result)
            }
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

/// Parse Turtle input line by line with error recovery
fn parse_turtle_line_by_line<R: Read>(reader: BufReader<R>) -> Result<ParseResult> {
    let mut result = ParseResult::new();
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    // First pass: collect all prefix declarations
    let mut prefix_declarations = String::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("@prefix") {
            prefix_declarations.push_str(line);
            prefix_declarations.push('\n');
        }
    }

    // Second pass: try to parse each triple statement
    let mut current_statement = String::new();

    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num + 1;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip prefix lines (already processed)
        if trimmed.starts_with("@prefix") {
            continue;
        }

        current_statement.push_str(line);
        current_statement.push('\n');

        // If line ends with period, try to parse the complete statement
        if trimmed.ends_with('.') {
            let full_input = format!("{prefix_declarations}\n{current_statement}");

            match try_parse_turtle_fragment(&full_input) {
                Ok((triples, prefixes)) => {
                    // Successfully parsed, add to results
                    result.triples.extend(triples);
                    for (prefix, iri) in prefixes {
                        result.prefixes.insert(prefix, iri);
                    }
                }
                Err(e) => {
                    result.errors.push(ParseError {
                        line: line_number,
                        position: 1,
                        message: format!("Parse error: {e}"),
                    });
                }
            }
            current_statement.clear();
        }
    }

    // Try to parse any remaining accumulated statement
    if !current_statement.trim().is_empty() {
        let full_input = format!("{prefix_declarations}\n{current_statement}");
        if let Err(e) = try_parse_turtle_fragment(&full_input) {
            result.errors.push(ParseError {
                line: lines.len(),
                position: 1,
                message: format!("Parse error in final statement: {e}"),
            });
        }
    }

    Ok(result)
}

fn try_parse_turtle_fragment(
    input: &str,
) -> std::result::Result<(Vec<OwnedTriple>, HashMap<String, String>), TurtleError> {
    let reader = BufReader::new(input.as_bytes());
    let mut parser = TurtleParser::new(reader, None);
    let mut triples = Vec::new();

    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    parser.parse_all(&mut callback)?;

    let mut prefixes = HashMap::new();
    for (prefix, iri) in parser.prefixes().iter() {
        prefixes.insert(prefix.to_string(), iri.to_string());
    }

    Ok((triples, prefixes))
}

/// Parse TriG input with robust error handling
pub fn parse_trig_robust<R: Read>(
    reader: BufReader<R>,
    continue_on_error: bool,
) -> Result<ParseResult> {
    if continue_on_error {
        parse_trig_line_by_line(reader)
    } else {
        // Use the standard parser for strict mode
        let mut parser = TriGParser::new(reader, None);
        let mut result = ParseResult::new();

        let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
            let owned_triple = quad_to_owned(&quad);
            result.triples.push(owned_triple);
            Ok(())
        };

        match parser.parse_all(&mut callback) {
            Ok(_) => {
                // Get prefixes from parser
                for (prefix, iri) in parser.prefixes().iter() {
                    result.prefixes.insert(prefix.to_string(), iri.to_string());
                }
                Ok(result)
            }
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

fn parse_trig_line_by_line<R: Read>(reader: BufReader<R>) -> Result<ParseResult> {
    let mut result = ParseResult::new();
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    // First pass: collect all prefix declarations
    let mut prefix_declarations = String::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("@prefix") {
            prefix_declarations.push_str(line);
            prefix_declarations.push('\n');
        }
    }

    // Second pass: try to parse each statement
    let mut current_statement = String::new();
    let mut brace_depth = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num + 1;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip prefix lines (already processed)
        if trimmed.starts_with("@prefix") {
            continue;
        }

        current_statement.push_str(line);
        current_statement.push('\n');

        // Track brace depth for TriG graphs
        brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
        brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

        // If line ends with period or closing brace and we're not inside a graph, try to parse
        if trimmed.ends_with('.') || (trimmed.ends_with('}') && brace_depth == 0) {
            let full_input = format!("{prefix_declarations}\n{current_statement}");

            match try_parse_trig_fragment(&full_input) {
                Ok((triples, prefixes)) => {
                    // Successfully parsed, add to results
                    result.triples.extend(triples);
                    for (prefix, iri) in prefixes {
                        result.prefixes.insert(prefix, iri);
                    }
                }
                Err(e) => {
                    result.errors.push(ParseError {
                        line: line_number,
                        position: 1,
                        message: format!("Parse error: {e}"),
                    });
                }
            }
            current_statement.clear();
        }
    }

    // Try to parse any remaining accumulated statement
    if !current_statement.trim().is_empty() {
        let full_input = format!("{prefix_declarations}\n{current_statement}");
        if let Err(e) = try_parse_trig_fragment(&full_input) {
            result.errors.push(ParseError {
                line: lines.len(),
                position: 1,
                message: format!("Parse error in final statement: {e}"),
            });
        }
    }

    Ok(result)
}

fn try_parse_trig_fragment(
    input: &str,
) -> std::result::Result<(Vec<OwnedTriple>, HashMap<String, String>), TurtleError> {
    let reader = BufReader::new(input.as_bytes());
    let mut parser = TriGParser::new(reader, None);
    let mut triples = Vec::new();

    let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
        let owned_triple = quad_to_owned(&quad);
        triples.push(owned_triple);
        Ok(())
    };

    parser.parse_all(&mut callback)?;

    let mut prefixes = HashMap::new();
    for (prefix, iri) in parser.prefixes().iter() {
        prefixes.insert(prefix.to_string(), iri.to_string());
    }

    Ok((triples, prefixes))
}

/// Parse N-Triples input with robust error handling
pub fn parse_ntriples_robust<R: Read>(
    reader: BufReader<R>,
    continue_on_error: bool,
) -> Result<ParseResult> {
    if continue_on_error {
        parse_ntriples_line_by_line(reader)
    } else {
        // Use the standard parser for strict mode
        let mut parser = NTriplesParser::new(reader);
        let mut result = ParseResult::new();

        let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
            let owned_triple = triple_to_owned(&triple);
            result.triples.push(owned_triple);
            Ok(())
        };

        match parser.parse_all(&mut callback) {
            Ok(_) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

fn parse_ntriples_line_by_line<R: Read>(reader: BufReader<R>) -> Result<ParseResult> {
    let mut result = ParseResult::new();
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num + 1;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Try to parse this single line as N-Triples
        match try_parse_ntriples_line(line) {
            Ok(triple) => {
                result.triples.push(triple);
            }
            Err(e) => {
                result.errors.push(ParseError {
                    line: line_number,
                    position: 1,
                    message: format!("Parse error: {e}"),
                });
            }
        }
    }

    Ok(result)
}

fn try_parse_ntriples_line(line: &str) -> std::result::Result<OwnedTriple, TurtleError> {
    let reader = BufReader::new(line.as_bytes());
    let mut parser = NTriplesParser::new(reader);
    let mut triple_result = None;

    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        triple_result = Some(triple_to_owned(&triple));
        Ok(())
    };

    parser.parse_all(&mut callback)?;

    triple_result.ok_or_else(|| {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No valid triple found in line",
        );
        TurtleError::from(io_error)
    })
}

/// Parse N-Quads input with robust error handling
pub fn parse_nquads_robust<R: Read>(
    reader: BufReader<R>,
    continue_on_error: bool,
) -> Result<ParseResult> {
    if continue_on_error {
        parse_nquads_line_by_line(reader)
    } else {
        // Use the standard parser for strict mode
        let mut parser = NQuadsParser::new(reader);
        let mut result = ParseResult::new();

        let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
            let owned_triple = quad_to_owned(&quad);
            result.triples.push(owned_triple);
            Ok(())
        };

        match parser.parse_all(&mut callback) {
            Ok(_) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

fn parse_nquads_line_by_line<R: Read>(reader: BufReader<R>) -> Result<ParseResult> {
    let mut result = ParseResult::new();
    let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num + 1;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Try to parse this single line as N-Quads
        match try_parse_nquads_line(line) {
            Ok(triple) => {
                result.triples.push(triple);
            }
            Err(e) => {
                result.errors.push(ParseError {
                    line: line_number,
                    position: 1,
                    message: format!("Parse error: {e}"),
                });
            }
        }
    }

    Ok(result)
}

fn try_parse_nquads_line(line: &str) -> std::result::Result<OwnedTriple, TurtleError> {
    let reader = BufReader::new(line.as_bytes());
    let mut parser = NQuadsParser::new(reader);
    let mut triple_result = None;

    let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
        triple_result = Some(quad_to_owned(&quad));
        Ok(())
    };

    parser.parse_all(&mut callback)?;

    triple_result.ok_or_else(|| {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No valid quad found in line",
        );
        TurtleError::from(io_error)
    })
}
