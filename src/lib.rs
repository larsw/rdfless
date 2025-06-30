// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

//! # rdfless
//!
//! A command-line tool for parsing, filtering, and formatting RDF data with colored output.
//! This crate is primarily intended as a CLI application, not as a reusable library.

// Internal modules - not exposed as public API
pub(crate) mod config;
pub(crate) mod filter;
pub(crate) mod formatter;
pub(crate) mod pager;
pub(crate) mod parser;
pub(crate) mod types;
pub(crate) mod utils;

// Minimal public API - only what's needed by main.rs and tests
pub use config::{
    get_effective_colors, load_config, string_to_color, ColorConfig, Config, OutputConfig,
    ThemeConfig,
};
pub use filter::TripleFilter;
pub use formatter::writer::{format_object, format_predicate, format_subject};
pub use formatter::{estimate_output_lines, render_output};
pub use pager::should_use_pager;
pub use parser::common::{extract_prefixes, quad_to_owned, triple_to_owned};
pub use parser::robust::{ParseError, ParseResult};
pub use parser::{parse_for_estimation, parse_robust};
pub use types::{
    detect_format_from_path, ArgsConfig, InputFormat, ObjectType, OwnedTriple, SubjectType,
};
pub use utils::get_terminal_height;

// Legacy function aliases for tests
pub use format_object as format_owned_object;
pub use format_predicate as format_owned_predicate;
pub use format_subject as format_owned_subject;

// Test-only function for legacy test compatibility
pub fn process_input<R: std::io::Read, A: ArgsConfig>(
    reader: std::io::BufReader<R>,
    args: &A,
    colors: &config::ColorConfig,
    config: &config::Config,
) -> anyhow::Result<String> {
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
