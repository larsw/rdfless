use rdfless::config::{ColorConfig, Config, OutputConfig};
use rdfless::{ArgsConfig, InputFormat, OwnedTriple};
use rstest::rstest;
use std::collections::HashMap;
use std::io::Write;
use std::io::{BufReader, Cursor};
use tempfile::NamedTempFile;

struct TestArgs {
    expand: bool,
    compact: bool,
    format: Option<InputFormat>,
}

impl ArgsConfig for TestArgs {
    fn expand(&self, config: &Config) -> bool {
        if self.compact {
            false
        } else if self.expand {
            true
        } else {
            config.output.expand
        }
    }

    fn format(&self) -> Option<InputFormat> {
        self.format
    }

    fn use_pager(&self, _config: &Config) -> bool {
        false // Default to no pager for tests
    }

    fn no_pager_explicit(&self) -> bool {
        true // Explicitly disable paging in tests
    }

    fn get_colors(&self, config: &Config) -> rdfless::config::ColorConfig {
        config.colors.clone()
    }

    fn is_output_to_file(&self) -> bool {
        false // Tests don't output to files
    }

    fn continue_on_error(&self) -> bool {
        false // Default to strict parsing in tests
    }
}

// Helper function to capture stdout for testing
#[allow(dead_code)]
fn capture_stdout<F>(f: F) -> String
where
    F: FnOnce(),
{
    use std::io::Read;

    // Create a pipe to capture stdout
    let mut pipe = std::process::Command::new("sh")
        .arg("-c")
        .arg("cat > /dev/null")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    // Redirect stdout to the pipe
    let pipe_stdin = pipe.stdin.as_mut().unwrap();

    // Execute the function
    f();

    // Close the pipe stdin
    let _ = pipe_stdin;

    // Take stdout out of pipe to avoid partial move issues
    let mut stdout = pipe.stdout.take().unwrap();

    // Read the captured output
    let mut output = String::new();
    stdout.read_to_string(&mut output).unwrap();

    // Wait for the process to finish to avoid zombie processes
    pipe.wait().unwrap();

    output
}

#[rstest]
fn test_process_input_basic() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    // This test is more of a smoke test to ensure process_input doesn't panic
    // We can't easily capture the stdout in a unit test, so we just verify it doesn't error
    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_expand() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs {
        expand: true,
        compact: false,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_compact() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs {
        expand: false,
        compact: true,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_expand_and_compact() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    // When both flags are provided, compact takes precedence
    let args = TestArgs {
        expand: true,
        compact: true,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_file() {
    // Create a temporary file with TTL content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "@prefix ex: <https://example.org/> .").unwrap();
    writeln!(temp_file).unwrap();
    writeln!(temp_file, "ex:subject ex:predicate \"object\" .").unwrap();

    // Get a reference to the file
    let file = temp_file.reopen().unwrap();
    let reader = BufReader::new(file);
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_multiple_triples() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .
        @prefix foaf: <https://xmlns.com/foaf/0.1/> .

        ex:john a foaf:Person ;
            foaf:name "John Doe" ;
            foaf:age 30 .

        ex:jane a foaf:Person ;
            foaf:name "Jane Smith" ;
            foaf:knows ex:john .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_config_expand() {
    let ttl = r#"
        @prefix ex: <https://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    // No command line expand option provided
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::Turtle),
    };
    let colors = ColorConfig::default();

    // Config with expand=true
    let config = Config {
        output: OutputConfig {
            expand: true,
            pager: false,
            auto_pager: true,
            auto_pager_threshold: 0,
        },
        ..Default::default()
    };

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());

    // Test with config expand=false
    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::Turtle),
    };
    let config = Config {
        output: OutputConfig {
            expand: false,
            pager: false,
            auto_pager: true,
            auto_pager_threshold: 0,
        },
        ..Default::default()
    };

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_trig_format() {
    let trig = r#"
        @prefix ex: <https://example.org/> .
        @prefix foaf: <https://xmlns.com/foaf/0.1/> .

        ex:graph1 {
            ex:john a foaf:Person ;
                foaf:name "John Doe" ;
                foaf:age 30 .
        }
    "#;

    let reader = BufReader::new(Cursor::new(trig));
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::TriG),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_ntriples_format() {
    let ntriples = r#"
        <https://example.org/john> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://xmlns.com/foaf/0.1/Person> .
        <https://example.org/john> <https://xmlns.com/foaf/0.1/name> "John Doe" .
        <https://example.org/john> <https://xmlns.com/foaf/0.1/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
    "#;

    let reader = BufReader::new(Cursor::new(ntriples));
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::NTriples),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_nquads_format() {
    let nquads = r#"
        <https://example.org/john> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://xmlns.com/foaf/0.1/Person> <https://example.org/graph1> .
        <https://example.org/john> <https://xmlns.com/foaf/0.1/name> "John Doe" <https://example.org/graph1> .
        <https://example.org/john> <https://xmlns.com/foaf/0.1/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> <https://example.org/graph1> .
    "#;

    let reader = BufReader::new(Cursor::new(nquads));
    let args = TestArgs {
        expand: false,
        compact: false,
        format: Some(InputFormat::NQuads),
    };
    let colors = ColorConfig::default();
    let config = Config::default();

    let result = rdfless::process_input(reader, &args, &colors, &config);
    assert!(result.is_ok());
}

