use oxttl::{NQuadsParser, NTriplesParser, TurtleParser};
use rdfless::{quad_to_owned, triple_to_owned};
use rstest::rstest;
use std::io::BufReader;

#[rstest]
fn test_turtle_parser_basic() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let parser = TurtleParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::NamedNode);
    assert_eq!(triple.subject_value, "https://example.org/subject");
    assert_eq!(triple.predicate, "https://example.org/predicate");
    assert_eq!(triple.object_type, rdfless::ObjectType::Literal);
    assert_eq!(triple.object_value, "object");
}

#[rstest]
fn test_turtle_parser_with_prefixes() {
    let ttl = r#"
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix ex: <https://example.org/> .

        ex:Resource a rdf:Class .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let parser = TurtleParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::NamedNode);
    assert_eq!(triple.subject_value, "https://example.org/Resource");
    assert_eq!(
        triple.predicate,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
    );
    assert_eq!(triple.object_type, rdfless::ObjectType::NamedNode);
    assert_eq!(
        triple.object_value,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#Class"
    );

    // Note: oxttl doesn't currently provide direct access to parsed prefixes
    // This functionality is not available in the current version
}

#[rstest]
fn test_turtle_parser_with_blank_nodes() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        _:blank ex:predicate "value" .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let parser = TurtleParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::BlankNode);
    assert_eq!(triple.predicate, "https://example.org/predicate");
    assert_eq!(triple.object_type, rdfless::ObjectType::Literal);
    assert_eq!(triple.object_value, "value");
}

#[rstest]
fn test_turtle_parser_with_literals() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:subject ex:string "simple string" .
        ex:subject ex:langString "hello"@en .
        ex:subject ex:integer "42"^^xsd:integer .
    "#;

    let reader = BufReader::new(ttl.as_bytes());
    let parser = TurtleParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 3);

    // Check simple string literal
    let simple_string = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/string")
        .unwrap();
    assert_eq!(simple_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(simple_string.object_value, "simple string");

    // Check language-tagged string
    let lang_string = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/langString")
        .unwrap();
    assert_eq!(lang_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(lang_string.object_value, "hello");
    assert_eq!(lang_string.object_language.as_deref(), Some("en"));

    // Check typed literal
    let typed_literal = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/integer")
        .unwrap();
    assert_eq!(typed_literal.object_type, rdfless::ObjectType::Literal);
    assert_eq!(typed_literal.object_value, "42");
    assert_eq!(
        typed_literal.object_datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#integer")
    );
}

#[rstest]
fn test_ntriples_parser_basic() {
    let nt = r#"
        <https://example.org/subject> <https://example.org/predicate> "object" .
    "#;

    let reader = BufReader::new(nt.as_bytes());
    let parser = NTriplesParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::NamedNode);
    assert_eq!(triple.subject_value, "https://example.org/subject");
    assert_eq!(triple.predicate, "https://example.org/predicate");
    assert_eq!(triple.object_type, rdfless::ObjectType::Literal);
    assert_eq!(triple.object_value, "object");
}

#[rstest]
fn test_ntriples_parser_with_blank_nodes() {
    let nt = r#"
        _:blank <https://example.org/predicate> "value" .
    "#;

    let reader = BufReader::new(nt.as_bytes());
    let parser = NTriplesParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 1);

    let triple = &triples[0];
    assert_eq!(triple.subject_type, rdfless::SubjectType::BlankNode);
    assert_eq!(triple.predicate, "https://example.org/predicate");
    assert_eq!(triple.object_type, rdfless::ObjectType::Literal);
    assert_eq!(triple.object_value, "value");
}

