# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **Rust MCP (Model Context Protocol) server** that validates Mermaid diagrams and GitHub-style Markdown preview compatibility. It uses the official Mermaid CLI (`mmdc`) to parse and render diagrams.

**Key Features:**
- Validates single Mermaid diagrams and returns rendered images (SVG/PNG)
- Validates GitHub-style Markdown preview behavior
- Scans Markdown files for Mermaid blocks
- Provides detailed error reporting with line/column positions

## Common Commands

### Build & Run
```bash
# Build the project
cargo build

# Run the MCP server directly from source (recommended for development)
cargo run --quiet

# Build and install locally
cargo install --path .
```

### Testing
```bash
# Run all tests
cargo test

# Note: Some tests require `mmdc` to be installed and available in PATH
```

### Performance Testing & Benchmarking
```bash
# Run Criterion benchmarks (measures function execution time)
cargo bench

# Generate HTML benchmark reports
cargo bench -- --output-format html

# Compare benchmark results between versions
cargo bench > /tmp/old.txt
# ... make changes ...
cargo bench > /tmp/new.txt
cargo benchcmp /tmp/old.txt /tmp/new.txt

# Profile with flamegraph (visual call stack analysis)
cargo flamegraph --bin mermaid_validator -- --diagram "graph TD\nA-->B" --format svg

# Use the profiling script (comprehensive profiling)
./tools/perf/profile_flamegraph.sh
```

**Prerequisites for full testing:**
```bash
# Install Mermaid CLI
npm install -g @mermaid-js/mermaid-cli

# Verify installation
mmdc --version
```

### Environment Variables
- `MERMAID_CLI` (default: `mmdc`) - Path to mermaid-cli executable
- `MERMAID_TIMEOUT` (default: `30s`) - Timeout for diagram rendering (supports `10`, `10s`, `250ms`)

## Code Architecture

### Core Components

**1. Server Layer (`src/server.rs`)**
- Implements MCP server with 4 tools using the `rmcp` framework
- Routes tool calls to appropriate handlers
- Manages input validation and normalization

**2. CLI Runner (`src/cli_runner.rs`)**
- Interface to the `mmdc` (Mermaid CLI) executable
- Handles async process spawning and I/O
- Implements timeout management and error handling
- Supports SVG and PNG output formats

**3. Preview Validator (`src/preview_validator.rs`)**
- Scans Markdown for Mermaid code blocks
- Detects fence issues (unclosed blocks, invalid syntax)
- Validates diagrams against GitHub preview rules
- Maps parse errors to Markdown line numbers

**4. Response Builder (`src/response_builder.rs`)**
- Formats MCP tool results
- Handles error message parsing and presentation
- Extracts line/column information from Mermaid errors

**5. Main Entry (`src/main.rs`)**
- Minimal entry point that initializes the MCP server
- Uses stdio transport for Claude Desktop integration

### Data Flow

```
Client Request
    ↓
server.rs (MCP Server)
    ├─→ normalize_diagram() - Extract/fence Mermaid code
    ├─→ cli_runner.rs - Spawn mmdc, render diagram
    ├─→ preview_validator.rs - Scan markdown, detect blocks
    └─→ response_builder.rs - Format results/errors
    ↓
MCP Response (CallToolResult)
```

### Module Dependencies

```
server.rs
    ├── cli_runner.rs (render_diagram, OutputFormat)
    ├── preview_validator.rs (scan/validate markdown)
    └── response_builder.rs (format responses)
```

## MCP Tools

The server exposes 4 tools:

1. **validateMermaid** - Validates and renders a single diagram
   - Input: `diagram` (string), `format` (optional: "svg" | "png")
   - Output: Base64-encoded image

2. **validateMermaidPreview** - Validates GitHub-style Markdown
   - Input: `markdown` (string)
   - Output: Validation status and issues list

3. **scanMermaidBlocks** - Scans Markdown file for Mermaid blocks
   - Input: `file_path` (string)
   - Output: List of blocks with positions

4. **validateMermaidBlock** - Validates specific block by index
   - Input: `file_path` (string), `block_index` (number)
   - Output: Validation result for that block

