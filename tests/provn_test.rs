use rdfless::{InputFormat, ObjectType, SubjectType};
use rstest::rstest;
use std::io::BufReader;

#[rstest]
fn test_provn_parser_entity() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  entity(ex:e1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, prefixes) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    // Should have at least one triple (rdf:type prov:Entity)
    assert!(!triples.is_empty());

    // Check that prefixes were parsed
    assert!(prefixes.contains_key("ex"));
    assert!(prefixes.contains_key("prov"));
    assert_eq!(prefixes.get("ex").unwrap(), "http://example.org/");

    // Check the type triple
    let type_triple = triples
        .iter()
        .find(|t| t.predicate.contains("rdf-syntax-ns#type"))
        .expect("Should have rdf:type triple");
    assert_eq!(type_triple.subject_type, SubjectType::NamedNode);
    assert_eq!(type_triple.subject_value, "http://example.org/e1");
    assert_eq!(type_triple.object_type, ObjectType::NamedNode);
    assert_eq!(type_triple.object_value, "http://www.w3.org/ns/prov#Entity");
}

#[rstest]
fn test_provn_parser_entity_with_attributes() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  entity(ex:e1, [prov:type="File"])
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    // Should have type triple and attribute triple
    assert!(triples.len() >= 2);

    // Check attribute triple
    let attr_triple = triples
        .iter()
        .find(|t| t.predicate.contains("prov#type"))
        .expect("Should have prov:type attribute");
    assert_eq!(attr_triple.object_type, ObjectType::Literal);
    assert_eq!(attr_triple.object_value, "File");
}

#[rstest]
fn test_provn_parser_activity() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  activity(ex:a1, 2024-01-15T10:00:00, 2024-01-15T11:00:00)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    // Should have type triple, start time, and end time
    assert!(triples.len() >= 3);

    // Check type triple
    let type_triple = triples
        .iter()
        .find(|t| t.predicate.contains("rdf-syntax-ns#type"))
        .expect("Should have rdf:type triple");
    assert_eq!(
        type_triple.object_value,
        "http://www.w3.org/ns/prov#Activity"
    );

    // Check start time
    let start_time_triple = triples
        .iter()
        .find(|t| t.predicate.contains("startedAtTime"))
        .expect("Should have startedAtTime triple");
    assert_eq!(start_time_triple.object_type, ObjectType::Literal);
    assert_eq!(start_time_triple.object_value, "2024-01-15T10:00:00");

    // Check end time
    let end_time_triple = triples
        .iter()
        .find(|t| t.predicate.contains("endedAtTime"))
        .expect("Should have endedAtTime triple");
    assert_eq!(end_time_triple.object_type, ObjectType::Literal);
    assert_eq!(end_time_triple.object_value, "2024-01-15T11:00:00");
}

#[rstest]
fn test_provn_parser_agent() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  agent(ex:ag1, [prov:type="Person"])
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    // Should have type triple and attribute
    assert!(triples.len() >= 2);

    // Check type triple
    let type_triple = triples
        .iter()
        .find(|t| t.predicate.contains("rdf-syntax-ns#type"))
        .expect("Should have rdf:type triple");
    assert_eq!(type_triple.object_value, "http://www.w3.org/ns/prov#Agent");
}

#[rstest]
fn test_provn_parser_was_generated_by() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  wasGeneratedBy(ex:e2, ex:a1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, SubjectType::NamedNode);
    assert_eq!(triple.subject_value, "http://example.org/e2");
    assert_eq!(triple.predicate, "http://www.w3.org/ns/prov#wasGeneratedBy");
    assert_eq!(triple.object_type, ObjectType::NamedNode);
    assert_eq!(triple.object_value, "http://example.org/a1");
}

#[rstest]
fn test_provn_parser_used() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  used(ex:a1, ex:e1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_value, "http://example.org/a1");
    assert_eq!(triple.predicate, "http://www.w3.org/ns/prov#used");
    assert_eq!(triple.object_value, "http://example.org/e1");
}

#[rstest]
fn test_provn_parser_was_associated_with() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  wasAssociatedWith(ex:a1, ex:ag1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_value, "http://example.org/a1");
    assert_eq!(
        triple.predicate,
        "http://www.w3.org/ns/prov#wasAssociatedWith"
    );
    assert_eq!(triple.object_value, "http://example.org/ag1");
}

#[rstest]
fn test_provn_parser_was_attributed_to() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  wasAttributedTo(ex:e1, ex:ag1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_value, "http://example.org/e1");
    assert_eq!(
        triple.predicate,
        "http://www.w3.org/ns/prov#wasAttributedTo"
    );
    assert_eq!(triple.object_value, "http://example.org/ag1");
}

#[rstest]
fn test_provn_parser_was_derived_from() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  wasDerivedFrom(ex:e2, ex:e1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, _) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_value, "http://example.org/e2");
    assert_eq!(triple.predicate, "http://www.w3.org/ns/prov#wasDerivedFrom");
    assert_eq!(triple.object_value, "http://example.org/e1");
}

#[rstest]
fn test_provn_parser_complex_document() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  entity(ex:e1, [prov:type="File"])
  entity(ex:e2, [prov:type="File"])
  activity(ex:a1, 2024-01-15T10:00:00, 2024-01-15T11:00:00)
  agent(ex:ag1, [prov:type="Person"])
  
  wasGeneratedBy(ex:e2, ex:a1)
  used(ex:a1, ex:e1)
  wasAssociatedWith(ex:a1, ex:ag1)
  wasAttributedTo(ex:e2, ex:ag1)
  wasDerivedFrom(ex:e2, ex:e1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let (triples, prefixes) = rdfless::parse_for_estimation(reader, InputFormat::ProvN).unwrap();

    // Should have many triples
    assert!(triples.len() >= 10);

    // Check prefixes
    assert_eq!(prefixes.len(), 2);
    assert!(prefixes.contains_key("ex"));
    assert!(prefixes.contains_key("prov"));
}

#[rstest]
fn test_provn_robust_parsing() {
    let provn = r#"
document
  prefix ex <http://example.org/>
  prefix prov <http://www.w3.org/ns/prov#>
  
  entity(ex:e1)
  activity(ex:a1)
  wasGeneratedBy(ex:e1, ex:a1)
endDocument
    "#;

    let reader = BufReader::new(provn.as_bytes());
    let result = rdfless::parse_robust(reader, InputFormat::ProvN, false).unwrap();

    // Should successfully parse
    assert!(!result.has_errors());
    assert!(result.triple_count() >= 3);
}

#[rstest]
fn test_provn_format_detection() {
    use std::path::Path;

    let path = Path::new("test.provn");
    let format = rdfless::detect_format_from_path(path);

    assert_eq!(format, Some(InputFormat::ProvN));
}
