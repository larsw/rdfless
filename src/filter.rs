// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use crate::types::OwnedTriple;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TripleFilter {
    pub subject_filter: Option<String>,
    pub predicate_filter: Option<String>,
    pub object_filter: Option<String>,
}

impl TripleFilter {
    pub fn new(
        subject_filter: Option<&str>,
        predicate_filter: Option<&str>,
        object_filter: Option<&str>,
    ) -> Self {
        TripleFilter {
            subject_filter: subject_filter.map(|s| s.to_string()),
            predicate_filter: predicate_filter.map(|s| s.to_string()),
            object_filter: object_filter.map(|s| s.to_string()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.subject_filter.is_none()
            && self.predicate_filter.is_none()
            && self.object_filter.is_none()
    }

    /// Filter triples based on the configured criteria
    pub fn filter_triples(
        &self,
        triples: &[OwnedTriple],
        prefixes: &HashMap<String, String>,
    ) -> Vec<OwnedTriple> {
        if self.is_empty() {
            return triples.to_vec();
        }

        triples
            .iter()
            .filter(|triple| self.matches_triple(triple, prefixes))
            .cloned()
            .collect()
    }

    fn matches_triple(&self, triple: &OwnedTriple, prefixes: &HashMap<String, String>) -> bool {
        // Check subject filter
        if let Some(ref subject_filter) = self.subject_filter {
            if !self.matches_subject(triple, subject_filter, prefixes) {
                return false;
            }
        }

        // Check predicate filter
        if let Some(ref predicate_filter) = self.predicate_filter {
            if !self.matches_predicate(triple, predicate_filter, prefixes) {
                return false;
            }
        }

        // Check object filter
        if let Some(ref object_filter) = self.object_filter {
            if !self.matches_object(triple, object_filter, prefixes) {
                return false;
            }
        }

        true
    }

    fn matches_subject(
        &self,
        triple: &OwnedTriple,
        filter: &str,
        prefixes: &HashMap<String, String>,
    ) -> bool {
        let expanded_filter = self.expand_prefixed_name(filter, prefixes);

        // Direct IRI match
        if triple.subject_value == expanded_filter {
            return true;
        }

        // If filter contains a colon and matches a prefixed form
        if filter.contains(':') && !filter.starts_with("http") {
            return triple.subject_value == expanded_filter;
        }

        false
    }

    fn matches_predicate(
        &self,
        triple: &OwnedTriple,
        filter: &str,
        prefixes: &HashMap<String, String>,
    ) -> bool {
        let expanded_filter = self.expand_prefixed_name(filter, prefixes);

        // Direct IRI match
        if triple.predicate == expanded_filter {
            return true;
        }

        // If filter contains a colon and matches a prefixed form
        if filter.contains(':') && !filter.starts_with("http") {
            return triple.predicate == expanded_filter;
        }

        false
    }

    fn matches_object(
        &self,
        triple: &OwnedTriple,
        filter: &str,
        prefixes: &HashMap<String, String>,
    ) -> bool {
        // For literals, check direct value match
        if triple.object_value == filter {
            return true;
        }

        // For IRIs, check expanded form
        let expanded_filter = self.expand_prefixed_name(filter, prefixes);
        if triple.object_value == expanded_filter {
            return true;
        }

        // If filter contains a colon and matches a prefixed form
        if filter.contains(':') && !filter.starts_with("http") {
            return triple.object_value == expanded_filter;
        }

        false
    }

    fn expand_prefixed_name(&self, name: &str, prefixes: &HashMap<String, String>) -> String {
        if name.starts_with('<') && name.ends_with('>') {
            // Already a full IRI in angle brackets, remove them
            return name[1..name.len() - 1].to_string();
        }

        if name.starts_with("http") {
            // Already a full IRI
            return name.to_string();
        }

        if let Some(colon_pos) = name.find(':') {
            let prefix = &name[..colon_pos];
            let local_name = &name[colon_pos + 1..];

            if let Some(namespace) = prefixes.get(prefix) {
                return format!("{namespace}{local_name}");
            }
        }

        // Return as-is if no prefix expansion possible
        name.to_string()
    }
}

impl Clone for OwnedTriple {
    fn clone(&self) -> Self {
        OwnedTriple {
            subject_type: self.subject_type.clone(),
            subject_value: self.subject_value.clone(),
            predicate: self.predicate.clone(),
            object_type: self.object_type.clone(),
            object_value: self.object_value.clone(),
            object_datatype: self.object_datatype.clone(),
            object_language: self.object_language.clone(),
            graph: self.graph.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ObjectType, SubjectType};

    fn create_test_triple(subject: &str, predicate: &str, object: &str) -> OwnedTriple {
        OwnedTriple {
            subject_type: SubjectType::NamedNode,
            subject_value: subject.to_string(),
            predicate: predicate.to_string(),
            object_type: ObjectType::Literal,
            object_value: object.to_string(),
            object_datatype: None,
            object_language: None,
            graph: None,
        }
    }

    #[test]
    fn test_empty_filter() {
        let filter = TripleFilter::new(None, None, None);
        assert!(filter.is_empty());

        let triples = vec![create_test_triple(
            "https://example.org/subject",
            "https://example.org/predicate",
            "object",
        )];
        let prefixes = HashMap::new();

        let filtered = filter.filter_triples(&triples, &prefixes);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_subject_filter() {
        let filter = TripleFilter::new(Some("https://example.org/subject"), None, None);
        let triples = vec![
            create_test_triple(
                "https://example.org/subject",
                "https://example.org/predicate",
                "object1",
            ),
            create_test_triple(
                "https://example.org/other",
                "https://example.org/predicate",
                "object2",
            ),
        ];
        let prefixes = HashMap::new();

        let filtered = filter.filter_triples(&triples, &prefixes);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].object_value, "object1");
    }

    #[test]
    fn test_prefixed_filter() {
        let mut prefixes = HashMap::new();
        prefixes.insert("ex".to_string(), "https://example.org/".to_string());

        let filter = TripleFilter::new(Some("ex:subject"), None, None);
        let triples = vec![
            create_test_triple(
                "https://example.org/subject",
                "https://example.org/predicate",
                "object1",
            ),
            create_test_triple(
                "https://example.org/other",
                "https://example.org/predicate",
                "object2",
            ),
        ];

        let filtered = filter.filter_triples(&triples, &prefixes);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].object_value, "object1");
    }

    #[test]
    fn test_multiple_filters() {
        let mut prefixes = HashMap::new();
        prefixes.insert("ex".to_string(), "https://example.org/".to_string());

        let filter = TripleFilter::new(Some("ex:subject"), Some("ex:predicate"), Some("target"));
        let triples = vec![
            create_test_triple(
                "https://example.org/subject",
                "https://example.org/predicate",
                "target",
            ),
            create_test_triple(
                "https://example.org/subject",
                "https://example.org/predicate",
                "other",
            ),
            create_test_triple(
                "https://example.org/other",
                "https://example.org/predicate",
                "target",
            ),
        ];

        let filtered = filter.filter_triples(&triples, &prefixes);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].object_value, "target");
    }
}
