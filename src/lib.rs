// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use anyhow::Result;
use std::io::{BufReader, Read};

pub mod config;
pub mod types;
pub mod utils;
pub mod parser;
pub mod formatter;
pub mod pager;

// Re-export commonly used types
pub use types::{ArgsConfig, InputFormat, OwnedTriple, SubjectType, ObjectType, detect_format_from_path};
pub use parser::parse_for_estimation;
pub use parser::common::{triple_to_owned, quad_to_owned};
pub use formatter::{render_output, estimate_output_lines};
pub use formatter::writer::{format_subject, format_predicate, format_object};
pub use pager::{should_use_pager, process_with_auto_pager};
pub use utils::get_terminal_height;

// Keep the old function names for backward compatibility
pub use parser::turtle::parse_for_estimation as parse_turtle_for_estimation;
pub use parser::trig::parse_for_estimation as parse_trig_for_estimation;
pub use parser::ntriples::parse_for_estimation as parse_ntriples_for_estimation;
pub use parser::nquads::parse_for_estimation as parse_nquads_for_estimation;

// Legacy function names for backward compatibility
pub use format_subject as format_owned_subject;
pub use format_predicate as format_owned_predicate;
pub use format_object as format_owned_object;

// Legacy process_input function for tests
pub fn process_input<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> Result<String> {
    let format = args.format().unwrap_or(InputFormat::Turtle);
    let (triples, prefixes) = parse_for_estimation(reader, format)?;
    
    let mut output = Vec::new();
    formatter::render_output(
        &triples,
        &prefixes,
        args.expand(config),
        colors,
        &mut output,
    )?;
    
    Ok(String::from_utf8(output)?)
}

// Legacy functions that are no longer needed but kept for compatibility
pub fn process_input_auto_pager<R: Read, A: ArgsConfig>(
    reader: BufReader<R>,
    args: &A,
    config: &config::Config,
) -> Result<()> {
    let format = args.format().unwrap_or(InputFormat::Turtle);
    let (triples, prefixes) = parse_for_estimation(reader, format)?;
    process_with_auto_pager(&triples, &prefixes, args, config)
}