## Development Notes

### Error Handling
- `RenderError` enum in `cli_runner.rs` handles all mmdc process errors
- Parse errors include line/column information mapped to Markdown
- Unclosed fence detection happens before rendering

### Testing Strategy
- Unit tests for pure functions (normalization, parsing)
- Integration tests with mmdc (skipped if not available)
- Async tests using tokio test runtime

### Key Patterns
- **Fence State Machine** - Tracks code block boundaries in Markdown
- **Error Context Parsing** - Extracts structured info from mmdc stderr
- **Async Process Management** - Spawns mmdc with stdin/stdout/stderr pipes

## Dependencies

- `rmcp = "0.14.0"` - MCP protocol server/client framework
- `tokio = "1.43.0"` - Async runtime with process support
- `serde + schemars` - JSON serialization with schema generation
- `base64` - Encode rendered images
- `thiserror` - Error enum derive macro

## Configuration

**Claude Desktop Integration:**
```json
{
  "mcpServers": {
    "mermaid-validator-rs": {
      "command": "cargo",
      "args": ["run", "--quiet", "--manifest-path", "/path/to/mermaid_validator_rs/Cargo.toml"],
      "env": {
        "MERMAID_TIMEOUT": "30s",
        "MERMAID_CLI": "mmdc"
      }
    }
  }
}
```

## Important Implementation Details

### Diagram Normalization
The server handles multiple input formats:
- Plain Mermaid syntax
- Fenced code blocks: ````mermaid`...```
- Embedded in Markdown

It automatically extracts the Mermaid content and validates fence completeness.

### GitHub Preview Validation
- Only `error` severity issues affect validity
- Unclosed fences are detected before rendering
- Parse errors are mapped to Markdown line numbers
- Multiple blocks are indexed starting from 1

### Process Management
- mmdc is spawned with stdin/stdout/stderr piped
- All I/O happens concurrently using tokio tasks
- Timeout is enforced via tokio::select
- Errors include stderr output for debugging

## Performance Optimization

### Performance Bottlenecks (Identified)

1. **Mermaid CLI Process Spawning** (`src/cli_runner.rs:117`)
   - Each validation spawns a new `mmdc` process
   - Spawn overhead: ~10-20ms per process
   - **Optimization:** Implement process pooling to reuse mmdc instances

2. **Sequential Markdown Validation** (`src/preview_validator.rs:149`)
   - Diagrams validated one-by-one
   - No parallel processing of blocks
   - **Optimization:** Use `tokio::spawn` for concurrent validation

3. **Markdown Parsing Overhead** (`src/preview_validator.rs:223`)
   - Line-by-line parsing on every request
   - String allocations for each line
   - **Optimization:** Use `&str` slices and precompile patterns

4. **String Normalization** (`src/server.rs:250`)
   - Multiple passes over input string
   - **Optimization:** Single-pass detection and extraction

### Performance Targets

| Operation | Target Latency | Max Latency |
|-----------|---------------|-------------|
| Simple diagram render | < 100ms | 200ms |
| Complex diagram render | < 500ms | 1s |
| Markdown scan (10 blocks) | < 50ms | 100ms |
| Markdown validation (10 blocks) | < 1s | 2s |

### Quick Optimization Wins

```rust
// 1. Process Pooling
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct MermaidCliPool {
    semaphore: Arc<Semaphore>,
}

impl MermaidCliPool {
    pub async fn render(&self, diagram: &str, format: OutputFormat) -> Result<Vec<u8>, RenderError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        render_diagram(diagram, format, self.timeout).await
    }
}

// 2. Parallel Validation
use tokio::task::JoinSet;

pub async fn validate_markdown_parallel(
    blocks: &[MermaidBlock],
    timeout: Duration,
) -> Vec<Option<PreviewIssue>> {
    let mut join_set = JoinSet::new();
    for block in blocks {
        join_set.spawn(render_and_collect(block, timeout));
    }
    // Collect results...
}
```

See `docs/performance/PERFORMANCE.md` for detailed optimization guide.
