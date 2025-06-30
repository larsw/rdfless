// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use rdfless::extract_prefixes;
use std::collections::HashMap;
use std::io::Cursor;

#[test]
fn test_extract_turtle_prefixes() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:Person a foaf:Person .
ex:Person foaf:name "John Doe" .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("ex".to_string(), "https://example.org/".to_string());
    expected.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());
    expected.insert(
        "rdf".to_string(),
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
    );

    assert_eq!(prefixes, expected);
}

#[test]
fn test_extract_sparql_prefixes() {
    let input = r#"
PREFIX ex: <https://example.org/>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

ex:Person a foaf:Person .
ex:Person foaf:name "John Doe" .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("ex".to_string(), "https://example.org/".to_string());
    expected.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());
    expected.insert(
        "rdf".to_string(),
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
    );

    assert_eq!(prefixes, expected);
}

#[test]
fn test_extract_mixed_prefixes() {
    let input = r#"
@prefix ex: <https://example.org/> .
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:Person a foaf:Person .
ex:Person foaf:name "John Doe" .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("ex".to_string(), "https://example.org/".to_string());
    expected.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());
    expected.insert(
        "rdf".to_string(),
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
    );

    assert_eq!(prefixes, expected);
}

#[test]
fn test_extract_base_declaration() {
    let input = r#"
@base <https://example.org/> .
@prefix ex: <person/> .

ex:Person a <type/Person> .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("".to_string(), "https://example.org/".to_string()); // Base IRI stored with empty prefix
    expected.insert("ex".to_string(), "person/".to_string());

    assert_eq!(prefixes, expected);
}

#[test]
fn test_extract_prefixes_with_comments() {
    let input = r#"
# This is a comment
@prefix ex: <https://example.org/> .
# Another comment
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:Person a foaf:Person .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("ex".to_string(), "https://example.org/".to_string());
    expected.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());

    assert_eq!(prefixes, expected);
}

#[test]
fn test_extract_prefixes_empty_input() {
    let input = "";
    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));
    assert!(prefixes.is_empty());
}

#[test]
fn test_extract_prefixes_no_prefixes() {
    let input = r#"
<https://example.org/person> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));
    assert!(prefixes.is_empty());
}
