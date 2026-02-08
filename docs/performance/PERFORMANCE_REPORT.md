# 📊 性能优化对比报告（2026-02-08）

## 1. 执行目标
在**不改变现有功能行为**的前提下，对关键字符串解析路径做性能优化，并与当前 `PERFORMANCE_REPORT.md`（旧版）进行对比。

## 2. 本次优化项（无功能变更）

### 2.1 `src/preview_validator.rs`
- `FenceState` 将 `Vec<String>` 改为 `Vec<&str>`，避免逐行 `to_string()` 分配。
- 代码块语言判定改为 `is_mermaid: bool`，避免重复字符串构造与重复判断。
- `is_fence_close` 改为字节级前缀扫描。
- `split_error_details` 改为 `split_once`，减少中间 `Vec`/`join` 分配。
- `parse_error_context` 改为单次遍历（同时提取 line/snippet/column/reason）。
- `extract_line_number` 移除 `to_ascii_lowercase` 和数字字符串构造，改为字节扫描解析。

### 2.2 `src/response_builder.rs`
- 同步应用 `split_once`、单次遍历错误上下文解析、字节级行号提取优化。

### 2.3 `benches/mermaid_bench.rs`
- 新增真实代码路径基准：
  - `scan_markdown_small`
  - `scan_markdown_large`
  - `invalid_result_parse_error`
  - `invalid_result_unknown_diagram`

## 3. 功能回归结果
- 执行命令：`cargo test`
- 结果：**全部通过**（lib/unit/integration 全通过）。

## 4. Benchmark 执行命令
- 优化前基线（旧微基准）：`cargo bench --bench mermaid_bench`（保存到 `/tmp/bench_before.txt`）
- 优化前基线（真实路径）：`cargo bench --bench mermaid_bench -- 'scan_markdown|invalid_result'`（`/tmp/bench_real_before.txt`）
- 优化后：`cargo bench --bench mermaid_bench`（`/tmp/bench_after.txt`）

## 5. 与旧版报告（当前仓库报告）对比

| 指标 | 旧版报告 | 优化后 | 变化 |
|---|---:|---:|---:|
| `parse_error_context` | 221 ns | 218.33 ns | **-1.2%** |
| `find_reason` | 543 ns | 539.46 ns | **-0.7%** |
| `normalize_fenced_diagram` | 86 ns | 79.97 ns | **-7.0%** |
| `string_parsing_split` | 96.3 ns | 96.18 ns | -0.1% |
| `string_join` | 320 ns | 323.90 ns | +1.2% |

> 说明：旧版报告是历史采样，和本轮机器状态/采样窗口不完全一致，因此该表用于“方向性对比”。

## 6. 同环境优化前后对比（本轮最可信）

### 6.1 新增真实路径基准
| 指标 | 优化前 | 优化后 | 提升 |
|---|---:|---:|---:|
| `scan_markdown_small` | 314.93 ns | 252.56 ns | **-19.8%** |
| `scan_markdown_large` | 1.6671 µs | 1.3055 µs | **-21.7%** |
| `invalid_result_parse_error` | 1.4732 µs | 1.2376 µs | **-16.0%** |
| `invalid_result_unknown_diagram` | 678.80 ns | 554.14 ns | **-18.4%** |

### 6.2 旧微基准中的相关项
| 指标 | 优化前 | 优化后 | 变化 |
|---|---:|---:|---:|
| `parse_error_context` | 236.29 ns | 218.33 ns | **-7.6%** |
| `find_reason` | 540.90 ns | 539.46 ns | -0.3% |
| `string_join` | 324.02 ns | 323.90 ns | ~0% |
| `string_concat_100` | 3.8244 µs | 3.6759 µs | -3.9% |
| `hashmap_insert_100` | 5.7496 µs | 5.8865 µs | +2.4% |

## 7. 结论
- 本次优化在**真实业务路径**上有明确收益（约 **16% ~ 22%**）。
- 功能行为未受影响（`cargo test` 全量通过）。
- 旧报告中的部分合成微基准变化较小或有噪声，符合预期；核心路径已实测显著提升。

---
测试时间：2026-02-08 02:45:31 CST
