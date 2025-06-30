use rdfless::{parse_robust, InputFormat, ParseResult};
use rstest::rstest;
use std::io::BufReader;

#[rstest]
fn test_turtle_robust_parsing_continue_on_error() {
    let ttl = r#"
@prefix ex: <https://example.org/> .

# Valid triple
ex:subject1 ex:predicate1 "valid object 1" .

# Invalid triple with undefined prefix
undefined:subject ex:predicate "invalid object" .

# Another valid triple
ex:subject2 ex:predicate2 "valid object 2" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let result = parse_robust(reader, InputFormat::Turtle, true).unwrap();

    // Should have parsed 2 valid triples
    assert_eq!(result.triple_count(), 2);

    // Should have 1 error
    assert_eq!(result.error_count(), 1);
    assert!(result.has_errors());

    // Note: oxttl doesn't currently provide access to parsed prefixes
    // This functionality is not available in the current version

    // Check the parsed triples
    let triple1 = result
        .triples
        .iter()
        .find(|t| t.predicate == "https://example.org/predicate1")
        .unwrap();
    assert_eq!(triple1.subject_value, "https://example.org/subject1");
    assert_eq!(triple1.object_value, "valid object 1");

    let triple2 = result
        .triples
        .iter()
        .find(|t| t.predicate == "https://example.org/predicate2")
        .unwrap();
    assert_eq!(triple2.subject_value, "https://example.org/subject2");
    assert_eq!(triple2.object_value, "valid object 2");

    // Check the error
    let error = &result.errors[0];
    // Note: The error message format may be different with oxttl
    assert!(
        error.message.contains("undefined")
            || error.message.contains("unknown")
            || error.message.contains("prefix")
    );
}

#[rstest]
fn test_turtle_robust_parsing_strict_mode() {
    let ttl = r#"
@prefix ex: <https://example.org/> .

# Valid triple
ex:subject1 ex:predicate1 "valid object 1" .

# Invalid triple with undefined prefix
undefined:subject ex:predicate "invalid object" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let result = parse_robust(reader, InputFormat::Turtle, false);

    // Should fail in strict mode
    assert!(result.is_err());
}

#[rstest]
fn test_ntriples_robust_parsing_continue_on_error() {
    let nt = r#"
<https://example.org/subject1> <https://example.org/predicate1> "valid object 1" .
This is not a valid N-Triple line
<https://example.org/subject2> <https://example.org/predicate2> "valid object 2" .
Another invalid line without proper formatting
<https://example.org/subject3> <https://example.org/predicate3> "valid object 3" .
    "#;

    let reader = BufReader::new(nt.as_bytes());
    let result = parse_robust(reader, InputFormat::NTriples, true).unwrap();

    // Should have parsed 3 valid triples
    assert_eq!(result.triple_count(), 3);

    // Should have 2 errors
    assert_eq!(result.error_count(), 2);
    assert!(result.has_errors());

    // N-Triples doesn't have prefixes
    assert!(result.prefixes.is_empty());

    // Check that all valid triples were parsed
    assert!(result
        .triples
        .iter()
        .any(|t| t.object_value == "valid object 1"));
    assert!(result
        .triples
        .iter()
        .any(|t| t.object_value == "valid object 2"));
    assert!(result
        .triples
        .iter()
        .any(|t| t.object_value == "valid object 3"));
}

#[rstest]
fn test_ntriples_robust_parsing_strict_mode() {
    let nt = r#"
<https://example.org/subject1> <https://example.org/predicate1> "valid object 1" .
This is not a valid N-Triple line
    "#;

    let reader = BufReader::new(nt.as_bytes());
    let result = parse_robust(reader, InputFormat::NTriples, false);

    // Should fail in strict mode
    assert!(result.is_err());
}

#[rstest]
fn test_parse_result_methods() {
    let mut result = ParseResult::new();

    // Initially empty
    assert_eq!(result.triple_count(), 0);
    assert_eq!(result.error_count(), 0);
    assert!(!result.has_errors());

    // Add some mock data
    result.triples.push(rdfless::OwnedTriple {
        subject_type: rdfless::SubjectType::NamedNode,
        subject_value: "https://example.org/subject".to_string(),
        predicate: "https://example.org/predicate".to_string(),
        object_type: rdfless::ObjectType::Literal,
        object_value: "object".to_string(),
        object_datatype: None,
        object_language: None,
        graph: None,
        subject_triple: None,
        object_triple: None,
    });

    result.errors.push(rdfless::ParseError {
        line: 5,
        position: 10,
        message: "Test error".to_string(),
    });

    result
        .prefixes
        .insert("ex".to_string(), "https://example.org/".to_string());

    // Check counts
    assert_eq!(result.triple_count(), 1);
    assert_eq!(result.error_count(), 1);
    assert!(result.has_errors());
}

#[rstest]
fn test_turtle_with_comments_and_mixed_content() {
    let ttl = r#"
@prefix ex: <https://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

# This is a comment
ex:person1 foaf:name "Alice" .

# Another comment
ex:person1 foaf:age 30 .

# This line has a syntax error
ex:person1 foaf:knows missing_angle_bracket .

# This is valid again
ex:person2 foaf:name "Bob" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let result = parse_robust(reader, InputFormat::Turtle, true).unwrap();

    // Should have parsed 3 valid triples (Alice's name, Alice's age, Bob's name)
    assert_eq!(result.triple_count(), 3);

    // Should have 1 error (the missing angle bracket line)
    assert_eq!(result.error_count(), 1);

    // Note: oxttl doesn't currently provide access to parsed prefixes
    // This functionality is not available in the current version
}
