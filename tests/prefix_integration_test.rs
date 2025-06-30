// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use rdfless::{parse_for_estimation, InputFormat};
use std::io::{BufReader, Cursor};

#[test]
fn test_turtle_with_prefix_integration() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:person1 a foaf:Person .
ex:person1 foaf:name "Alice" .
ex:person2 a foaf:Person .
ex:person2 foaf:name "Bob" .
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let result = parse_for_estimation(reader, InputFormat::Turtle);

    assert!(result.is_ok());
    let (triples, prefixes) = result.unwrap();

    // Check that we parsed the expected number of triples
    assert_eq!(triples.len(), 4);

    // Check that we extracted the expected prefixes
    assert_eq!(prefixes.len(), 3);
    assert_eq!(
        prefixes.get("ex"),
        Some(&"https://example.org/".to_string())
    );
    assert_eq!(
        prefixes.get("foaf"),
        Some(&"http://xmlns.com/foaf/0.1/".to_string())
    );
    assert_eq!(
        prefixes.get("rdf"),
        Some(&"http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string())
    );
}

#[test]
fn test_trig_with_prefix_integration() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

{
    ex:person1 a foaf:Person .
    ex:person1 foaf:name "Alice" .
}

ex:graph2 {
    ex:person2 a foaf:Person .
    ex:person2 foaf:name "Bob" .
}
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let result = parse_for_estimation(reader, InputFormat::TriG);

    assert!(result.is_ok());
    let (triples, prefixes) = result.unwrap();

    // Check that we parsed the expected number of triples (quads)
    assert_eq!(triples.len(), 4);

    // Check that we extracted the expected prefixes
    assert_eq!(prefixes.len(), 2);
    assert_eq!(
        prefixes.get("ex"),
        Some(&"https://example.org/".to_string())
    );
    assert_eq!(
        prefixes.get("foaf"),
        Some(&"http://xmlns.com/foaf/0.1/".to_string())
    );

    // Check that graph information is preserved
    let graph_triples: Vec<_> = triples.iter().filter(|t| t.graph.is_some()).collect();
    assert_eq!(graph_triples.len(), 2); // Two triples in named graph
}

#[test]
fn test_ntriples_no_prefixes() {
    let input = r#"
<https://example.org/person1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<https://example.org/person1> <http://xmlns.com/foaf/0.1/name> "Alice" .
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let result = parse_for_estimation(reader, InputFormat::NTriples);

    assert!(result.is_ok());
    let (triples, prefixes) = result.unwrap();

    // Check that we parsed the expected number of triples
    assert_eq!(triples.len(), 2);

    // N-Triples doesn't support prefixes, so should be empty
    assert!(prefixes.is_empty());
}
