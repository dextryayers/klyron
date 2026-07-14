# Contributing to Klyron

Thank you for considering contributing to Klyron! This document outlines the process for contributing to the project.

## Development Environment Setup

### Prerequisites

- **Rust** (latest stable): Install via [rustup](https://rustup.rs/)
- **Node.js** 20+ (for JavaScript/TypeScript tooling)
- **PNPM** (for package management)
- **Make** (for build scripts)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/dextryayers/klyron.git
cd klyron

# Install Rust toolchain
rustup install stable
rustup default stable

# Install development dependencies
make setup

# Build the project
make build
```

## Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with specific engine support
cargo build --features v8
cargo build --features quickjs
cargo build --features boa

# Fast iterative builds (default-members only)
cargo build -p klyron_cli -p klyron_engine
```

## Running Tests

```bash
# Run all tests
make test

# Run Rust tests only
cargo test

# Run specific crate tests
cargo test -p klyron_core
cargo test -p klyron_engine

# Run integration tests
cargo test --test integration

# Run with all features
cargo test --all-features
```

## Code Style and Conventions

### Rust

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `rustfmt` with the project's `rustfmt.toml` configuration
- Run `cargo clippy` before submitting code
- All public APIs must have documentation comments
- Use `anyhow::Result` for error propagation

### General

- Use async/await patterns where appropriate
- Prefer `deno_core` patterns for JavaScript interop
- Use `tracing` for structured logging (not `println`)
- Write idiomatic Rust; avoid unsafe code unless absolutely necessary

### Formatting

```bash
# Format Rust code
cargo fmt

# Run linter
cargo clippy --all-targets -- -D warnings
```

## Pull Request Process

1. **Fork the repository** and create your branch from `main`
2. **Run tests** to ensure nothing is broken
3. **Update documentation** if you change functionality
4. **Update PLAN.md** if your changes affect the project roadmap
5. **Submit a pull request** with a clear description of changes

### PR Requirements

- All tests must pass
- Code must be formatted with `cargo fmt`
- No new clippy warnings
- Meaningful commit messages (see format below)
- Changes should be focused and atomic

### Review Process

- Maintainers will review your PR within 5 business days
- Address review feedback promptly
- Once approved, a maintainer will merge your PR

## Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Adding or updating tests
- `chore` - Build process or tooling changes
- `ci` - CI configuration changes

### Examples

```
feat(engine): add QuickJS engine support
fix(runtime): resolve memory leak in module loader
docs(cli): update command reference
test(core): add unit tests for cache layer
```

## Project Structure

```
klyron/
├── crates/           # Core Rust crates (refactored)
├── src/              # Legacy source (being migrated)
│   ├── cli/          # CLI implementation
│   ├── core/         # Core runtime
│   └── ext/          # Extension modules
├── runtime/          # JavaScript runtime layer
├── engines/          # Script engine implementations
├── adapters/         # Platform adapters
├── registry/         # Package registry
├── storage/          # Storage backends
├── sdk/              # SDK implementations
├── orm/              # ORM layer
└── polyglot/         # Polyglot support
```

## Getting Help

- Open an issue on GitHub
- Join the community discussions

Thank you for contributing!
