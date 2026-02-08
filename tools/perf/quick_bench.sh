#!/bin/bash

# Quick Performance Test Script
# Runs basic benchmarks and displays results

set -e

echo "🚀 Mermaid Validator - Quick Performance Test"
echo "=============================================="
echo ""

# Check if mmdc is available
if ! command -v mmdc &> /dev/null; then
    echo "⚠️  Warning: mmdc not found"
    echo "   Install with: npm install -g @mermaid-js/mermaid-cli"
    echo "   Benchmarks will run but may be slower"
    echo ""
fi

echo "📊 Running benchmarks..."
echo ""

# Run benchmarks with timeout
echo "1. Basic diagram rendering benchmark"
timeout 300 cargo bench -- render_simple_diagram 2>&1 | tail -20 || echo "⚠️  Benchmark timed out or failed"

echo ""
echo "2. Markdown parsing benchmark"
timeout 300 cargo bench -- collect_mermaid_blocks 2>&1 | tail -20 || echo "⚠️  Benchmark timed out or failed"

echo ""
echo "3. Complex diagram benchmark"
timeout 300 cargo bench -- render_complex_diagram 2>&1 | tail -20 || echo "⚠️  Benchmark timed out or failed"

echo ""
echo "✅ Quick benchmarks complete!"
echo ""

# Check if results exist
if [ -d "target/criterion" ]; then
    echo "📈 Benchmark results saved to: target/criterion/"
    echo ""
    echo "To view detailed reports:"
    echo "  cargo bench -- --output-format html"
    echo ""
    echo "To generate flamegraphs:"
    echo "  cargo install flamegraph"
    echo "  cargo flamegraph --bin mermaid_validator"
fi

# Performance summary
echo ""
echo "💡 Performance Tips:"
echo "  - Monitor 'time' column in benchmark results"
echo "  - Lower is better (measured in nanoseconds)"
echo "  - Compare results with: cargo benchcmp before.txt after.txt"
echo "  - Use flamegraph to identify hot paths: ./tools/perf/profile_flamegraph.sh"
echo ""

# Memory usage check
echo "📊 Current Memory Usage:"
if command -v ps &> /dev/null; then
    ps -o pid,rss,vsz,comm -p $$ | tail -1
fi

echo ""
echo "🔍 Next Steps:"
echo "  1. Review benchmark results in target/criterion/"
echo "  2. Generate flamegraph: ./tools/perf/profile_flamegraph.sh"
echo "  3. Optimize hot paths identified in profiling"
echo "  4. Re-run benchmarks to verify improvements"
echo ""
