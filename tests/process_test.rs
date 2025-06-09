use rdfless::config::{ColorConfig, Config};
use rdfless::{Args, InputFormat};
use rstest::rstest;
use std::io::Write;
use std::io::{BufReader, Cursor};
use tempfile::NamedTempFile;

struct TestArgs {
    expand: bool,
    compact: bool,
    format: Option<InputFormat>,
}

impl Args for TestArgs {
    fn expand(&self, config: &Config) -> bool {
        if self.compact {
            false
        } else if self.expand {
            true
        } else {
            config.expand
        }
    }

    fn format(&self) -> Option<InputFormat> {
        self.format
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
        expand: true,
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
        expand: false,
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
