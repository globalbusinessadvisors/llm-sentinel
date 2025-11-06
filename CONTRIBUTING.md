# Contributing to LLM-Sentinel

Thank you for your interest in contributing to LLM-Sentinel! This document provides guidelines and instructions for contributing to this project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Code Style](#code-style)
- [Documentation](#documentation)
- [Security](#security)

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. We expect all contributors to:

- Be respectful and considerate
- Welcome newcomers and help them get started
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Docker and Docker Compose (for running dependencies)
- Git

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/llm-sentinel.git
cd llm-sentinel
```

3. Add the upstream repository:

```bash
git remote add upstream https://github.com/llm-devops/llm-sentinel.git
```

4. Create a new branch for your changes:

```bash
git checkout -b feature/your-feature-name
```

## Development Environment

### Quick Setup

1. Install dependencies:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add rustfmt clippy
```

2. Start development infrastructure:

```bash
docker-compose up -d
```

3. Build the project:

```bash
cargo build
```

4. Run tests:

```bash
cargo test
```

### IDE Setup

#### VS Code

Recommended extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- crates (for dependency management)
- Even Better TOML

#### IntelliJ IDEA / CLion

Install the Rust plugin from the marketplace.

## Making Changes

### Branch Naming

Use descriptive branch names with prefixes:

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions or modifications
- `chore/` - Maintenance tasks

Examples:
- `feature/add-isolation-forest-detector`
- `fix/kafka-connection-timeout`
- `docs/update-deployment-guide`

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code restructuring without behavior change
- `test`: Adding or updating tests
- `chore`: Maintenance tasks, dependency updates

Examples:

```
feat(detection): add isolation forest anomaly detector

Implement a new detector based on scikit-learn's Isolation Forest
algorithm for detecting anomalies in high-dimensional data.

Closes #123
```

```
fix(ingestion): handle Kafka connection timeout gracefully

Previously, the application would panic when Kafka was unavailable.
Now it retries with exponential backoff.

Fixes #456
```

### Keep Changes Focused

- Make small, focused commits
- One feature or fix per pull request
- Keep pull requests under 400 lines when possible
- Split large changes into multiple PRs

## Testing

### Running Tests

Run all tests:

```bash
cargo test
```

Run specific test:

```bash
cargo test test_name
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run tests with specific features:

```bash
cargo test --features redis
```

### Writing Tests

1. **Unit Tests**: Place alongside the code being tested

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore_detection() {
        let detector = ZScoreDetector::new(3.0);
        // Test implementation
    }
}
```

2. **Integration Tests**: Place in `tests/` directory

```rust
// tests/kafka_integration.rs
#[tokio::test]
async fn test_kafka_ingestion() {
    // Integration test implementation
}
```

3. **Property-Based Tests**: Use proptest for property-based testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_baseline_never_panics(samples in prop::collection::vec(any::<f64>(), 0..1000)) {
        let baseline = Baseline::new(1000);
        baseline.update(samples);
    }
}
```

### Test Coverage

We aim for:
- 80%+ code coverage overall
- 90%+ coverage for core detection algorithms
- 100% coverage for critical paths (data loss prevention, security)

Check coverage:

```bash
cargo tarpaulin --out Html --output-dir coverage
```

## Submitting Changes

### Before Submitting

1. **Format your code**:

```bash
cargo fmt
```

2. **Run clippy**:

```bash
cargo clippy -- -D warnings
```

3. **Run tests**:

```bash
cargo test
```

4. **Update documentation**:

```bash
cargo doc --no-deps --open
```

5. **Check for security issues**:

```bash
cargo audit
```

### Pull Request Process

1. **Update your fork**:

```bash
git fetch upstream
git rebase upstream/main
```

2. **Push your changes**:

```bash
git push origin feature/your-feature-name
```

3. **Create a pull request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what and why
   - Reference to related issues
   - Screenshots for UI changes
   - Test results

4. **PR Template**:

```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- Change 1
- Change 2

## Testing
How was this tested?

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Code formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
```

5. **Address review feedback**:
   - Respond to all comments
   - Make requested changes
   - Push updates to the same branch

6. **Merge requirements**:
   - All CI checks must pass
   - At least one approval from a maintainer
   - No unresolved conversations
   - Branch is up to date with main

## Code Style

### Rust Style Guide

We follow the [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md) with these additions:

1. **Imports**: Group and sort imports

```rust
// Standard library
use std::collections::HashMap;
use std::sync::Arc;

// External crates
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// Internal modules
use crate::core::error::SentinelError;
use crate::detection::baseline::Baseline;
```

2. **Error Handling**: Use Result types, avoid unwrap() in production code

```rust
// Good
pub fn process(&self) -> Result<Output> {
    let data = self.fetch()?;
    Ok(data.process())
}

// Bad
pub fn process(&self) -> Output {
    let data = self.fetch().unwrap();
    data.process()
}
```

3. **Documentation**: Document all public APIs

```rust
/// Detects anomalies using the Z-score method.
///
/// # Arguments
///
/// * `baseline` - The statistical baseline for comparison
/// * `value` - The value to test for anomalies
///
/// # Returns
///
/// Returns `Some(anomaly)` if detected, `None` otherwise.
///
/// # Examples
///
/// ```
/// let detector = ZScoreDetector::new(3.0);
/// let anomaly = detector.detect(&baseline, 100.0);
/// ```
pub fn detect(&self, baseline: &Baseline, value: f64) -> Option<Anomaly> {
    // Implementation
}
```

4. **Async Code**: Use async/await consistently

```rust
// Good
async fn process_event(&self, event: Event) -> Result<()> {
    let validated = self.validate(event).await?;
    self.store(validated).await
}

// Bad
fn process_event(&self, event: Event) -> impl Future<Output = Result<()>> {
    async move {
        // Implementation
    }
}
```

5. **Constants**: Use SCREAMING_SNAKE_CASE

```rust
const MAX_RETRY_ATTEMPTS: u32 = 3;
const DEFAULT_TIMEOUT_MS: u64 = 5000;
```

### Configuration

- Use `rustfmt.toml` at repository root
- Use `clippy.toml` for clippy configuration
- Enable all clippy lints, disable specific ones with justification

## Documentation

### Code Documentation

- Document all public functions, structs, and modules
- Include examples in doc comments
- Use proper markdown formatting
- Link to related types using backticks

### User Documentation

- Update relevant docs in `/docs` directory
- Include examples and use cases
- Keep documentation in sync with code
- Add diagrams where helpful

### API Documentation

Generate and review docs:

```bash
cargo doc --no-deps --open
```

## Security

### Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities.

Instead, email security@llm-devops.io with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours.

### Security Best Practices

1. **Input Validation**: Always validate and sanitize inputs
2. **Dependencies**: Keep dependencies up to date
3. **Secrets**: Never commit secrets or credentials
4. **Error Messages**: Don't leak sensitive information in errors
5. **Authentication**: Use proper authentication mechanisms
6. **Encryption**: Use TLS for network communication

### Security Scanning

We use:
- `cargo audit` for dependency vulnerabilities
- `cargo clippy` for common security issues
- GitHub Dependabot for automated updates
- Security scanning in CI/CD

## Release Process

Releases are managed by maintainers:

1. Version bump in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release tag
4. Publish to crates.io (if applicable)
5. Build and publish Docker images
6. Update documentation

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Security**: Email security@llm-devops.io
- **Chat**: Join our community Slack (link in README)

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project README

## License

By contributing to LLM-Sentinel, you agree that your contributions will be licensed under the Apache License 2.0.

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Project Documentation](./docs/)

Thank you for contributing to LLM-Sentinel!
