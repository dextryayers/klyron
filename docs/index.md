# Klyron

**Universal Polyglot Runtime & Developer Platform**

Klyron is a next-generation runtime and toolchain that unifies the developer experience across JavaScript, TypeScript, PHP, Rust, and more. It combines the speed of a modern runtime with the convenience of an all-in-one build tool, package manager, and test runner.

## Key Features

Klyron is a 7-in-1 fusion of the best developer tools:

- **Engine** -- Multi-engine runtime supporting V8, Boa, QuickJS, and JavaScriptCore, with seamless switching per project or per file.
- **Package Manager** -- Fast, disk-efficient package management with pnpm-compatible symlinking, lockfile guarantees, and workspace support.
- **Dev Server** -- Instant dev server with Hot Module Replacement (HMR), auto-detecting your framework (React, Vue, Svelte, Laravel, Inertia).
- **Build System** -- Production bundler with tree-shaking, minification, sourcemaps, and multi-target output (ESM, CJS, IIFE, WASM).
- **Test Runner** -- Built-in test runner with assertion library, coverage, watch mode, and multi-framework adapters (Vitest, Jest, PHPUnit, pytest).
- **Adapter System** -- Plug-and-play adapters for Laravel, Rails, Django, and other frameworks, providing tight integration out of the box.
- **Security Sandbox** -- Permission-based sandbox with network, filesystem, and process isolation, plus supply-chain security via TUF, SRI, and package signing.

## Quick Start

```bash
# Install Klyron
cargo install klyron

# Create your first project
klyron create react my-app
cd my-app

# Start development
klyron dev

# Build for production
klyron build

# Run tests
klyron test
```

## Documentation

| Section | Description |
|---|---|
| [Getting Started](getting-started.md) | Installation, first project, configuration |
| [Commands](commands/) | CLI reference for run, dev, build, install, test |
| [Guides](guides/) | Task-oriented guides for common workflows |
| [Architecture](architecture.md) | System design, engine layer, adapters, security |
| [API Reference](api/) | Rust and JavaScript API documentation |
| [Contributing](contributing.md) | How to contribute to Klyron |
| [Security](security.md) | Security policies and vulnerability reporting |
| [Changelog](changelog/) | Release notes and version history |
