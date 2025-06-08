use rdfless::config::ColorConfig;
use rstest::rstest;
use std::collections::HashMap;

// Import the OwnedTriple, SubjectType, and ObjectType from the main module
// We need to make these public in the main.rs file
use rdfless::{ObjectType, OwnedTriple, SubjectType};

#[rstest]
fn test_format_owned_subject_with_prefix() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "value".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "http://example.org/".to_string());

    let colors = ColorConfig::default();

    let result = rdfless::format_owned_subject(&triple, Some(&prefixes), &colors);
    assert_eq!(result, "ex:subject");
}

#[rstest]
fn test_format_owned_subject_without_prefix() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "value".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let colors = ColorConfig::default();

    let result = rdfless::format_owned_subject(&triple, None, &colors);
    assert_eq!(result, "<http://example.org/subject>");
}

#[rstest]
fn test_format_owned_subject_blank_node() {
    let triple = OwnedTriple {
        subject_type: SubjectType::BlankNode,
        subject_value: "blank1".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "value".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let colors = ColorConfig::default();

    let result = rdfless::format_owned_subject(&triple, None, &colors);
    assert_eq!(result, "_:blank1");
}

#[rstest]
fn test_format_owned_predicate_with_prefix() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "value".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "http://example.org/".to_string());

    let colors = ColorConfig::default();

    // Since the result includes color formatting, we can't directly compare strings
    // Instead, we'll check that it contains the expected prefix format
    let result = rdfless::format_owned_predicate(&triple, Some(&prefixes), &colors);
    assert!(result.contains("ex:predicate"));
}

#[rstest]
fn test_format_owned_object_named_node_with_prefix() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::NamedNode,
        object_value: "http://example.org/object".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "http://example.org/".to_string());

    let colors = ColorConfig::default();

    // Since the result includes color formatting, we can't directly compare strings
    // Instead, we'll check that it contains the expected prefix format
    let result = rdfless::format_owned_object(&triple, Some(&prefixes), &colors);
    assert!(result.contains("ex:object"));
}

#[rstest]
fn test_format_owned_object_blank_node() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::BlankNode,
        object_value: "blank1".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let colors = ColorConfig::default();

    // Since the result includes color formatting, we can't directly compare strings
    // Instead, we'll check that it contains the expected blank node format
    let result = rdfless::format_owned_object(&triple, None, &colors);
    assert!(result.contains("_:blank1"));
}

#[rstest]
fn test_format_owned_object_simple_literal() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "simple value".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
    };

    let colors = ColorConfig::default();

    // Since the result includes color formatting, we can't directly compare strings
    // Instead, we'll check that it contains the expected literal format
    let result = rdfless::format_owned_object(&triple, None, &colors);
    assert!(result.contains("\"simple value\""));
}

#[rstest]
fn test_format_owned_object_language_tagged_literal() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "hello".to_string(),
        object_datatype: None,
        object_language: Some("en".to_string()),
        graph: None,
    };

    let colors = ColorConfig::default();

    // Since the result includes color formatting, we can't directly compare strings
    // Instead, we'll check that it contains the expected language-tagged literal format
    let result = rdfless::format_owned_object(&triple, None, &colors);
    assert!(result.contains("\"hello\"@en"));
}

#[rstest]
fn test_format_owned_object_typed_literal() {
    let triple = OwnedTriple {
        subject_type: SubjectType::NamedNode,
        subject_value: "http://example.org/subject".to_string(),
        predicate: "http://example.org/predicate".to_string(),
        object_type: ObjectType::Literal,
        object_value: "42".to_string(),
        object_datatype: Some("http://www.w3.org/2001/XMLSchema#integer".to_string()),
        object_language: None,
        graph: None,
    };

    let mut prefixes = HashMap::new();
    prefixes.insert(
        "xsd".to_string(),
        "http://www.w3.org/2001/XMLSchema#".to_string(),
    );

    let colors = ColorConfig::default();

    // Since the result includes color formatting, we can't directly compare strings
    // Instead, we'll check that it contains the expected typed literal format
    let result = rdfless::format_owned_object(&triple, Some(&prefixes), &colors);
    assert!(result.contains("\"42\"^^xsd:integer"));
}
