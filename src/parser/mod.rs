// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

pub mod turtle;
pub mod trig;
pub mod ntriples;
pub mod nquads;
pub mod common;

use crate::types::{InputFormat, OwnedTriple};
use anyhow::Result;
use std::collections::HashMap;
use std::io::{BufReader, Read};

/// Parse input and return triples and prefixes for estimation or processing
pub fn parse_for_estimation<R: Read>(
    reader: BufReader<R>,
    format: InputFormat,
) -> Result<(Vec<OwnedTriple>, HashMap<String, String>)> {
    match format {
        InputFormat::Turtle => turtle::parse_for_estimation(reader),
        InputFormat::TriG => trig::parse_for_estimation(reader),
        InputFormat::NTriples => ntriples::parse_for_estimation(reader),
        InputFormat::NQuads => nquads::parse_for_estimation(reader),
    }
}
