// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::parser::common::{extract_prefixes, quad_to_owned};
use crate::types::OwnedTriple;
use anyhow::Result;
use oxttl::TriGParser;
use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read};

/// Parse TriG input and return triples and prefixes for estimation
pub fn parse_for_estimation<R: Read>(
    mut reader: BufReader<R>,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    // Read the entire input to extract prefixes and parse quads
    let mut content = Vec::new();
    reader.read_to_end(&mut content)?; // Extract prefixes first
    let prefixes = extract_prefixes(Cursor::new(&content));

    // Then parse quads (RDF 1.2 quoted triples enabled by feature)
    // Set a default base IRI to allow relative IRI references in prefixes
    let parser = TriGParser::new()
        .with_base_iri("http://example.org/")?
        .for_reader(Cursor::new(&content));
    let mut triples = Vec::new();

    // Process each quad
    for result in parser {
        let quad = result?;
        let owned_triple = quad_to_owned(&quad);
        triples.push(owned_triple);
    }

    Ok((triples, prefixes))
}
