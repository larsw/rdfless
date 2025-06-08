use rstest::rstest;
use std::io::BufReader;
use rio_api::model::{Triple, Term, Literal, Subject};
use rio_api::parser::TriplesParser;
use rio_turtle::{TurtleParser, TurtleError};
use rdfless::{OwnedTriple, triple_to_owned};

#[rstest]
fn test_turtle_parser_basic() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let mut parser = TurtleParser::new(reader, None);

    let mut triples = Vec::new();
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    parser.parse_all(&mut callback).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::NamedNode);
    assert_eq!(triple.subject_value, "http://example.org/subject");
    assert_eq!(triple.predicate, "http://example.org/predicate");
    assert_eq!(triple.object_type, rdfless::ObjectType::Literal);
    assert_eq!(triple.object_value, "object");
}

#[rstest]
fn test_turtle_parser_with_prefixes() {
    let ttl = r#"
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix ex: <http://example.org/> .

        ex:Resource a rdf:Class .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let mut parser = TurtleParser::new(reader, None);

    let mut triples = Vec::new();
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    parser.parse_all(&mut callback).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::NamedNode);
    assert_eq!(triple.subject_value, "http://example.org/Resource");
    assert_eq!(triple.predicate, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
    assert_eq!(triple.object_type, rdfless::ObjectType::NamedNode);
    assert_eq!(triple.object_value, "http://www.w3.org/1999/02/22-rdf-syntax-ns#Class");

    // Check that prefixes were parsed correctly
    let prefixes = parser.prefixes();

    // Check that we have the expected prefixes
    assert!(prefixes.contains_key("rdf"));
    assert!(prefixes.contains_key("ex"));

    // Check the values
    assert_eq!(prefixes.get("rdf").unwrap(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
    assert_eq!(prefixes.get("ex").unwrap(), "http://example.org/");
}

#[rstest]
fn test_turtle_parser_with_blank_nodes() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .

        _:blank ex:predicate "value" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let mut parser = TurtleParser::new(reader, None);

    let mut triples = Vec::new();
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    parser.parse_all(&mut callback).unwrap();

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::BlankNode);
    assert_eq!(triple.predicate, "http://example.org/predicate");
    assert_eq!(triple.object_type, rdfless::ObjectType::Literal);
    assert_eq!(triple.object_value, "value");
}

#[rstest]
fn test_turtle_parser_with_literals() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:subject ex:string "simple string" .
        ex:subject ex:langString "hello"@en .
        ex:subject ex:integer "42"^^xsd:integer .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let mut parser = TurtleParser::new(reader, None);

    let mut triples = Vec::new();
    let mut callback = |triple: Triple| -> std::result::Result<(), TurtleError> {
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
        Ok(())
    };

    parser.parse_all(&mut callback).unwrap();

    assert_eq!(triples.len(), 3);

    // Check simple string literal
    let simple_string = triples.iter().find(|t| t.predicate == "http://example.org/string").unwrap();
    assert_eq!(simple_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(simple_string.object_value, "simple string");

    // Check language-tagged string
    let lang_string = triples.iter().find(|t| t.predicate == "http://example.org/langString").unwrap();
    assert_eq!(lang_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(lang_string.object_value, "hello");
    assert_eq!(lang_string.object_language.as_deref(), Some("en"));

    // Check typed literal
    let typed_literal = triples.iter().find(|t| t.predicate == "http://example.org/integer").unwrap();
    assert_eq!(typed_literal.object_type, rdfless::ObjectType::Literal);
    assert_eq!(typed_literal.object_value, "42");
    assert_eq!(typed_literal.object_datatype.as_deref(), Some("http://www.w3.org/2001/XMLSchema#integer"));
}