#[rstest]
fn test_ntriples_parser_with_literals() {
    let nt = r#"
        <https://example.org/subject> <https://example.org/string> "simple string" .
        <https://example.org/subject> <https://example.org/langString> "hello"@en .
        <https://example.org/subject> <https://example.org/integer> "42"^^<http://www.w3.org/2001/XMLSchema#integer> .
    "#;

    let reader = BufReader::new(nt.as_bytes());
    let parser = NTriplesParser::new().for_reader(reader);

    let mut triples = Vec::new();
    for triple_result in parser {
        let triple = triple_result.unwrap();
        let owned_triple = triple_to_owned(&triple);
        triples.push(owned_triple);
    }

    assert_eq!(triples.len(), 3);

    // Check simple string literal
    let simple_string = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/string")
        .unwrap();
    assert_eq!(simple_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(simple_string.object_value, "simple string");

    // Check language-tagged string
    let lang_string = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/langString")
        .unwrap();
    assert_eq!(lang_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(lang_string.object_value, "hello");
    assert_eq!(lang_string.object_language.as_deref(), Some("en"));

    // Check typed literal
    let typed_literal = triples
        .iter()
        .find(|t| t.predicate == "https://example.org/integer")
        .unwrap();
    assert_eq!(typed_literal.object_type, rdfless::ObjectType::Literal);
    assert_eq!(typed_literal.object_value, "42");
    assert_eq!(
        typed_literal.object_datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#integer")
    );
}

#[rstest]
fn test_nquads_parser_basic() {
    let nq = r#"
        <https://example.org/subject> <https://example.org/predicate> "object" <https://example.org/graph> .
    "#;

    let reader = BufReader::new(nq.as_bytes());
    let parser = NQuadsParser::new().for_reader(reader);

    let mut quads = Vec::new();
    for quad_result in parser {
        let quad = quad_result.unwrap();
        let owned_triple = quad_to_owned(&quad);
        quads.push(owned_triple);
    }

    assert_eq!(quads.len(), 1);

    let quad = &quads[0];
    assert_eq!(quad.subject_type, rdfless::SubjectType::NamedNode);
    assert_eq!(quad.subject_value, "https://example.org/subject");
    assert_eq!(quad.predicate, "https://example.org/predicate");
    assert_eq!(quad.object_type, rdfless::ObjectType::Literal);
    assert_eq!(quad.object_value, "object");
    assert_eq!(quad.graph.as_deref(), Some("https://example.org/graph"));
}

#[rstest]
fn test_nquads_parser_with_blank_nodes() {
    let nq = r#"
        _:blank <https://example.org/predicate> "value" <https://example.org/graph> .
    "#;

    let reader = BufReader::new(nq.as_bytes());
    let parser = NQuadsParser::new().for_reader(reader);

    let mut quads = Vec::new();
    for quad_result in parser {
        let quad = quad_result.unwrap();
        let owned_triple = quad_to_owned(&quad);
        quads.push(owned_triple);
    }

    assert_eq!(quads.len(), 1);

    let quad = &quads[0];
    assert_eq!(quad.subject_type, rdfless::SubjectType::BlankNode);
    assert_eq!(quad.predicate, "https://example.org/predicate");
    assert_eq!(quad.object_type, rdfless::ObjectType::Literal);
    assert_eq!(quad.object_value, "value");
    assert_eq!(quad.graph.as_deref(), Some("https://example.org/graph"));
}

#[rstest]
fn test_nquads_parser_with_literals() {
    let nq = r#"
        <https://example.org/subject> <https://example.org/string> "simple string" <https://example.org/graph> .
        <https://example.org/subject> <https://example.org/langString> "hello"@en <https://example.org/graph> .
        <https://example.org/subject> <https://example.org/integer> "42"^^<http://www.w3.org/2001/XMLSchema#integer> <https://example.org/graph> .
    "#;

    let reader = BufReader::new(nq.as_bytes());
    let parser = NQuadsParser::new().for_reader(reader);

    let mut quads = Vec::new();
    for quad_result in parser {
        let quad = quad_result.unwrap();
        let owned_triple = quad_to_owned(&quad);
        quads.push(owned_triple);
    }

    assert_eq!(quads.len(), 3);

    // Check simple string literal
    let simple_string = quads
        .iter()
        .find(|t| t.predicate == "https://example.org/string")
        .unwrap();
    assert_eq!(simple_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(simple_string.object_value, "simple string");
    assert_eq!(
        simple_string.graph.as_deref(),
        Some("https://example.org/graph")
    );

    // Check language-tagged string
    let lang_string = quads
        .iter()
        .find(|t| t.predicate == "https://example.org/langString")
        .unwrap();
    assert_eq!(lang_string.object_type, rdfless::ObjectType::Literal);
    assert_eq!(lang_string.object_value, "hello");
    assert_eq!(lang_string.object_language.as_deref(), Some("en"));
    assert_eq!(
        lang_string.graph.as_deref(),
        Some("https://example.org/graph")
    );

    // Check typed literal
    let typed_literal = quads
        .iter()
        .find(|t| t.predicate == "https://example.org/integer")
        .unwrap();
    assert_eq!(typed_literal.object_type, rdfless::ObjectType::Literal);
    assert_eq!(typed_literal.object_value, "42");
    assert_eq!(
        typed_literal.object_datatype.as_deref(),
        Some("http://www.w3.org/2001/XMLSchema#integer")
    );
    assert_eq!(
        typed_literal.graph.as_deref(),
        Some("https://example.org/graph")
    );
}
