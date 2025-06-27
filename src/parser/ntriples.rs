// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::parser::common::triple_to_owned;
use crate::types::OwnedTriple;
use anyhow::Result;
use rio_api::model::Triple;
use rio_api::parser::TriplesParser;
use rio_turtle::{NTriplesParser, TurtleError};
use std::collections::HashMap;
use std::io::{BufReader, Read};

/// Parse N-Triples input and return triples for estimation
pub fn parse_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let mut parser = NTriplesParser::new(reader);
    let mut triples = Vec::new();

    // Process each triple
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    // Parse all triples
    parser.parse_all(&mut callback)?;

    // N-Triples doesn't have prefixes
    Ok((triples, HashMap::new()))
}
