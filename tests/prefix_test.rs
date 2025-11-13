// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use rdfless::extract_prefixes;
use rdfless::parse_for_estimation;
use rdfless::InputFormat;
use std::collections::HashMap;
use std::io::{BufReader, Cursor};

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

#[test]
fn test_extract_relative_prefix_sparql_syntax() {
    let input = r#"
PREFIX : <#>
PREFIX ex: <https://example.org/>

ex:Person a :Person .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("".to_string(), "#".to_string());
    expected.insert("ex".to_string(), "https://example.org/".to_string());

    assert_eq!(prefixes, expected);
}

#[test]
fn test_extract_relative_prefix_turtle_syntax() {
    let input = r#"
@prefix : <#> .
@prefix ex: <https://example.org/> .

ex:Person a :Person .
"#;

    let prefixes = extract_prefixes(Cursor::new(input.as_bytes()));

    let mut expected = HashMap::new();
    expected.insert("".to_string(), "#".to_string());
    expected.insert("ex".to_string(), "https://example.org/".to_string());

    assert_eq!(prefixes, expected);
}

#[test]
fn test_parse_turtle_with_relative_prefix_sparql_syntax() {
    let input = r#"
PREFIX : <#>
PREFIX ex: <https://example.org/>

ex:Person a :Person .
"#;

    // This should work once we add base IRI support
    let reader = BufReader::new(input.as_bytes());
    let result = parse_for_estimation(reader, InputFormat::Turtle);

    // Currently this fails with "No scheme found in an absolute IRI"
    // After the fix, it should succeed
    assert!(
        result.is_ok(),
        "Failed to parse Turtle with relative prefix IRI: {:?}",
        result.err()
    );

    let (triples, _prefixes) = result.unwrap();
    assert_eq!(triples.len(), 1);
}

#[test]
fn test_parse_turtle_with_relative_prefix_turtle_syntax() {
    let input = r#"
@prefix : <#> .
@prefix ex: <https://example.org/> .

ex:Person a :Person .
"#;

    // This should work once we add base IRI support
    let reader = BufReader::new(input.as_bytes());
    let result = parse_for_estimation(reader, InputFormat::Turtle);

    // Currently this fails with "No scheme found in an absolute IRI"
    // After the fix, it should succeed
    assert!(
        result.is_ok(),
        "Failed to parse Turtle with relative prefix IRI: {:?}",
        result.err()
    );

    let (triples, _prefixes) = result.unwrap();
    assert_eq!(triples.len(), 1);
}
