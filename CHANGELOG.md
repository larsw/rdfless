# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.14] - 2025-06-30

### Added
- Migration from deprecated rio_api/rio_turtle to oxttl/oxrdf for improved RDF parsing
- RDF-star (embedded triples) support via `rdf-star` feature in oxttl/oxrdf
- Enhanced prefix parsing functionality with custom extraction algorithms
- Support for both Turtle (`@prefix`) and SPARQL (`PREFIX`) syntax
- Comprehensive prefix integration into Turtle and TriG parsers
- Extended data model for RDF-star with support for embedded triples in subject and object positions
- New triple/quad conversion functions with recursive embedded triple handling
- Updated formatter and filter logic for RDF-star syntax (`<< ... >>`)
- **RESTORED**: Prefix parsing functionality now implemented directly in the utility
- Support for both Turtle (@prefix) and SPARQL (PREFIX) syntax
- Support for @base declarations
- Comprehensive test suite for prefix extraction functionality
- **NEW**: RDF-star (embedded triples) support enabled for Turtle and TriG formats
- Support for embedded triples in both subject and object positions

### Changed
- **BREAKING**: Replaced rio_api/rio_turtle with oxttl/oxrdf dependencies  
- Updated all parser modules (turtle.rs, trig.rs, ntriples.rs, nquads.rs, robust.rs) to use oxttl/oxrdf APIs
- Enhanced OwnedTriple structure to support embedded triples with new SubjectType/ObjectType enums
- Improved triple_to_owned and quad_to_owned functions for recursive embedded triple handling
- Updated formatter output to properly display RDF-star syntax
- Enhanced filter functionality to work with new triple/object types
- **BREAKING**: Migrated from deprecated `rio_api` and `rio_turtle` packages to `oxttl` 0.1.8 and `oxrdf` 0.2.4
- Updated all RDF parsing functionality to use the new oxttl APIs
- Parser APIs now use iterator-based approach instead of callback-based approach
- Error handling updated to work with oxttl error types
- Initial migration from rio_api/rio_turtle to oxttl/oxrdf (prefix support was missing)

### Testing
- Added comprehensive test suite for prefix extraction and integration (tests/prefix_test.rs, tests/prefix_integration_test.rs)
- Added RDF-star test coverage (tests/rdf_star_test.rs)
- Updated all existing test files to support new OwnedTriple structure
- Verified compatibility with legacy and new RDF features

### Technical Notes
- RDF-star syntax: Uses `<< subject predicate object >>` format (not `<<( ... )>>`)
- Prefix extraction supports mixed Turtle and SPARQL syntax in same document
- All clippy warnings resolved and code properly formatted
- VERSION/@version directives from RDF 1.2 not yet supported by oxttl/oxrdf (tests skipped upstream)
- All parsers (Turtle, N-Triples, N-Quads, TriG) now use oxttl parsers
- Updated robust parsing functionality to work with new parser APIs
- Test suite updated and all tests passing
- Code formatting and linting issues resolved

## [0.2.13] - Previous release
