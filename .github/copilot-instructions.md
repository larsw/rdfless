---
applyTo: "**/.rs"
---

# Copilot instructions for Rust code in the rdfless project

- Always fix tests if you touch production code and ensure that they run OK.
- Always run `cargo clippy` and `cargo fmt` and fix warnings and errors.
- Always update README and man files if you touch relevant functionality or command line options.
- Always update the `CHANGELOG.md` file with a summary of changes.
- Always update the version in `Cargo.toml` when you make changes that warrant a new release.
- Always ensure that the code is documented where needed, especially public functions and modules.
- Always ensure that the code is idiomatic Rust, following best practices and conventions.
- Always ensure that the code is efficient and does not introduce unnecessary complexity.
- Always ensure that the code is secure and does not introduce vulnerabilities.