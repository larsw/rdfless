# CI/CD Setup for rdfless

## Overview

This document provides an overview of the CI/CD setup that has been implemented for the rdfless project using GitHub Actions.

## What Has Been Implemented

1. **Continuous Integration (CI) Workflow**
   - File: `.github/workflows/ci.yml`
   - Triggered on: Pushes to main branch and pull requests targeting main branch
   - Jobs:
     - Testing: Runs all tests with `cargo test`
     - Formatting: Checks code formatting with `rustfmt`
     - Linting: Runs Clippy to catch common mistakes
     - Building: Builds the project on Linux, Windows, and macOS

2. **Continuous Deployment (CD) Workflow**
   - File: `.github/workflows/publish.yml`
   - Triggered on: Merge commits to main branch (via pull requests)
   - Jobs:
     - Publishing: Automatically publishes new versions to crates.io

3. **Documentation**
   - Added CI/CD status badges to README.md
   - Created CONTRIBUTING.md with guidelines for contributors

## How to Enable the CI/CD Setup

1. **Push the Changes to GitHub**
   - The workflow files will be automatically recognized by GitHub Actions

2. **Set Up the Required Secret**
   - For the publishing workflow to work, you need to add a repository secret:
     1. Generate a new token on [crates.io](https://crates.io/me/tokens)
     2. Go to your GitHub repository settings
     3. Navigate to Secrets and variables > Actions
     4. Add a new repository secret with the name `CRATES_IO_TOKEN` and the value of your crates.io token

## How It Works

### Publishing a New Version

1. Update the version in `Cargo.toml` (e.g., from 0.1.3 to 0.1.4)
2. Create a pull request with your changes
3. Once the PR is approved and merged to main:
   - The CI workflow will run to ensure everything works
   - The CD workflow will check if the version is new
   - If the version is new, it will automatically publish to crates.io

### Workflow Customization

If you need to customize the workflows:

1. Edit the workflow files in the `.github/workflows/` directory
2. Commit and push your changes
3. GitHub Actions will use the updated workflow files for future runs

## Benefits

- **Automated Testing**: Ensures code quality and prevents regressions
- **Cross-Platform Compatibility**: Verifies the project works on multiple operating systems
- **Simplified Releases**: Automates the publishing process
- **Improved Collaboration**: Makes it easier for contributors to submit high-quality PRs

## Next Steps

Consider implementing additional CI/CD features:

- Code coverage reporting
- Dependency scanning for security vulnerabilities
- Automated changelog generation
- Release asset creation (pre-built binaries)