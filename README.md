# Mermaid Validator MCP (Rust)

A Rust MCP server for Mermaid validation and GitHub preview checks.
It uses the official Mermaid CLI (`mmdc`) for parsing/rendering behavior.

For Chinese documentation, see `README_zh.md`.

## Features

- `validateMermaid`: validate a single Mermaid diagram (`svg`/`png`)
- `validateMermaidPreview`: validate Mermaid preview behavior in GitHub-style Markdown
- `scanMermaidBlocks`: scan Mermaid code blocks from a Markdown file path
- `validateMermaidBlock`: validate one Mermaid block by block index from a file path

## Requirements

- Rust `1.75+`
- Node.js (for Mermaid CLI)
- `mmdc` available in `PATH`

Install `mmdc`:

```bash
npm install -g @mermaid-js/mermaid-cli
```

Verify:

```bash
mmdc --version
```

## Install This MCP Server

Build/install locally:

```bash
cargo install --path .
```

Run directly from source:

```bash
cargo run --quiet
```

## Claude Desktop Configuration

Claude Desktop config file (macOS):

- `~/Library/Application Support/Claude/claude_desktop_config.json`

### Option A: Run from source (recommended for development)

```json
{
  "mcpServers": {
    "mermaid-validator-rs": {
      "command": "cargo",
      "args": [
        "run",
        "--quiet",
        "--manifest-path",
        "/path/to/mermaid_validator_rs/Cargo.toml"
      ],
      "env": {
        "MERMAID_TIMEOUT": "30s",
        "MERMAID_CLI": "mmdc"
      }
    }
  }
}
```

### Option B: Use installed binary

```json
{
  "mcpServers": {
    "mermaid-validator-rs": {
      "command": "mermaid_validator",
      "args": [],
      "env": {
        "MERMAID_TIMEOUT": "30s",
        "MERMAID_CLI": "mmdc"
      }
    }
  }
}
```

## Tool Usage

### 1) `validateMermaid`

Input:

```json
{
  "diagram": "graph TD\\nA[Start] --> B[End]",
  "format": "png"
}
```

### 2) `validateMermaidPreview`

Input:

```json
{
  "markdown": "## Diagram\\n\\n```mermaid\\ngraph TD\\nA-->B\\n```\\n"
}
```

### 3) Large file workflow (avoid context explosion)

Step 1: scan blocks

```json
{
  "filePath": "/path/to/interact.md"
}
```

Step 2: validate one block

```json
{
  "filePath": "/path/to/interact.md",
  "blockIndex": 2
}
```

## Validation Rules (GitHub Preview)

`validateMermaidPreview` and path-based tools use GitHub-style Markdown assumptions:

- detect unclosed Mermaid fences
- validate all Mermaid blocks (or one selected block)
- only `error` affects final validity (`valid = false`)

## Environment Variables

- `MERMAID_CLI` (default: `mmdc`)
- `MERMAID_TIMEOUT` (default: `30s`, supports `10`, `10s`, `250ms`)

## Test

```bash
cargo test
```

## Performance Testing

### Benchmark Tests

```bash
# Run all benchmarks
cargo bench

# Generate HTML report with charts
cargo bench -- --output-format html

# Compare performance between versions
cargo bench > /tmp/old.txt
# ... make changes ...
cargo bench > /tmp/new.txt
cargo benchcmp /tmp/old.txt /tmp/new.txt
```

### Flamegraph Profiling

```bash
# Install flamegraph tool
cargo install flamegraph

# Profile specific operations
cargo flamegraph --bin mermaid_validator -- --diagram "graph TD\nA-->B" --format svg

# Run comprehensive profiling script
./tools/perf/profile_flamegraph.sh
```

### Performance Bottlenecks

Key performance issues identified:

1. **Mermaid CLI Process Spawning** - Each validation spawns a new process (~10-20ms overhead)
2. **Sequential Validation** - Markdown blocks validated one-by-one
3. **String Parsing** - Multiple allocations during Markdown parsing

For detailed optimization strategies, see [PERFORMANCE.md](./docs/performance/PERFORMANCE.md).
