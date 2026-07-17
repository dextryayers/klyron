<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/dextryayers/klyron/main/assets/logo-dark.svg">
    <img alt="Klyron" src="https://raw.githubusercontent.com/dextryayers/klyron/main/assets/logo-light.svg" width="400">
  </picture>
</p>

<p align="center">
  <strong>Universal Polyglot Runtime</strong><br>
  Run JavaScript, TypeScript, PHP, Python, Ruby, Go, Zig, Rust, C, and C++ — all from a single runtime.
</p>

<p align="center">
  <a href="https://github.com/dextryayers/klyron/actions"><img src="https://img.shields.io/github/actions/workflow/status/dextryayers/klyron/ci.yml?branch=main&style=flat-square&color=blue" alt="Build Status"></a>
  <a href="https://crates.io/crates/klyron"><img src="https://img.shields.io/crates/v/klyron?style=flat-square&color=blue" alt="Crates.io"></a>
  <a href="https://github.com/dextryayers/klyron/blob/main/LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT License"></a>
  <a href="https://klyron.dev"><img src="https://img.shields.io/badge/docs-klyron.dev-blue?style=flat-square" alt="Docs"></a>
  <a href="https://github.com/dextryayers/klyron"><img src="https://img.shields.io/github/stars/dextryayers/klyron?style=flat-square&color=blue" alt="GitHub Stars"></a>
</p>

---

## Quick Start

```bash
# Install globally via npm (macOS / Linux / Windows)
npm install -g klyron

# Or via curl (macOS / Linux / WSL)
curl -fsSL https://klyron.dev/install.sh | bash

# Or via PowerShell (Windows)
iwr -useb https://klyron.dev/install.ps1 | iex

# Or via Cargo
cargo install klyron-cli

# Verify installation
klyron --version

# Run your first script
klyron run hello.js
```

## Features

- **Polyglot Execution** — Run 10 languages from a single runtime without manually installing separate interpreters
- **Unified HTTP API** — Write servers once using `Klyron.serve()` that work across all languages
- **Built-in Package Manager** — No need for npm, pip, gem; use `klyron install` for everything
- **Scaffolding System** — Generate full-stack apps, APIs, microservices, and more with `klyron scaffold`
- **Hot Reload** — Watch mode with automatic restarts on file changes
- **Bundling & Transpilation** — Built-in bundler and transpiler for JS/TS
- **Plugin System** — Extend Klyron with native Rust plugins
- **AI Integration** — Built-in crate for LLM-powered features (`klyron_ai`)
- **Docker Support** — Seamless containerization
- **Workspace Management** — Monorepo support out of the box

## Usage

### Run JavaScript

```bash
klyron run hello.js
```

### Run TypeScript

```bash
klyron run hello.ts
```

### HTTP Server

```javascript
// server.js
Klyron.serve({ port: 3000 }, (req) => {
  return new Response("Hello from Klyron!", {
    headers: { "Content-Type": "text/plain" },
  });
});
```

```bash
klyron run server.js
```

### Scaffold a Project

```bash
klyron scaffold my-app --template react
klyron scaffold my-api --template api
```

### Package Management

```bash
klyron install lodash
klyron install flask
klyron install requests
```

## Supported Languages

| Language     | Extension  | Status |
|-------------|-----------|--------|
| JavaScript   | `.js`     | ✅     |
| TypeScript   | `.ts`     | ✅     |
| PHP          | `.php`    | ✅     |
| Python       | `.py`     | ✅     |
| Ruby         | `.rb`     | ✅     |
| Go           | `.go`     | ✅     |
| Zig          | `.zig`    | ✅     |
| Rust         | `.rs`     | ✅     |
| C            | `.c`      | ✅     |
| C++          | `.cc` `.cpp` `.cxx` `.hpp` | ✅ |

## Documentation

- [Getting Started](https://klyron.dev/docs/getting-started)
- [CLI Reference](https://klyron.dev/docs/cli)
- [HTTP API](https://klyron.dev/docs/http)
- [Package Manager](https://klyron.dev/docs/packages)
- [Scaffolding](https://klyron.dev/docs/scaffold)
- [Plugin Development](https://klyron.dev/docs/plugins)
- [API Reference](https://klyron.dev/docs/api)

## Examples

```bash
# Clone and explore
git clone https://github.com/dextryayers/klyron.git
cd klyron/examples/01-hello-world
klyron run hello.js
```

Check the [examples](./examples) directory for:
- [`01-hello-world`](./examples/01-hello-world/) — Polyglot hello world in 8 languages
- [`02-http-server`](./examples/02-http-server/) — HTTP server with Klyron's unified API
- [`03-react-app`](./examples/03-react-app/) — Full React frontend
- [`04-laravel-app`](./examples/04-laravel-app/) — Laravel PHP application
- [`05-next-app`](./examples/05-next-app/) — Next.js full-stack app
- [`06-fullstack`](./examples/06-fullstack/) — Full-stack monolith
- [`07-microservices`](./examples/07-microservices/) — Microservices architecture
- [`08-desktop`](./examples/08-desktop/) — Desktop application

## Contributing

We welcome contributions from the community!

1. **Fork** the repository
2. **Create a branch** (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to your branch (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

Please read our [Contributing Guide](https://klyron.dev/docs/contributing) for details on our code of conduct and development process.

### Development Setup

```bash
git clone https://github.com/dextryayers/klyron.git
cd klyron
cargo build
cargo test
```

## License

This project is licensed under the [MIT License](./LICENSE.md) — see the [LICENSE](./LICENSE.md) file for details.

---

<p align="center">
  <a href="https://klyron.dev">klyron.dev</a> &nbsp;·&nbsp;
  <a href="https://github.com/dextryayers/klyron">GitHub</a> &nbsp;·&nbsp;
  <a href="https://github.com/dextryayers/klyron/issues">Issues</a> &nbsp;·&nbsp;
  <a href="https://github.com/dextryayers/klyron/discussions">Discussions</a>
</p>
