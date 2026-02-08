#!/bin/bash

# Performance Report Generator and Viewer
# 自动生成性能测试报告

set -e

echo "📊 性能测试报告生成器"
echo "================================"
echo ""

# 1. 运行基准测试
echo "🚀 步骤 1: 运行基准测试..."
cargo bench 2>&1 | tee benchmark_output.log
echo ""

# 2. 生成性能总结
echo "📝 步骤 2: 生成性能总结报告..."
cat > PERF_SUMMARY.txt << 'EOF'
==========================================
   MERMAID VALIDATOR 性能测试总结
==========================================

测试时间: $(date)
测试环境: Rust $(rustc --version)

基准测试结果:
--------------
EOF

# 提取关键指标
grep -E "(Benchmarking|time:)" benchmark_output.log >> PERF_SUMMARY.txt

cat >> PERF_SUMMARY.txt << 'EOF'

性能瓶颈 (按耗时排序):
----------------------
EOF

# 提取耗时最长的测试
grep "time:" benchmark_output.log | grep -v "ps\|ns" | sort -k2 -r | head -10 >> PERF_SUMMARY.txt || true

cat >> PERF_SUMMARY.txt << 'EOF'

建议优化项目:
------------
1. string_concat_100 - 字符串连接优化 (3.77 µs)
2. hashmap_insert_100 - HashMap 重用 (5.89 µs)
3. find_reason - 预编译正则 (543 ns)
4. string_parsing_split - memchr 优化 (96.3 ns)

详细报告:
---------
- HTML 报告: target/criterion/report/index.html
- 完整报告: docs/performance/PERFORMANCE_REPORT.md
- 优化指南: docs/performance/PERFORMANCE.md
EOF

echo "✅ 性能总结已保存到 PERF_SUMMARY.txt"
echo ""

# 3. 生成 HTML 报告
echo "🎨 步骤 3: 检查 HTML 报告..."
if [ -f "target/criterion/report/index.html" ]; then
    echo "✅ HTML 报告已生成: target/criterion/report/index.html"
    echo ""
    echo "📖 打开报告的方法:"
    echo "   macOS: open target/criterion/report/index.html"
    echo "   Linux: xdg-open target/criterion/report/index.html"
    echo ""
else
    echo "⚠️  HTML 报告未找到，运行以下命令生成:"
    echo "   cargo install cargo-criterion"
    echo "   cargo criterion -- --output-format html"
    echo ""
fi

# 4. 显示性能总结
echo "📊 步骤 4: 性能总结..."
echo ""
cat PERF_SUMMARY.txt
echo ""

# 5. 提供优化建议
echo "💡 步骤 5: 优化建议..."
echo ""
echo "立即可执行的优化:"
echo "  1. 编辑 src/preview_validator.rs:"
echo "     - 减少 string_join 调用"
echo "     - 使用 &str 代替 String"
echo "     - 预分配 String 容量"
echo ""
echo "  2. 编辑 src/response_builder.rs:"
echo "     - 预编译正则表达式"
echo "     - 使用 once_cell::sync::Lazy"
echo ""
echo "  3. 内存优化:"
echo "     - 重用 HashMap"
echo "     - 使用对象池"
echo ""

# 6. 下一步行动
echo "🎯 下一步行动:"
echo ""
echo "  1. 查看详细报告:    cat docs/performance/PERFORMANCE_REPORT.md"
echo "  2. 查看优化指南:    cat docs/performance/PERFORMANCE.md"
echo "  3. 实施优化建议"
echo "  4. 重新运行测试:    ./tools/perf/perf_report.sh"
echo "  5. 对比性能变化:    cargo benchcmp before.txt after.txt"
echo ""

echo "✅ 性能测试报告生成完成!"
echo ""
