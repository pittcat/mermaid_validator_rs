#!/bin/bash

# Flamegraph Profiling Script for Mermaid Validator
# This script helps identify performance bottlenecks in the mermaid_validator

set -e

echo "🔥 Flamegraph Profiling Script for Mermaid Validator"
echo "=================================================="
echo ""

# Check if required tools are installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "❌ cargo-flamegraph not found"
    echo "Installing cargo-flamegraph..."
    cargo install flamegraph
fi

if ! command -v infer &> /dev/null; then
    echo "⚠️  infer not found - for more advanced profiling"
    echo "Install with: cargo install infer"
fi

# Create flamegraph output directory
mkdir -p flamegraphs

echo "📊 Running performance profiling..."
echo ""

echo "1. Profiling basic mermaid validation (SVG)"
echo "   - Creating flamegraph for render_diagram"
timeout 60 cargo flamegraph --bin mermaid_validator -- --diagram "graph TD\nA-->B" --format svg 2>&1 | tee flamegraphs/basic_validation.log || true

echo ""
echo "2. Profiling complex mermaid validation"
timeout 60 cargo flamegraph --bin mermaid_validator -- --diagram "graph TB\nA[Start] --> B{Decision}\nB -->|Yes| C[Process 1]\nB -->|No| D[Process 2]" --format png 2>&1 | tee flamegraphs/complex_validation.log || true

echo ""
echo "3. Profiling markdown preview validation"
cat > /tmp/test_markdown.md << 'EOF'
# Test

```mermaid
graph TD
A --> B
```

```mermaid
flowchart LR
X --> Y
```
EOF

timeout 60 cargo flamegraph --bin mermaid_validator -- --markdown-preview-file /tmp/test_markdown.md 2>&1 | tee flamegraphs/markdown_validation.log || true

echo ""
echo "4. Running Criterion benchmarks with flamegraph"
echo "   - This will take a few minutes..."
timeout 120 cargo bench -- --profile-time 5 2>&1 | tee flamegraphs/benchmark.log || true

echo ""
echo "✅ Profiling complete!"
echo ""
echo "📈 Results saved to:"
echo "   - flamegraphs/*.svg (flamegraph visualizations)"
echo "   - flamegraphs/*.log (detailed logs)"
echo ""
echo "📊 To view flamegraphs:"
echo "   - Open flamegraph.svg files in a web browser"
echo "   - Use 'flamegraph.pl' script for advanced analysis"
echo ""
echo "💡 For deeper analysis:"
echo "   - Install infer: cargo install infer"
echo "   - Run: cargo infer --release"
echo "   - Analyze output in target/infer-out/"
echo ""

# Optional: Generate HTML report if criterion is used
if [ -d "target/criterion" ]; then
    echo "📋 Criterion reports:"
    find target/criterion -name "*.html" -type f | head -10 || true
    echo ""
    echo "View reports by opening these files in a browser"
fi
