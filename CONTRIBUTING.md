# Contributing to rdfless

Thank you for your interest in contributing to rdfless! This document provides guidelines and explains the CI/CD setup for the project.

## Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests locally
5. Submit a pull request

### Build System

This project supports both traditional Make and the modern Just command runner:

#### Using Just (Recommended)

[Just](https://github.com/casey/just) is a modern command runner that provides a simpler and more readable syntax:

```bash
# Install Just
cargo install just

# Development workflow (format, lint, test, build)
just dev

# Individual tasks
just fmt      # Format code
just clippy   # Run linter
just test     # Run tests
just build    # Build release binary

# See all available commands
just list
```

#### Using Make (Legacy)

You can also use the traditional Makefile:

```bash
make fmt
make clippy
make build
make dist
```

### Running Tests

Always run the full test suite before submitting a PR:

```bash
just test
# or
make test
```

## CI/CD Setup

This project uses GitHub Actions for continuous integration and deployment. The workflows now use [Just](https://github.com/casey/just) for consistent build commands.

### CI Workflow

The CI workflow runs on every push to the main branch and on every pull request targeting the main branch. It performs the following checks:

1. **Testing**: Runs `just test` to ensure functionality works as expected
2. **Formatting**: Runs `just fmt --check` to verify code follows Rust's formatting guidelines
3. **Linting**: Runs `just clippy` to catch common mistakes and improve code quality
4. **Building**: Runs `just build` on multiple platforms (Linux, Windows, macOS)
5. **Dev Workflow**: Runs `just dev` to test the complete development workflow

If any of these checks fail, the pull request cannot be merged until the issues are fixed.

### CD Workflow (Publishing)

The CD workflow automatically publishes new versions to crates.io when code is merged to the main branch via a pull request. The workflow uses Just commands for all build operations:

- `just dist-linux` - Build Linux distribution with UPX compression
- `just dist-windows` - Build Windows distribution with cross-compilation
- `just dist-macos` - Build macOS distribution with cross-compilation

1. Only runs on merge commits to the main branch
2. Checks if the version in Cargo.toml has been updated
3. Publishes to crates.io if the version is new

## Version Management

When making changes that should be released:

1. Update the version in `Cargo.toml` following [Semantic Versioning](https://semver.org/) principles:
   - MAJOR version for incompatible API changes
   - MINOR version for backwards-compatible functionality additions
   - PATCH version for backwards-compatible bug fixes

2. The CD workflow will automatically publish the new version when your PR is merged.

## Setting up Secrets

For the CD workflow to publish to crates.io, a repository secret named `CRATES_IO_TOKEN` must be set up:

1. Generate a new token on [crates.io](https://crates.io/me/tokens)
2. Go to your GitHub repository settings
3. Navigate to Secrets and variables > Actions
4. Add a new repository secret with the name `CRATES_IO_TOKEN` and the value of your crates.io token

## Questions?

If you have any questions about contributing, please open an issue in the repository.