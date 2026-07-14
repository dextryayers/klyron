# Changelog

## [Unreleased]

### Added
- Engine switching: `--engine v8|boa|quickjs|jsc|auto` flag for `eval`/`run`/`repl` commands
- Criterion.rs benchmark suite for JS engine comparison (startup + eval performance)
- Laravel ecosystem commands: Horizon, Telescope, Reverb, Pulse, Pennant, Breeze, Jetstream, Sail
- 20+ Artisan `make:*` command wrappers (controller, model, migration, seeder, etc.)
- `klyron bench engine` subcommand for quick JS engine comparison

### Changed
- `klyron info` now shows available JS engines and best detected engine
- REPL supports `.engine <name>` to switch engines at runtime
- `klyron_engine::JsEngine` trait no longer requires `Send + Sync` (supports Boa engine)

### Fixed
- `klyron_engine` all-features compilation (resolved `Adapter::new` name conflicts)
- `klyron_fs::copy_with_progress` accepts `FnMut` progress callback
- `DeployConfig` and `DockerConfig` missing field errors in CLI
- `Linter::lint_fix` correct API usage in lint command
- `BenchmarkRunner::run_micro` correct API usage in bench command
- `TestRunner` API alignment in test command
