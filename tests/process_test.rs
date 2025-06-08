use rstest::rstest;
use std::io::{BufReader, Cursor};
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs::File;
use rdfless::config::ColorConfig;
use rdfless::Args;

struct TestArgs {
    expand: bool,
}

impl Args for TestArgs {
    fn expand(&self) -> bool {
        self.expand
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

    // Read the captured output
    let mut output = String::new();
    pipe.stdout.unwrap().read_to_string(&mut output).unwrap();

    output
}

#[rstest]
fn test_process_input_basic() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs { expand: false };
    let colors = ColorConfig::default();

    // This test is more of a smoke test to ensure process_input doesn't panic
    // We can't easily capture the stdout in a unit test, so we just verify it doesn't error
    let result = rdfless::process_input(reader, &args, &colors);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_expand() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .

        ex:subject ex:predicate "object" .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs { expand: true };
    let colors = ColorConfig::default();

    let result = rdfless::process_input(reader, &args, &colors);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_file() {
    // Create a temporary file with TTL content
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "@prefix ex: <http://example.org/> .").unwrap();
    writeln!(temp_file, "").unwrap();
    writeln!(temp_file, "ex:subject ex:predicate \"object\" .").unwrap();

    // Get a reference to the file
    let file = temp_file.reopen().unwrap();
    let reader = BufReader::new(file);
    let args = TestArgs { expand: false };
    let colors = ColorConfig::default();

    let result = rdfless::process_input(reader, &args, &colors);
    assert!(result.is_ok());
}

#[rstest]
fn test_process_input_with_multiple_triples() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        @prefix foaf: <http://xmlns.com/foaf/0.1/> .

        ex:john a foaf:Person ;
            foaf:name "John Doe" ;
            foaf:age 30 .

        ex:jane a foaf:Person ;
            foaf:name "Jane Smith" ;
            foaf:knows ex:john .
    "#;

    let reader = BufReader::new(Cursor::new(ttl));
    let args = TestArgs { expand: false };
    let colors = ColorConfig::default();

    let result = rdfless::process_input(reader, &args, &colors);
    assert!(result.is_ok());
}
