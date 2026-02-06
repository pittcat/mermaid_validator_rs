# Mermaid Validator MCP (Rust)

这是一个 Rust 实现的 Mermaid MCP 服务端，用于 Mermaid 语法校验和 GitHub 预览兼容性检查。
底层校验/渲染使用官方 Mermaid CLI（`mmdc`）。

英文文档请见 `README.md`。

## 功能

- `validateMermaid`：校验单段 Mermaid 图（支持 `svg`/`png`）
- `validateMermaidPreview`：按 GitHub Markdown 预览语义校验
- `scanMermaidBlocks`：按文件路径扫描 Mermaid 代码块
- `validateMermaidBlock`：按块索引校验指定 Mermaid 代码块

## 依赖

- Rust `1.75+`
- Node.js（用于 Mermaid CLI）
- `mmdc` 可在 `PATH` 中找到

安装 `mmdc`：

```bash
npm install -g @mermaid-js/mermaid-cli
```

验证：

```bash
mmdc --version
```

## 安装并运行 MCP 服务

本地安装：

```bash
cargo install --path .
```

从源码直接运行：

```bash
cargo run --quiet
```

## Claude Desktop 配置

Claude Desktop 配置文件（macOS）：

- `~/Library/Application Support/Claude/claude_desktop_config.json`

### 方式 A：源码运行（开发调试推荐）

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

### 方式 B：使用已安装二进制

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

## 工具调用示例

### 1) `validateMermaid`

```json
{
  "diagram": "graph TD\\nA[Start] --> B[End]",
  "format": "png"
}
```

### 2) `validateMermaidPreview`

```json
{
  "markdown": "## Diagram\\n\\n```mermaid\\ngraph TD\\nA-->B\\n```\\n"
}
```

### 3) 大文件建议流程（避免上下文爆炸）

先扫描块：

```json
{
  "filePath": "/path/to/interact.md"
}
```

再校验指定块：

```json
{
  "filePath": "/path/to/interact.md",
  "blockIndex": 2
}
```

## 预览校验规则（GitHub）

`validateMermaidPreview` 及路径模式工具按 GitHub 预览语义处理：

- 检测 Mermaid fence 是否闭合
- 可校验全部 Mermaid 代码块或单块
- 仅 `error` 影响最终结果（`valid = false`）

## 环境变量

- `MERMAID_CLI`（默认：`mmdc`）
- `MERMAID_TIMEOUT`（默认：`30s`，支持 `10`、`10s`、`250ms`）

## 测试

```bash
cargo test
```
