# rdfless

A colorful pretty printer for RDF data with ANSI colors.

## Description

`rdfless` is a command-line tool that reads RDF data in Turtle format and pretty-prints it with syntax highlighting. It's designed to make RDF data more readable in terminal environments.

Key features:
- Colorized output for different RDF components (subjects, predicates, objects, literals)
- Support for reading from files or stdin (piped input)
- Option to expand prefixes or display PREFIX declarations
- Customizable colors through a configuration file

### Supported Formats
- Turtle (TTL)
- Turtle-star
- TriG
- TriG-star

## Installation

### Using Cargo

```bash
cargo install rdfless
```

### From Source

```bash
git clone https://github.com/larsw/rdfless.git
cd rdfless
cargo build --release
```

The binary will be available at `target/release/rdfless`.

## Usage

```bash
# Process a TTL file
rdfless file.ttl

# Process a TriG file
rdfless file.trig

# Process multiple files
rdfless file1.ttl file2.trig

# Read from stdin
cat file.ttl | rdfless

# Expand prefixes instead of showing PREFIX declarations
rdfless --expand file.ttl

# Override the input format (auto-detected from file extension by default)
rdfless --format turtle file.rdf
rdfless --format trig file.rdf
```

## Configuration

`rdfless` uses a YAML configuration file to customize colors. The configuration file is located at:

```
~/.local/rdfless/colors.yml
```

If the file doesn't exist, a default configuration will be created automatically.

Example configuration:

```yaml
subject: blue
predicate: green
object: white
literal: red
prefix: yellow
base: yellow
```

Available colors:
- black, red, green, yellow, blue, magenta, cyan, white
- bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white

## Example

Input:
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.org/> .

ex:Person a rdfs:Class ;
    rdfs:label "Person" .

ex:john a ex:Person ;
    rdfs:label "John Doe" .
```

Output will be the same content but with syntax highlighting for better readability.

## Screenshots

### Turtle Format (TTL)

Compact Mode | Expanded Mode
:----------:|:------------:
![Turtle Compact](assets/sample-ttl-compact.png) | ![Turtle Expanded](assets/sample-ttl-expanded.png)

### TriG Format (TRIG)

Compact Mode | Expanded Mode
:----------:|:------------:
![TriG Compact](assets/sample-trig-compact.png) | ![TriG Expanded](assets/sample-trig-expanded.png)

## License

This project is licensed under the BSD-3-Clause License - see the LICENSE file for details.

## Author

Lars Wilhelmsen <lars@lars-backwards.org>
