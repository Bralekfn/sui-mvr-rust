# Contributing to sui-mvr

First off, thank you for considering contributing to sui-mvr! 🎉

This project aims to provide the best MVR (Move Registry) experience for Rust developers in the Sui ecosystem. We welcome contributions from everyone, whether you're fixing a typo, adding a feature, or improving documentation.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Be respectful, inclusive, and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Git
- A GitHub account

### Setting up your development environment

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/sui-mvr-rust.git
   cd sui-mvr-rust
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/Bralekfn/sui-mvr-rust.git
   ```
4. **Install dependencies and test**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

## Development Process

1. **Create a branch** for your work:
   ```bash
   git checkout -b feature/amazing-feature
   # or
   git checkout -b bugfix/fix-important-bug
   ```

2. **Make your changes** following our coding standards

3. **Test your changes**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

4. **Run examples** to ensure they still work:
   ```bash
   cargo run --example basic_usage
   cargo run --example with_overrides
   cargo run --example batch_operations
   ```

5. **Commit your changes**:
   ```bash
   git add .
   git commit -m "Add amazing feature"
   ```

6. **Push to your fork**:
   ```bash
   git push origin feature/amazing-feature
   ```

7. **Create a Pull Request** on GitHub

## Pull Request Process

### Before submitting

- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated if needed
- [ ] CHANGELOG.md is updated for user-facing changes
- [ ] Examples work and are updated if needed

### PR Guidelines

1. **Title**: Use a clear, descriptive title
   - Good: "Add batch type resolution with caching"
   - Bad: "Fix stuff"

2. **Description**: Include:
   - What changes you made
   - Why you made them
   - How to test the changes
   - Any breaking changes

3. **Size**: Keep PRs focused and reasonably sized
   - Large PRs should be discussed in an issue first

4. **Tests**: Include tests for new functionality

### Review Process

- All PRs require at least one review
- Reviews focus on correctness, performance, and maintainability
- We may ask for changes or suggest improvements
- Once approved, a maintainer will merge your PR

## Issue Guidelines

### Before creating an issue

- Search existing issues to avoid duplicates
- Try to reproduce the issue with the latest version
- Gather relevant information (Rust version, OS, error messages)

### Issue Types

**🐛 Bug Reports**
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Environment details

**✨ Feature Requests**
- Clear description of the proposed feature
- Use case and motivation
- Possible implementation approach

**📚 Documentation**
- What documentation needs improvement
- Specific suggestions for changes

**❓ Questions**
- For usage questions, consider GitHub Discussions first
- Include context about what you're trying to achieve

## Code Style

### Rust Guidelines

We follow standard Rust conventions:

- **Formatting**: Use `cargo fmt` with default settings
- **Linting**: Address all `cargo clippy` warnings
- **Naming**: Follow Rust naming conventions
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Code Organization

- **Modules**: Keep modules focused and well-defined
- **Functions**: Small, single-purpose functions
- **Error Handling**: Use `Result` types, avoid panics in library code
- **Documentation**: Document all public APIs

### Example Code Style

```rust
/// Resolve a package name to its address
///
/// # Arguments
/// * `package_name` - The MVR package name (e.g., "@suifrens/core")
///
/// # Returns
/// * `Ok(String)` - The resolved package address
/// * `Err(MvrError)` - If resolution fails
///
/// # Example
/// ```rust
/// use sui_mvr::MvrResolver;
/// 
/// # tokio_test::block_on(async {
/// let resolver = MvrResolver::mainnet();
/// let address = resolver.resolve_package("@suifrens/core").await?;
/// println!("Package address: {}", address);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
pub async fn resolve_package(&self, package_name: &str) -> MvrResult<String> {
    validate_package_name(package_name)?;
    // ... implementation
}
```

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions/modules
2. **Integration Tests**: Test complete workflows
3. **Example Tests**: Ensure examples work correctly
4. **Documentation Tests**: Test code in documentation

### Writing Tests

```rust
#[tokio::test]
async fn test_package_resolution() {
    let resolver = create_test_resolver();
    let result = resolver.resolve_package("@test/package").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0x123456");
}

#[test]
fn test_error_handling() {
    let error = MvrError::PackageNotFound("test".to_string());
    assert!(error.is_client_error());
    assert!(!error.is_retryable());
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_package_resolution

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

## Documentation

### API Documentation

- Document all public functions, structs, and traits
- Include examples in doc comments
- Use `cargo doc --open` to preview

### README Updates

- Keep examples current
- Update feature lists
- Maintain installation instructions

### Examples

- Examples should be self-contained
- Include error handling
- Add comments explaining key concepts

## Release Process

### Version Numbers

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Test on multiple Rust versions
- [ ] Run full test suite
- [ ] Create release PR
- [ ] Tag release after merge
- [ ] Publish to crates.io
- [ ] Update documentation

## Getting Help

- **Questions**: Use GitHub Discussions
- **Bugs**: Create an issue with reproduction steps
- **Features**: Open an issue to discuss before implementing
- **Chat**: Find us in the Sui Discord #rust-dev channel

## Recognition

Contributors are recognized in:
- CHANGELOG.md for their contributions
- README.md contributors section
- GitHub contributors page

Thank you for contributing to sui-mvr! 🦀