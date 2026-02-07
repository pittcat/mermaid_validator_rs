# Repository Guidelines

## Project Structure & Module Organization
Core code lives in `src/`:
- `main.rs`: stdio server entrypoint.
- `server.rs`: MCP tool definitions and routing (`validateMermaid`, preview tools).
- `cli_runner.rs`: `mmdc` process execution, timeout/env handling, output format logic.
- `preview_validator.rs`: Markdown fence scanning and GitHub-preview validation rules.
- `response_builder.rs`: user-facing MCP response shaping.
- `lib.rs`: module exports.

Integration tests are in `tests/` (`mermaid_validator_tests.rs`, `preview_validator_tests.rs`). Project metadata is in `Cargo.toml`; runtime defaults are controlled via environment variables (not hardcoded config files).

## Build, Test, and Development Commands
- `cargo check`: fast compile/type check.
- `cargo build`: build debug binary.
- `cargo run --quiet`: run MCP server over stdio locally.
- `cargo test`: run unit + integration tests.
- `cargo fmt --all`: format code with Rustfmt.
- `cargo clippy --all-targets --all-features -D warnings`: lint with warnings treated as errors.
- `cargo install --path .`: install local binary (`mermaid_validator`).

Prerequisite for rendering/parse tests: `mmdc` in `PATH` (`npm i -g @mermaid-js/mermaid-cli`).

## Coding Style & Naming Conventions
Use Rust 2021 idioms and Rustfmt defaults (4-space indentation, trailing commas where formatter applies).  
Naming:
- `snake_case` for functions/modules/files.
- `UpperCamelCase` for structs/enums.
- `SCREAMING_SNAKE_CASE` for constants.

Keep JSON/MCP parameter structs in `server.rs` with `serde(rename_all = "camelCase")` for wire compatibility. Prefer small, single-purpose functions and explicit error messages.

## Testing Guidelines
Write unit tests next to implementation (`#[cfg(test)]`) and cross-module behavior tests in `tests/`.  
Use descriptive test names like `preview_reports_parse_error_location`.  
Async flows should use `#[tokio::test]`.  
Tests that require Mermaid CLI should gracefully skip when `mmdc` is unavailable (current pattern in integration tests). No fixed coverage threshold is enforced; add tests for every behavior change.

## Commit & Pull Request Guidelines
Recent history uses concise, imperative commit subjects (English or Chinese), e.g. `Update project documentation and configuration`. Keep subject lines short and specific; add a body when behavior or protocol contracts change.

PRs should include:
- What changed and why.
- How to verify (`cargo test`, manual MCP call examples if relevant).
- Any env/config impacts (`MERMAID_CLI`, `MERMAID_TIMEOUT`).
