use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use rio_api::model::{Triple, Term, NamedNode, BlankNode, Literal, Subject};
use rio_api::parser::TriplesParser;
use rio_turtle::TurtleParser;
use std::fs::File;
use std::io::{self, BufReader, IsTerminal, Read};
use std::path::PathBuf;

/// A TTL (Turtle) pretty printer with ANSI colors
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input files (TTL format)
    #[arg(name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check if we should read from stdin or files
    if args.files.is_empty() {
        // Read from stdin if no files are provided and stdin is not a terminal
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);
            process_input(reader)?;
        } else {
            eprintln!("No input files provided and no input piped to stdin.");
            eprintln!("Usage: cat file.ttl | ttlless");
            eprintln!("   or: ttlless file1.ttl [file2.ttl ...]");
            std::process::exit(1);
        }
    } else {
        // Process each file
        for file_path in &args.files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let reader = BufReader::new(file);
            process_input(reader)?;
        }
    }

    Ok(())
}

fn process_input<R: Read>(reader: BufReader<R>) -> Result<()> {
    let mut parser = TurtleParser::new(reader, None);

    // Track the current subject to handle formatting with semicolons
    let mut current_subject: Option<String> = None;

    // Process each triple
    let mut callback = |triple: Triple| -> std::result::Result<(), rio_turtle::TurtleError> {
        let subject_str = format_subject(&triple.subject);
        let predicate_str = format_named_node(&triple.predicate);
        let object_str = format_term(&triple.object);

        // Check if we're continuing with the same subject
        if let Some(ref current) = current_subject {
            if *current == subject_str {
                // Same subject, print with semicolon
                println!("    {} ;", predicate_str.bright_green());
                println!("        {} .", object_str);
            } else {
                // New subject
                if current_subject.is_some() {
                    println!();  // Add a blank line between statements
                }
                println!("{}", subject_str.bright_blue().bold());
                println!("    {} ;", predicate_str.bright_green());
                println!("        {} .", object_str);
                current_subject = Some(subject_str);
            }
        } else {
            // First subject
            println!("{}", subject_str.bright_blue().bold());
            println!("    {} ;", predicate_str.bright_green());
            println!("        {} .", object_str);
            current_subject = Some(subject_str);
        }

        Ok(())
    };

    parser.parse_all(&mut callback)?;

    Ok(())
}

// Format a Subject
fn format_subject(subject: &Subject) -> String {
    match subject {
        Subject::NamedNode(node) => format_named_node(node),
        Subject::BlankNode(node) => format_blank_node(node),
        Subject::Triple(triple) => format!("{{ {} {} {} }}", 
            format_subject(&triple.subject), 
            format_named_node(&triple.predicate), 
            format_term(&triple.object)),
    }
}

// Format a Term (object)
fn format_term(term: &Term) -> String {
    match term {
        Term::NamedNode(node) => format_named_node(node),
        Term::BlankNode(node) => format_blank_node(node),
        Term::Literal(literal) => format_literal(literal),
        Term::Triple(triple) => format!("{{ {} {} {} }}", 
            format_subject(&triple.subject), 
            format_named_node(&triple.predicate), 
            format_term(&triple.object)),
    }
}

// Format a NamedNode (URI)
fn format_named_node(node: &NamedNode) -> String {
    format!("<{}>", node.iri)
}

// Format a BlankNode
fn format_blank_node(node: &BlankNode) -> String {
    format!("_:{}", node.id)
}

// Format a Literal
fn format_literal(literal: &Literal) -> String {
    match literal {
        Literal::Simple { value } => {
            format!("\"{}\"", value).bright_red().to_string()
        }
        Literal::LanguageTaggedString { value, language } => {
            format!("\"{}\"@{}", value, language).bright_red().to_string()
        }
        Literal::Typed { value, datatype } => {
            format!("\"{}\"^^<{}>", value, datatype.iri).bright_red().to_string()
        }
    }
}
