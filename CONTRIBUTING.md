# Contributing to rdfless

Thank you for your interest in contributing to rdfless! This document provides guidelines and explains the CI/CD setup for the project.

## Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests locally
5. Submit a pull request

## CI/CD Setup

This project uses GitHub Actions for continuous integration and deployment.

### CI Workflow

The CI workflow runs on every push to the main branch and on every pull request targeting the main branch. It performs the following checks:

1. **Testing**: Runs all tests to ensure functionality works as expected
2. **Formatting**: Checks that the code follows Rust's formatting guidelines using `rustfmt`
3. **Linting**: Uses Clippy to catch common mistakes and improve code quality
4. **Building**: Builds the project on multiple platforms (Linux, Windows, macOS)

If any of these checks fail, the pull request cannot be merged until the issues are fixed.

### CD Workflow (Publishing)

The CD workflow automatically publishes new versions to crates.io when code is merged to the main branch via a pull request. The workflow:

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