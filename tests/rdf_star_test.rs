// Copyright (c) 2025, Lars Wilhelmsen
// All rights reserved.
//
// This source code is licensed under the BSD-3-Clause license found in the
// LICENSE file in the root directory of this source tree.

use rdfless::{parse_for_estimation, InputFormat};
use std::io::{BufReader, Cursor};

#[test]
fn test_rdf_star_turtle_parsing() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:alice foaf:name "Alice" .
<< ex:alice foaf:name "Alice" >> ex:source ex:wikipedia .
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let result = parse_for_estimation(reader, InputFormat::Turtle);

    if let Err(e) = &result {
        eprintln!("Parse error: {e}");
    }

    assert!(result.is_ok());
    let (triples, prefixes) = result.unwrap();

    // Check that we parsed the expected number of triples
    assert_eq!(triples.len(), 2);

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

    // Find the RDF-star triple (the one with embedded triple as subject)
    let rdf_star_triple = triples
        .iter()
        .find(|t| t.subject_type == rdfless::SubjectType::Triple);
    assert!(rdf_star_triple.is_some());

    let rdf_star_triple = rdf_star_triple.unwrap();
    assert_eq!(rdf_star_triple.predicate, "https://example.org/source");
    assert_eq!(
        rdf_star_triple.object_value,
        "https://example.org/wikipedia"
    );

    // Verify that the embedded triple information is preserved
    assert!(rdf_star_triple.subject_triple.is_some());
    let embedded_triple = rdf_star_triple.subject_triple.as_ref().unwrap();
    assert_eq!(embedded_triple.subject_value, "https://example.org/alice");
    assert_eq!(embedded_triple.predicate, "http://xmlns.com/foaf/0.1/name");
    assert_eq!(embedded_triple.object_value, "Alice");
}

#[test]
fn test_rdf_star_trig_parsing() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

{
    ex:alice foaf:name "Alice" .
    << ex:alice foaf:name "Alice" >> ex:source ex:wikipedia .
}
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let result = parse_for_estimation(reader, InputFormat::TriG);

    assert!(result.is_ok());
    let (triples, prefixes) = result.unwrap();

    // Check that we parsed the expected number of triples
    assert_eq!(triples.len(), 2);

    // Check that we extracted the expected prefixes
    assert_eq!(prefixes.len(), 2);

    // Find the RDF-star triple
    let rdf_star_triple = triples
        .iter()
        .find(|t| t.subject_type == rdfless::SubjectType::Triple);
    assert!(rdf_star_triple.is_some());
}

#[test]
fn test_rdf_star_object_position() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:alice foaf:name "Alice" .
ex:statement ex:about << ex:alice foaf:name "Alice" >> .
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let result = parse_for_estimation(reader, InputFormat::Turtle);

    assert!(result.is_ok());
    let (triples, _) = result.unwrap();

    // Check that we parsed the expected number of triples
    assert_eq!(triples.len(), 2);

    // Find the RDF-star triple (the one with embedded triple as object)
    let rdf_star_triple = triples
        .iter()
        .find(|t| t.object_type == rdfless::ObjectType::Triple);
    assert!(rdf_star_triple.is_some());

    let rdf_star_triple = rdf_star_triple.unwrap();
    assert_eq!(
        rdf_star_triple.subject_value,
        "https://example.org/statement"
    );
    assert_eq!(rdf_star_triple.predicate, "https://example.org/about");

    // Verify that the embedded triple information is preserved
    assert!(rdf_star_triple.object_triple.is_some());
    let embedded_triple = rdf_star_triple.object_triple.as_ref().unwrap();
    assert_eq!(embedded_triple.subject_value, "https://example.org/alice");
    assert_eq!(embedded_triple.predicate, "http://xmlns.com/foaf/0.1/name");
    assert_eq!(embedded_triple.object_value, "Alice");
}
