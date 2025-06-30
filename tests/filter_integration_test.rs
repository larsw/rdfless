use rdfless::{ObjectType, OwnedTriple, SubjectType, TripleFilter};
use rstest::rstest;
use std::collections::HashMap;

fn create_test_triple(
    subject: &str,
    predicate: &str,
    object: &str,
    object_type: ObjectType,
) -> OwnedTriple {
    OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: subject.to_string(),
        predicate: predicate.to_string(),
        object_type,
        object_value: object.to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
        subject_triple: None,
        object_triple: None,
    }
}

#[rstest]
fn test_filter_integration_subject_iri() {
    let triples = vec![
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/name",
            "Alice",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/bob",
            "http://xmlns.com/foaf/0.1/name",
            "Bob",
            ObjectType::Literal,
        ),
    ];

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "https://example.org/".to_string());
    prefixes.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());

    // Test filtering by full IRI
    let filter = TripleFilter::new(Some("https://example.org/alice"), None, None);
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].object_value, "Alice");
}

#[rstest]
fn test_filter_integration_prefixed_names() {
    let triples = vec![
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/name",
            "Alice",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/age",
            "30",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/bob",
            "http://xmlns.com/foaf/0.1/name",
            "Bob",
            ObjectType::Literal,
        ),
    ];

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "https://example.org/".to_string());
    prefixes.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());

    // Test filtering by prefixed subject and predicate
    let filter = TripleFilter::new(Some("ex:alice"), Some("foaf:name"), None);
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].object_value, "Alice");
}

#[rstest]
fn test_filter_integration_object_iri() {
    let triples = vec![
        create_test_triple(
            "https://example.org/alice",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/Person",
            ObjectType::NamedNode,
        ),
        create_test_triple(
            "https://example.org/bob",
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            "https://example.org/Robot",
            ObjectType::NamedNode,
        ),
    ];

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "https://example.org/".to_string());
    prefixes.insert(
        "rdf".to_string(),
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
    );

    // Test filtering by object IRI using prefixed name
    let filter = TripleFilter::new(None, None, Some("ex:Person"));
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].subject_value, "https://example.org/alice");
}

#[rstest]
fn test_filter_integration_literal_values() {
    let triples = vec![
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/name",
            "Alice Smith",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/bob",
            "http://xmlns.com/foaf/0.1/name",
            "Bob Jones",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/charlie",
            "http://xmlns.com/foaf/0.1/name",
            "Alice Brown",
            ObjectType::Literal,
        ),
    ];

    let prefixes = HashMap::new();

    // Test filtering by exact literal match
    let filter = TripleFilter::new(None, None, Some("Alice Smith"));
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].subject_value, "https://example.org/alice");
}

#[rstest]
fn test_filter_integration_complex_combination() {
    let triples = vec![
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/name",
            "Alice",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/age",
            "30",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/bob",
            "http://xmlns.com/foaf/0.1/name",
            "Alice",
            ObjectType::Literal,
        ),
        create_test_triple(
            "https://example.org/alice",
            "http://xmlns.com/foaf/0.1/knows",
            "https://example.org/bob",
            ObjectType::NamedNode,
        ),
    ];

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "https://example.org/".to_string());
    prefixes.insert("foaf".to_string(), "http://xmlns.com/foaf/0.1/".to_string());

    // Test filtering with all three criteria
    let filter = TripleFilter::new(Some("ex:alice"), Some("foaf:name"), Some("Alice"));
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].subject_value, "https://example.org/alice");
    assert_eq!(filtered[0].predicate, "http://xmlns.com/foaf/0.1/name");
    assert_eq!(filtered[0].object_value, "Alice");
}

#[rstest]
fn test_filter_integration_no_matches() {
    let triples = vec![create_test_triple(
        "https://example.org/alice",
        "http://xmlns.com/foaf/0.1/name",
        "Alice",
        ObjectType::Literal,
    )];

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "https://example.org/".to_string());

    // Test filtering with no matching results
    let filter = TripleFilter::new(Some("ex:nonexistent"), None, None);
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 0);
}

#[rstest]
fn test_filter_integration_angle_bracket_iris() {
    let triples = vec![create_test_triple(
        "https://example.org/alice",
        "http://xmlns.com/foaf/0.1/name",
        "Alice",
        ObjectType::Literal,
    )];

    let prefixes = HashMap::new();

    // Test filtering with angle bracket enclosed IRI
    let filter = TripleFilter::new(Some("<https://example.org/alice>"), None, None);
    let filtered = filter.filter_triples(&triples, &prefixes);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].object_value, "Alice");
}
