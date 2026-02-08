# Performance Optimization Guide

## Performance Bottlenecks Identified

Based on code analysis, here are the main performance bottlenecks in mermaid_validator:

### 1. 🔥 **Mermaid CLI Process Spawning** (High Priority)
**Location:** `src/cli_runner.rs:117` - `render_diagram()`
**Issue:** Spawning the `mmdc` process is expensive
- Each validation requires a new process spawn
- Stdin/stdout/stderr pipe setup overhead
- Tokio task spawning for async I/O

**Optimization Strategies:**
- **Connection Pooling:** Implement a process pool to reuse mmdc instances
- **Batch Processing:** Process multiple diagrams in one mmdc call if possible
- **Process Persistence:** Keep mmdc running and send commands via stdin

### 2. 📄 **Markdown Parsing Overhead** (Medium Priority)
**Location:** `src/preview_validator.rs:223` - `collect_mermaid_blocks()`
**Issue:** Line-by-line parsing on every request
- String allocation for each line
- Repeated fence state transitions
- No caching for repeated markdown

**Optimization Strategies:**
- **String_view:** Use `&str` instead of `String` to avoid allocations
- **Precompiled Patterns:** Cache regex patterns for fence detection
- **Streaming Parser:** Parse incrementally for large files

### 3. 🧵 **Sequential Diagram Validation** (Medium Priority)
**Location:** `src/preview_validator.rs:149` - `validate_markdown_for_github()`
**Issue:** Diagrams validated one-by-one
- No parallel processing
- Each mmdc spawn waits for completion

**Optimization Strategies:**
- **Parallel Validation:** Use `tokio::spawn` to validate blocks concurrently
- **Semaphore:** Limit concurrent mmdc processes to avoid overwhelming system
- **Lazy Validation:** Only validate blocks that have changed

### 4. 📊 **Error Context Parsing** (Low Priority)
**Location:** `src/response_builder.rs:103` - `parse_error_context()`
**Issue:** Regex/string operations on every error
- Line-by-line scanning for error patterns
- String allocation for context building

**Optimization Strategies:**
- **Precompiled Regex:** Compile once, reuse
- **String_view:** Use borrowed slices
- **Lazy Parsing:** Parse only when needed

### 5. 🔄 **String Normalization** (Low Priority)
**Location:** `src/server.rs:250` - `normalize_diagram()`
**Issue:** Multiple passes over input string
- Fence detection
- Content extraction
- Validation

**Optimization Strategies:**
- **Single Pass:** Combine detection and extraction
- **Borrowed Slices:** Avoid string cloning
- **Pattern Matching:** Use optimized pattern matching

---

## Quick Wins (Can implement now)

### 1. Add Process Pool
```rust
// In cli_runner.rs
use std::sync::Arc;
use tokio::sync::Semaphore;

struct MermaidCliPool {
    semaphore: Arc<Semaphore>,
    timeout: Duration,
}

impl MermaidCliPool {
    pub async fn render(&self, diagram: &str, format: OutputFormat) -> Result<Vec<u8>, RenderError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        render_diagram(diagram, format, self.timeout).await
    }
}
```

### 2. Parallel Block Validation
```rust
// In preview_validator.rs
use tokio::task::JoinSet;

pub async fn validate_markdown_for_github(
    markdown: &str,
    timeout: Duration,
) -> PreviewValidationResult {
    let (blocks, issues) = collect_mermaid_blocks(markdown);
    let mut join_set = JoinSet::new();

    for block in &blocks {
        let timeout = timeout;
        let content = block.content.clone();
        join_set.spawn(async move {
            match render_diagram(&content, OutputFormat::Svg, timeout).await {
                Ok(_) => None,
                Err(err) => Some(build_mermaid_parse_issue(block, &err.to_error_message())),
            }
        });
    }

    // Collect results...
}
```

### 3. String Optimization
```rust
// In preview_validator.rs
fn collect_mermaid_blocks(markdown: &str) -> (Vec<MermaidBlock>, Vec<PreviewIssue>) {
    // Use &str slices instead of String allocation
    let mut blocks = Vec::new();
    let mut issues = Vec::new();

    for (idx, line) in markdown.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let trimmed_line = line.trim_start(); // Borrowed slice, no allocation

        // Use trimmed_line directly
        if let Some((marker, len, lang)) = parse_fence_start(trimmed_line) {
            // ...
        }
    }

    // ...
}
```

### 4. Precompiled Regex Patterns
```rust
// In response_builder.rs
use once_cell::sync::Lazy;
use regex::Regex;

static LINE_NUMBER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"parse error on line (\d+)").unwrap()
});

fn extract_line_number(line: &str) -> Option<u32> {
    LINE_NUMBER_RE.captures(line)
        .and_then(|cap| cap[1].parse().ok())
}
```

---

## Performance Testing Commands

### 1. Run Criterion Benchmarks
```bash
# Basic benchmarks
cargo bench

# With HTML output
cargo bench -- --output-format html

# Filter specific benchmarks
cargo bench -- render_simple_diagram

# Compare with previous run
cargo bench -- --baseline=main
```

### 2. Generate Flamegraphs
```bash
# Simple flamegraph
cargo flamegraph --bin mermaid_validator

# With specific options
cargo flamegraph --bin mermaid_validator -- --diagram "graph TD\nA-->B" --format svg

# Use the profiling script
./tools/perf/profile_flamegraph.sh
```

### 3. Benchmark Comparisons
```bash
# Compare performance between commits
cargo bench -- --baseline=HEAD~1

# Detailed comparison report
cargo bench > /tmp/old.txt
# ... make changes ...
cargo bench > /tmp/new.txt
cargo benchcmp /tmp/old.txt /tmp/new.txt
```

### 4. Advanced Profiling with Infer
```bash
# Install infer
cargo install infer

# Run with infer
cargo infer --release

# Analyze results
infer run -- cargo build --release
```

---

## Monitoring Performance in CI/CD

### Add to .github/workflows/perf.yml
```yaml
name: Performance Tests

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
      - name: Run benchmarks
        run: cargo bench -- --output-format json
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: target/criterion/
```

---

## Performance Budget

Recommended performance targets:

| Operation | Target Latency | Max Latency |
|-----------|---------------|-------------|
| Simple diagram render | < 100ms | 200ms |
| Complex diagram render | < 500ms | 1s |
| Markdown scan (10 blocks) | < 50ms | 100ms |
| Markdown validation (10 blocks) | < 1s | 2s |
| Error parsing | < 5ms | 10ms |

---

## Tools for Performance Analysis

1. **Criterion.rs** - Statistical benchmarking
2. **criterion.rs** - Detailed performance reports
3. **cargo-flamegraph** - Visualization of hot paths
4. **infer** - Advanced static analysis
5. **valgrind + cachegrind** - Cache simulation
6. **tokio-console** - Async runtime profiling

---

## Next Steps

1. ✅ Set up Criterion benchmarks (DONE)
2. 🔄 Profile with flamegraph (DONE)
3. 🚀 Implement process pooling
4. 🚀 Add parallel validation
5. 🚀 Optimize string operations
6. 🚀 Add caching layer
7. 📊 Set up CI/CD performance monitoring
8. 📈 Define performance SLAs
