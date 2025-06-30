// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::config;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFormat {
    Turtle,
    TriG,
    NTriples,
    NQuads,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubjectType {
    NamedNode,
    BlankNode,
    Triple, // RDF-star: embedded triple as subject
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    NamedNode,
    BlankNode,
    Literal,
    Triple, // RDF-star: embedded triple as object
}

#[derive(Debug)]
pub struct OwnedTriple {
    pub subject_type: SubjectType,
    pub subject_value: String,
    pub predicate: String,
    pub object_type: ObjectType,
    pub object_value: String,
    pub object_datatype: Option<String>,
    pub object_language: Option<String>,
    pub graph: Option<String>,
    // RDF-star support: embedded triples
    pub subject_triple: Option<Box<OwnedTriple>>, // For embedded triple as subject
    pub object_triple: Option<Box<OwnedTriple>>,  // For embedded triple as object
}

/// Define a trait for the Args interface
pub trait ArgsConfig {
    /// Determine if prefixes should be expanded based on args and config
    fn expand(&self, config: &config::Config) -> bool;

    /// Get the input format (either specified by user or detected from file extension)
    fn format(&self) -> Option<InputFormat>;

    /// Determine if paging should be used based on args and config (explicit user choice)
    fn use_pager(&self, config: &config::Config) -> bool;

    /// Check if user explicitly disabled paging
    fn no_pager_explicit(&self) -> bool;

    /// Get the effective color configuration
    fn get_colors(&self, config: &config::Config) -> config::ColorConfig;

    /// Check if output is going to a file
    fn is_output_to_file(&self) -> bool;

    /// Check if parsing should continue on errors
    fn continue_on_error(&self) -> bool;

    /// Get filter criteria for subject
    fn filter_subject(&self) -> Option<&str>;

    /// Get filter criteria for predicate
    fn filter_predicate(&self) -> Option<&str>;

    /// Get filter criteria for object
    fn filter_object(&self) -> Option<&str>;
}

/// Helper function to detect format from file extension
pub fn detect_format_from_path(path: &Path) -> Option<InputFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "ttl" => InputFormat::Turtle,
            "trig" => InputFormat::TriG,
            "nt" => InputFormat::NTriples,
            "nq" => InputFormat::NQuads,
            _ => InputFormat::Turtle, // Default to Turtle for unknown extensions
        })
}
