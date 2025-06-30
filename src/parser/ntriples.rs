// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::parser::common::triple_to_owned;
use crate::types::OwnedTriple;
use anyhow::Result;
use oxttl::NTriplesParser;
use std::collections::HashMap;
use std::io::{BufReader, Read};

/// Parse N-Triples input and return triples for estimation
pub fn parse_for_estimation<R: Read>(
    reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    let parser = NTriplesParser::new().for_reader(reader);
    let mut triples = Vec::new();

    // Process each triple
    for result in parser {
        let triple = result?;
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    // N-Triples doesn't have prefixes
    Ok((triples, HashMap::new()))
}
