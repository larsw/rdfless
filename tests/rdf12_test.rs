use rdfless::{parse_for_estimation, InputFormat};
use std::fs::File;
use std::io::{BufReader, Cursor, Read};

#[test]
fn test_version_directives_supported() {
    // VERSION "1.2"
    let mut f1 = File::open("tests/fixtures/test_version_directive.ttl").unwrap();
    let mut s1 = String::new();
    f1.read_to_string(&mut s1).unwrap();
    let reader1 = BufReader::new(Cursor::new(s1.as_bytes()));
    let res1 = parse_for_estimation(reader1, InputFormat::Turtle).unwrap();
    assert_eq!(res1.0.len(), 1);

    // @version "1.2"
    let mut f2 = File::open("tests/fixtures/test_at_version_directive.ttl").unwrap();
    let mut s2 = String::new();
    f2.read_to_string(&mut s2).unwrap();
    let reader2 = BufReader::new(Cursor::new(s2.as_bytes()));
    let res2 = parse_for_estimation(reader2, InputFormat::Turtle).unwrap();
    assert_eq!(res2.0.len(), 1);
}

#[test]
fn test_parenthesized_triple_term_object() {
    let input = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:alice foaf:name "Alice" .
ex:statement ex:about <<( ex:alice foaf:name "Alice" )>> .
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let (triples, _prefixes) = parse_for_estimation(reader, InputFormat::Turtle).unwrap();

    // Expect 2 triples: base triple and the triple-term usage (no implicit reifier mapping)
    assert_eq!(triples.len(), 2);

    let tt = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/about")
        .unwrap();
    assert_eq!(tt.subject_value, "https://example.org/statement");
    assert_eq!(tt.object_type, rdfless::ObjectType::Triple);
    let embedded = tt.object_triple.as_ref().unwrap();
    assert_eq!(embedded.subject_value, "https://example.org/alice");
    assert_eq!(embedded.predicate, "http://xmlns.com/foaf/0.1/name");
    assert_eq!(embedded.object_value, "Alice");
}

#[test]
fn test_rdf12_annotation_with_named_reifier() {
    let input = r#"
@prefix : <http://example/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:a :name "Alice" ~ :t {| :statedBy :bob ; :recorded "2021-07-07"^^xsd:date |} .
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let (triples, _prefixes) = parse_for_estimation(reader, InputFormat::Turtle).unwrap();

    // Expect 4 triples:
    // 1) base data triple (:a :name "Alice")
    // 2) :t rdf:reifies << :a :name "Alice" >>
    // 3) :t :statedBy :bob
    // 4) :t :recorded "2021-07-07"^^xsd:date
    assert_eq!(triples.len(), 4);

    // Base triple
    assert!(triples.iter().any(|t| {
        t.subject_value == "http://example/a"
            && t.predicate == "http://example/name"
            && t.object_type == rdfless::ObjectType::Literal
            && t.object_value == "Alice"
    }));

    // Reifier mapping
    let reif = triples
        .iter()
        .find(|t| {
            t.subject_value == "http://example/t"
                && t.predicate == "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies"
        })
        .expect("reifier triple not found");
    assert_eq!(reif.object_type, rdfless::ObjectType::Triple);
    let emb = reif
        .object_triple
        .as_ref()
        .expect("embedded triple missing");
    assert_eq!(emb.subject_value, "http://example/a");
    assert_eq!(emb.predicate, "http://example/name");
    assert_eq!(emb.object_value, "Alice");

    // Annotation facts
    assert!(triples.iter().any(|t| {
        t.subject_value == "http://example/t"
            && t.predicate == "http://example/statedBy"
            && t.object_type == rdfless::ObjectType::NamedNode
            && t.object_value == "http://example/bob"
    }));

    let rec = triples
        .iter()
        .find(|t| t.subject_value == "http://example/t" && t.predicate == "http://example/recorded")
        .expect(":recorded triple not found");
    assert_eq!(rec.object_value, "2021-07-07");
    assert_eq!(
        rec.object_datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#date")
    );
}

#[test]
fn test_trig_annotation_with_named_reifier_in_graph() {
    let input = r#"
@prefix : <http://example/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:g {
  :a :name "Alice" ~ :t {| :statedBy :bob ; :recorded "2021-07-07"^^xsd:date |} .
}
"#;

    let reader = BufReader::new(Cursor::new(input.as_bytes()));
    let (triples, _prefixes) = parse_for_estimation(reader, InputFormat::TriG).unwrap();

    // Expect 4 triples in graph :g
    assert_eq!(triples.len(), 4);

    // All triples should be tagged with the same graph
    for t in &triples {
        assert_eq!(t.graph.as_deref(), Some("http://example/g"));
    }

    // Base triple
    assert!(triples.iter().any(|t| {
        t.subject_value == "http://example/a"
            && t.predicate == "http://example/name"
            && t.object_type == rdfless::ObjectType::Literal
            && t.object_value == "Alice"
            && t.graph.as_deref() == Some("http://example/g")
    }));

    // Reifier mapping
    let reif = triples
        .iter()
        .find(|t| {
            t.subject_value == "http://example/t"
                && t.predicate == "http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies"
        })
        .expect("reifier triple not found");
    assert_eq!(reif.graph.as_deref(), Some("http://example/g"));
    assert_eq!(reif.object_type, rdfless::ObjectType::Triple);
    let emb = reif
        .object_triple
        .as_ref()
        .expect("embedded triple missing");
    assert_eq!(emb.subject_value, "http://example/a");
    assert_eq!(emb.predicate, "http://example/name");
    assert_eq!(emb.object_value, "Alice");

    // Annotation facts
    assert!(triples.iter().any(|t| {
        t.subject_value == "http://example/t"
            && t.predicate == "http://example/statedBy"
            && t.object_type == rdfless::ObjectType::NamedNode
            && t.object_value == "http://example/bob"
            && t.graph.as_deref() == Some("http://example/g")
    }));

    let rec = triples
        .iter()
        .find(|t| t.subject_value == "http://example/t" && t.predicate == "http://example/recorded")
        .expect(":recorded triple not found");
    assert_eq!(rec.graph.as_deref(), Some("http://example/g"));
    assert_eq!(rec.object_value, "2021-07-07");
    assert_eq!(
        rec.object_datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#date")
    );
}
