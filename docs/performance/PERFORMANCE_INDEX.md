# 性能测试文档索引

## 文档与脚本位置

### 核心文档（本目录）
- **[PERFORMANCE_REPORT.md](PERFORMANCE_REPORT.md)**：本次优化前后对比报告
- **[PERFORMANCE.md](PERFORMANCE.md)**：性能优化方法与排查指南

### 基准与脚本（仓库根目录）
- **[../../benches/mermaid_bench.rs](../../benches/mermaid_bench.rs)**：Criterion 基准集合
- **[../../tools/perf/perf_report.sh](../../tools/perf/perf_report.sh)**：完整基准 + 文本总结
- **[../../tools/perf/quick_bench.sh](../../tools/perf/quick_bench.sh)**：快速检查脚本
- **[../../tools/perf/profile_flamegraph.sh](../../tools/perf/profile_flamegraph.sh)**：火焰图脚本
- **[../../tools/perf/examples/flamegraph_test.rs](../../tools/perf/examples/flamegraph_test.rs)**：火焰图测试样例

### 运行时产物（已加入 `.gitignore`）
- `target/criterion/`
- `flamegraphs/`
- `benchmark_output.log`
- `PERF_SUMMARY.txt`
- `cargo-flamegraph.trace/`

## 快速开始

```bash
# 1) 快速跑一轮性能检查
./tools/perf/quick_bench.sh

# 2) 生成完整性能总结
./tools/perf/perf_report.sh

# 3) 查看详细报告
cat docs/performance/PERFORMANCE_REPORT.md

# 4) 打开 HTML 基准报告
open target/criterion/report/index.html
```

## 说明
- `PERF_SUMMARY.txt` 与 `benchmark_output.log` 为临时输出，不建议提交。
- 正式对比结果请以 `docs/performance/PERFORMANCE_REPORT.md` 为准。