#[rstest]
fn test_args_config_interface() {
    let args = TestArgs {
        expand: true,
        compact: false,
        format: Some(InputFormat::Turtle),
    };

    let config = Config::default();

    // Test the ArgsConfig methods
    assert!(args.expand(&config)); // expand is true in args
    assert_eq!(args.format(), Some(InputFormat::Turtle));
    assert!(!args.use_pager(&config)); // Disabled in test implementation
    assert!(args.no_pager_explicit()); // Explicitly disabled in test implementation

    let colors = args.get_colors(&config);
    assert_eq!(colors.subject, config.colors.subject);
}

#[rstest]
fn test_args_config_expand_precedence() {
    // Test compact flag takes precedence
    let args = TestArgs {
        expand: true,
        compact: true, // This should override expand
        format: None,
    };

    let mut config = Config::default();
    config.output.expand = true;

    assert!(!args.expand(&config)); // compact overrides both args.expand and config.expand

    // Test config fallback when neither expand nor compact is set
    let args = TestArgs {
        expand: false,
        compact: false,
        format: None,
    };

    config.output.expand = true;
    assert!(args.expand(&config)); // Should use config value

    config.output.expand = false;
    assert!(!args.expand(&config)); // Should use config value
}

struct TestArgsWithPaging {
    expand: bool,
    compact: bool,
    format: Option<InputFormat>,
    use_pager: bool,
    no_pager: bool,
}

impl ArgsConfig for TestArgsWithPaging {
    fn expand(&self, config: &Config) -> bool {
        if self.compact {
            false
        } else if self.expand {
            true
        } else {
            config.output.expand
        }
    }

    fn format(&self) -> Option<InputFormat> {
        self.format
    }

    fn use_pager(&self, config: &Config) -> bool {
        if self.no_pager {
            false
        } else if self.use_pager {
            true
        } else {
            config.output.pager
        }
    }

    fn no_pager_explicit(&self) -> bool {
        self.no_pager
    }

    fn get_colors(&self, config: &Config) -> rdfless::config::ColorConfig {
        config.colors.clone()
    }

    fn is_output_to_file(&self) -> bool {
        false // Tests don't output to files
    }

    fn continue_on_error(&self) -> bool {
        false // Default to strict parsing in tests
    }
}

