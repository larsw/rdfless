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

    // With RDF 1.2, quoted triple usage is represented via a reifier blank node + rdf:reifies
    // Hence we expect 3 triples: the base triple, the reifier mapping, and the linking triple
    assert_eq!(triples.len(), 3);

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

    // Find the rdf:reifies mapping triple _:r rdf:reifies << ex:alice foaf:name "Alice" >>
    let reifies_iri = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
    let reifies = triples.iter().find(|t| t.predicate == reifies_iri);
    assert!(reifies.is_some());
    let reifies = reifies.unwrap();
    assert_eq!(reifies.subject_type, rdfless::SubjectType::BlankNode);
    assert_eq!(reifies.object_type, rdfless::ObjectType::Triple);
    let embedded = reifies.object_triple.as_ref().unwrap();
    assert_eq!(embedded.subject_value, "https://example.org/alice");
    assert_eq!(embedded.predicate, "http://xmlns.com/foaf/0.1/name");
    assert_eq!(embedded.object_value, "Alice");

    // And the linking triple uses the same reifier as subject: _:r ex:source ex:wikipedia
    let link = triples.iter().find(|t| {
        t.subject_type == rdfless::SubjectType::BlankNode
            && t.subject_value == reifies.subject_value
            && t.predicate == "https://example.org/source"
            && t.object_value == "https://example.org/wikipedia"
    });
    assert!(link.is_some());
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

    // Expect 3 triples with RDF 1.2 reification mapping
    assert_eq!(triples.len(), 3);

    // Check that we extracted the expected prefixes
    assert_eq!(prefixes.len(), 2);

    // Find reifier mapping
    let reifies_iri = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
    let reifies = triples.iter().find(|t| t.predicate == reifies_iri);
    assert!(reifies.is_some());
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

    // Expect 3 triples with RDF 1.2 reification mapping
    assert_eq!(triples.len(), 3);

    // Find the rdf:reifies mapping triple _:r rdf:reifies << ex:alice foaf:name "Alice" >>
    let reifies_iri = "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies";
    let reifies = triples.iter().find(|t| t.predicate == reifies_iri).unwrap();
    assert_eq!(reifies.object_type, rdfless::ObjectType::Triple);
    let embedded = reifies.object_triple.as_ref().unwrap();
    assert_eq!(embedded.subject_value, "https://example.org/alice");
    assert_eq!(embedded.predicate, "http://xmlns.com/foaf/0.1/name");
    assert_eq!(embedded.object_value, "Alice");

    // And the main triple points to the reifier as object: ex:statement ex:about _:r
    let link = triples.iter().find(|t| {
        t.subject_value == "https://example.org/statement"
            && t.predicate == "https://example.org/about"
            && t.object_type == rdfless::ObjectType::BlankNode
            && t.object_value == reifies.subject_value
    });
    assert!(link.is_some());
}
