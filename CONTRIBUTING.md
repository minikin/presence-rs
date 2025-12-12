# Contributing to Presence-RS

Thank you for your interest in contributing to Presence-RS! This document provides guidelines and instructions for contributing.

- [Contributing to Presence-RS](#contributing-to-presence-rs)
  - [Development Setup](#development-setup)
  - [Building and Testing](#building-and-testing)
    - [Run all tests](#run-all-tests)
    - [Run specific test suites](#run-specific-test-suites)
    - [Check code formatting](#check-code-formatting)
    - [Run linter](#run-linter)
    - [Build documentation](#build-documentation)
  - [Making Changes](#making-changes)
    - [Code Style](#code-style)
    - [Testing](#testing)
    - [Documentation](#documentation)
  - [Pull Request Process](#pull-request-process)
    - [PR Requirements](#pr-requirements)
  - [Feature Flags](#feature-flags)
  - [Commit Message Format](#commit-message-format)
  - [Release Process](#release-process)
  - [Getting Help](#getting-help)
  - [Code of Conduct](#code-of-conduct)
  - [License](#license)

## Development Setup

**Note:** This project uses Rust edition 2024.

1. Install Rust via [rustup](https://rustup.rs/)
2. Install and use nightly Rust (required for edition 2024):
   ```bash
   rustup install nightly
   rustup default nightly
   ```
3. Clone the repository:
   ```bash
   git clone https://github.com/mnkn/presence-rs.git
   cd presence-rs
   ```
4. Install development dependencies:
   ```bash
   rustup component add rustfmt clippy
   ```

## Building and Testing

### Run all tests
```bash
cargo test --all-features
```

### Run specific test suites
```bash
# Run only integration tests
cargo test --tests --all-features

# Run only unit tests
cargo test --lib

# Run a specific test file
cargo test --test basic_tests
```

### Check code formatting
```bash
cargo fmt --all -- --check
```

### Run linter
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Build documentation
```bash
cargo doc --all-features --no-deps --open
```

## Making Changes

### Code Style

- Follow Rust's official [style guidelines](https://doc.rust-lang.org/1.0.0/style/)
- Run `cargo fmt` before committing
- Ensure `cargo clippy` produces no warnings
- Write clear, descriptive commit messages

### Testing

- Add tests for all new functionality
- Ensure all existing tests pass
- Integration tests go in the `tests/` directory
- Unit tests go in the same file as the code they test

### Documentation

- Document all public APIs with doc comments
- Include examples in doc comments where appropriate
- Update README.md if adding major features

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for your changes
5. Run the full test suite: `cargo test --all-features`
6. Run formatting and linting:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   ```
7. Commit your changes with a descriptive message
8. Push to your fork
9. Open a Pull Request against the `main` branch

### PR Requirements

- All CI checks must pass
- Code must be formatted with `rustfmt`
- No `clippy` warnings
- All tests must pass on Linux, macOS, and Windows
- Code coverage should not decrease
- Documentation is updated if needed

## Feature Flags

The project uses feature flags for optional functionality:

- `serde`: Serialization/deserialization support

When adding new features:
- Make them optional via feature flags when appropriate
- Test with and without the feature enabled
- Document the feature in README.md

## Commit Message Format

Use conventional commits format:

```
type(scope): subject

body

footer
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

Examples:
```
feat(core): add new is_nullish method
fix(serde): correct deserialization of absent fields
docs(readme): update usage examples
test(conversion): add tests for from_nullable
```

## Release Process

Releases are managed by maintainers:

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag: `git tag -a v0.x.y -m "Release v0.x.y"`
4. Push tag: `git push origin v0.x.y`
5. GitHub Actions will automatically publish to crates.io

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Review existing issues before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help create a welcoming environment for all contributors

## License

By contributing, you agree that your contributions will be dual-licensed under MIT OR Apache-2.0, matching the project's license.