#[rstest]
fn test_args_config_paging_options() {
    let mut config = Config::default();
    config.output.pager = true;

    // Test explicit pager usage
    let args = TestArgsWithPaging {
        expand: false,
        compact: false,
        format: None,
        use_pager: true,
        no_pager: false,
    };

    assert!(args.use_pager(&config));
    assert!(!args.no_pager_explicit());

    // Test explicit pager disabling
    let args = TestArgsWithPaging {
        expand: false,
        compact: false,
        format: None,
        use_pager: false,
        no_pager: true,
    };

    assert!(!args.use_pager(&config)); // Explicit no_pager overrides config
    assert!(args.no_pager_explicit());

    // Test config fallback
    let args = TestArgsWithPaging {
        expand: false,
        compact: false,
        format: None,
        use_pager: false,
        no_pager: false,
    };

    config.output.pager = true;
    assert!(args.use_pager(&config)); // Uses config value

    config.output.pager = false;
    assert!(!args.use_pager(&config)); // Uses config value
}

#[rstest]
fn test_terminal_height_detection() {
    let height = rdfless::get_terminal_height();

    // Should return a reasonable default even if not in a terminal
    assert!(height > 0);
    assert!(height <= 200); // Reasonable upper bound for most terminals
}

#[rstest]
fn test_output_estimation() {
    // Create some test triples
    let triples = vec![
        OwnedTriple {
            subject_type: rdfless::SubjectType::NamedNode,
            subject_value: "https://example.org/subject1".to_string(),
            predicate: "https://example.org/predicate1".to_string(),
            object_type: rdfless::ObjectType::Literal,
            object_value: "value1".to_string(),
            object_datatype: None,
            object_language: None,
            graph: None,
        },
        OwnedTriple {
            subject_type: rdfless::SubjectType::NamedNode,
            subject_value: "https://example.org/subject2".to_string(),
            predicate: "https://example.org/predicate2".to_string(),
            object_type: rdfless::ObjectType::Literal,
            object_value: "value2".to_string(),
            object_datatype: None,
            object_language: None,
            graph: None,
        },
    ];

    let mut prefixes = HashMap::new();
    prefixes.insert("ex".to_string(), "https://example.org/".to_string());

    // Test with compacted output (should include prefix lines)
    let estimated_lines_compact = rdfless::estimate_output_lines(&triples, &prefixes, false);
    assert!(estimated_lines_compact > 0);

    // Test with expanded output (no prefix lines)
    let estimated_lines_expanded = rdfless::estimate_output_lines(&triples, &prefixes, true);
    assert!(estimated_lines_expanded > 0);

    // Compact format should have more lines due to prefixes
    assert!(estimated_lines_compact >= estimated_lines_expanded);
}

#[rstest]
fn test_should_use_pager() {
    let args = TestArgsWithPaging {
        expand: false,
        compact: false,
        format: None,
        use_pager: false,
        no_pager: false,
    };

    let mut config = Config::default();

    // Test explicit no pager
    let args_no_pager = TestArgsWithPaging {
        expand: false,
        compact: false,
        format: None,
        use_pager: false,
        no_pager: true,
    };

    assert!(!rdfless::should_use_pager(&args_no_pager, &config, 100));

    // Test explicit use pager
    let args_use_pager = TestArgsWithPaging {
        expand: false,
        compact: false,
        format: None,
        use_pager: true,
        no_pager: false,
    };

    assert!(rdfless::should_use_pager(&args_use_pager, &config, 100));

    // Test auto-pager logic with explicit threshold
    config.output.auto_pager = true;
    config.output.auto_pager_threshold = 10; // Use explicit threshold instead of terminal height

    // Should not use pager for small output
    assert!(!rdfless::should_use_pager(&args, &config, 5));

    // Should use pager for large output
    assert!(rdfless::should_use_pager(&args, &config, 15));

    // Test disabled auto-pager
    config.output.auto_pager = false;
    assert!(!rdfless::should_use_pager(&args, &config, 100)); // Should not use pager even for large output
}
