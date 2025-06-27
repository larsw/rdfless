// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::parser::common::quad_to_owned;
use crate::types::OwnedTriple;
use anyhow::Result;
use rio_api::model::Quad;
use rio_api::parser::QuadsParser;
use rio_turtle::{TriGParser, TurtleError};
use std::collections::HashMap;
use std::io::{BufReader, Read};

/// Parse TriG input and return triples and prefixes for estimation
pub fn parse_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut parser = TriGParser::new(reader, None);
    let mut triples = Vec::new();
    let mut prefixes = HashMap::new();

    // Process each quad
    let mut callback = |quad: Quad| -> std::result::Result<(), TurtleError> {
        let owned_triple = quad_to_owned(&quad);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all quads
    parser.parse_all(&mut callback)?;

    // Get prefixes from parser
    for (prefix, iri) in parser.prefixes().iter() {
        prefixes.insert(prefix.to_string(), iri.to_string());
    }

    Ok((triples, prefixes))
}
