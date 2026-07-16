# Contributing to Klyron

Thank you for your interest in contributing to Klyron! This document outlines the development process, code style, and community guidelines.

## Code of Conduct

All contributors are expected to adhere to our Code of Conduct. Be respectful, inclusive, and constructive in all interactions.

## How to Contribute

### Report Bugs

- Check existing issues before filing a new one
- Use the bug report template
- Include minimal reproduction steps
- Include Klyron version (`klyron --version`) and platform information

### Suggest Features

- Check existing feature requests
- Describe the problem you're solving, not just a solution
- Explain how it fits into Klyron's architecture
- Be open to discussion and alternative approaches

### Submit Code Changes

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs))
- Cargo (included with Rust)
- Node.js 18+ (for integration tests)
- PHP 8.1+ (for PHP adapter tests)

### Clone and Build

```bash
git clone https://github.com/anomalyco/klyron.git
cd klyron
cargo build
```

### Run Tests

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features boa-engine

# Run integration tests
cargo test --test integration
```

### Development Workflow

```bash
# Build in debug mode (fast iteration)
cargo build

# Build in release mode (production)
cargo build --release

# Run the dev server for testing
cargo run -- dev --port 5174

# Run a specific command
cargo run -- run examples/hello.ts
```

### Code Formatting

```bash
# Format all Rust code
cargo fmt

# Lint
cargo clippy --all-features
```

## Project Structure

```
klyron/
├── Cargo.toml              # Workspace root
├── crates/                 # Internal library crates
│   ├── klyron-core/        # Core runtime types and traits
│   ├── klyron-engine/      # Multi-engine abstraction
│   ├── klyron-package/     # Package manager
│   ├── klyron-bundler/     # Build system
│   ├── klyron-test/        # Test runner
│   ├── klyron-adapter/     # Adapter system
│   └── klyron-plugin/      # Plugin system (WASM)
├── src/                    # CLI entry point
├── engines/                # Engine bindings (V8, Boa, QuickJS, JSC)
├── adapters/               # Framework adapters
├── plugins/                # Built-in plugins
├── scaffolds/              # Project templates
└── tests/                  # Integration tests
```

## Code Style

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo clippy` -- no warnings allowed
- Prefer `anyhow::Result` for CLI code, custom error types for library code
- Document public APIs with doc comments
- Use `tracing` for logging, not `println!`

### JavaScript/TypeScript

- Use TypeScript for all new code
- Follow the project's ESLint configuration
- Prefer `const` over `let` where possible
- Use async/await over raw promises

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(engine): add QuickJS support
fix(package): resolve hoisting conflict with scoped packages
docs(guides): add migration-from-npm guide
test(bundler): add snapshot tests for ESM output
chore(deps): update v8 crate to 0.85.0
```

## Pull Request Process

1. Ensure your branch is up to date with `main`
2. Run `cargo test` and `cargo clippy` -- all must pass
3. Add tests for new functionality
4. Update documentation if you change public APIs
5. Keep PRs focused on a single change
6. PRs require at least one approval from a maintainer

### PR Title Format

Use conventional commit format for PR titles (the title will be used for the squash merge commit).

### Review Process

- Maintainers review within 3 business days
- Address review feedback promptly
- Use `fixup!` commits during review, we squash on merge

## Community Guidelines

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community support
- **Discord**: Real-time chat (invite link in README)

### Governance

Klyron is maintained by the core team at Anomaly Co. Decision-making follows a consensus-based model with maintainer oversight for architectural decisions.

### Licensing

By contributing, you agree that your contributions will be licensed under the same license as the project (see LICENSE.md).
