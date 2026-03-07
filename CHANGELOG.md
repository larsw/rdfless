# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- _None_

### Changed
- Updated default dark theme colors to use bright variants (bright_blue, bright_green, bright_white, bright_red, bright_yellow) for improved visibility and contrast on dark terminal backgrounds, following recommendations from https://blog.xoria.org/terminal-colors/

### Fixed
- _None_

## [0.4.1] - 2026-01-22

### Added
- PROV-N (Provenance Notation) format support with parser and sample file
- Auto-detection of PROV-N format from `.provn` file extension
- PROV-N format option in CLI (`--format provn`)
- PROV-N parsing tests and documentation updates

### Fixed
- Improve PROV-N attribute parsing and constants after review

## [0.3.21] - 2025-11-13

### Fixed
- Display `@base` directives in output (previously parsed but not shown)
- Render `rdf:type` as the bold `a` shortcut in formatted output

## [0.3.19] - 2025-11-13

### Fixed
- Relative IRI prefix parsing for Turtle/TriG

### Added
- Regression tests for relative IRI prefix parsing

## [0.3.12] - 2025-09-21

### Fixed
- Update SLSA generator to v2.1.0 to resolve provenance generation failures
- Correct SLSA subjects format for v2.1.0 compatibility

## [0.3.8] - 2025-09-16

### Added
- SLSA Build Level 3 provenance generation for release artifacts

## [0.3.5] - 2025-08-29

### Changed
- Linux release binaries are now fully statically linked with musl

## [0.3.4] - 2025-08-29

### Added
- Auto-sync Dockerfile version label with the project version

### Changed
- Refreshed README screenshots

## [0.3.0] - 2025-08-09

### Added
- RDF 1.2 annotation syntax support in tests (named reifier `~` with `{| ... |}` blocks) for Turtle and TriG.
- Support for RDF 1.2 VERSION/@version directives in Turtle/TriG.
- Support for RDF 1.2 quoted triples with reification mapping via `rdf:reifies`.
- Tests for VERSION/@version and parenthesized triple terms `<<( ... )>>`.

### Changed
- Publish workflow now respects annotated release tags (vX.Y.Z); if a tag is present on the CI commit and matches Cargo.toml, it publishes that exact version without bumping.
- Bumped crate version to 0.3.0.
- Upgrade to oxttl 0.2.0-beta.2 and oxrdf 0.3.0-beta.2 with `rdf-12` feature to enable RDF 1.2.
- Switch from deprecated `Subject` to `NamedOrBlankNode` in internal conversion.
- Remove `.with_quoted_triples()` calls (RDF 1.2 is feature-gated now).

### Notes
- Quoted triple usage emits `rdf:reifies` mapping (RDF 1.2 semantics). TriG tests confirm graph propagation for annotation triples.
- RDF-star behavior now follows RDF 1.2: using `<< s p o >>` in subject/object positions emits an intermediate reifier node and an `rdf:reifies` triple; tests updated accordingly.

## [0.2.21] - 2025-08-08

### Changed
- Publish workflow builds .deb packages for Ubuntu 24.04 and 22.04

## [0.2.19] - 2025-08-05

### Fixed
- Ensure output uses `@prefix` instead of `PREFIX`

### Changed
- Enhanced TriG sample data

## [0.2.14] - 2025-06-30

### Added
- Migration from deprecated rio_api/rio_turtle to oxttl/oxrdf for improved RDF parsing
- RDF-star (embedded triples) support via `rdf-star` feature in oxttl/oxrdf
- Prefix parsing for Turtle (`@prefix`) and SPARQL (`PREFIX`) syntax
- Support for `@base` declarations
- Recursive embedded triple handling in triple/quad conversion
- Updated formatter and filter logic for RDF-star syntax (`<< ... >>`)

### Changed
- Parser modules updated to use oxttl/oxrdf APIs
- OwnedTriple structure updated to support embedded triples
- Parser APIs switched to iterator-based parsing and updated error types

### Testing
- Added prefix integration tests and RDF-star test coverage

### Notes
- RDF-star syntax uses `<< subject predicate object >>` format
- VERSION/@version directives from RDF 1.2 were not yet supported upstream at this time

## [0.2.13] - 2025-06-30

### Added
- `--keybindings` output and PAGER_KEYBINDINGS.md documentation
- Keybindings section in the man page

## [0.2.12] - 2025-06-30

### Fixed
- Restore missing CLI options for N-Triples and N-Quads

## [0.2.11] - 2025-06-29

### Added
- One-line install script for Linux
- `--completion` option for generating shell completion scripts

## [0.2.10] - 2025-06-29

### Added
- Dockerfile and helper Just tasks
- Initial screenshot automation script for README assets

## [0.2.9] - 2025-06-28

### Added
- Filtering by subject, predicate, or object with documentation updates

## [0.2.7] - 2025-06-27

### Added
- `--output` option to write formatted output to a file
- Man pages for rdfless and the configuration file

## [0.1.11] - 2025-06-27

### Added
- Multiple themes and pager support
- Automatic dark/light theme selection based on terminal background

## [0.1.5] - 2025-06-09

### Fixed
- Output formatting for integers, booleans, and related literals

### Changed
- Migrate from rio* crates to sophia* crates

## [0.1.3] - 2025-06-08

### Added
- 24-bit color support via #RRGGBB syntax

## [0.1.2] - 2025-06-08

### Fixed
- Default config file creation

## [0.1.1] - 2025-06-08

### Fixed
- TriG output formatting regression

### Added
- README screenshots

## [0.1.0] - 2025-06-08

### Added
- Initial public release
