# Mermaid Validator (Rust)

用 Rust + MCP SDK 重写 Mermaid Validator，使用官方 `mermaid-cli` (`mmdc`) 进行校验与渲染，保持 `validateMermaid` 工具与原 TypeScript 版本兼容。

## 依赖

- Rust 1.75+
- `mmdc` CLI（来自 `@mermaid-js/mermaid-cli`），需在 `PATH` 中可用

安装：
```bash
npm install -g @mermaid-js/mermaid-cli
```

验证：
```bash
mmdc --version
```

## 运行

```bash
cargo run
```

## 安装（本地）

```bash
cargo install --path .
```

## MCP 工具

- `validateMermaid`
  - 参数：
    - `diagram` (string)
    - `format` ("svg" | "png", 可选，默认 png)
  - 返回：
    - 文本 + 图片内容（base64）

- `validateMermaidPreview`
  - 目标：按 GitHub Markdown 预览语义做校验（支持多个 Mermaid 代码块）
  - 参数：
    - `markdown` (string)
  - 返回：
    - 文本摘要 + `structuredContent`
    - `structuredContent` 关键字段：
      - `target` (`github`)
      - `valid` (bool)
      - `errorCount` (number)
      - `mermaidBlockCount` (number)
      - `issues` (array，含 `code/message/line/column/snippet/blockIndex`)

- `scanMermaidBlocks`
  - 目标：只传文件路径，扫描 Markdown 中 Mermaid 代码块位置
  - 参数：
    - `filePath` (string)
  - 返回：
    - 文本摘要 + `structuredContent`
    - `structuredContent` 包含：
      - `mermaidBlockCount`
      - `blocks`（`index/startLine/endLine/lineCount/firstLine`）
      - `issues`（例如 `mermaid_unclosed_fence`）

- `validateMermaidBlock`
  - 目标：只校验指定块，避免整文件传参
  - 参数：
    - `filePath` (string)
    - `blockIndex` (number)
  - 返回：
    - 文本摘要 + `structuredContent`
    - `structuredContent` 包含：
      - `found`
      - `valid`
      - `issues`（含位置、片段、块索引）

## 使用说明（本地调试）

1. 先安装 `mmdc`：
```bash
npm install -g @mermaid-js/mermaid-cli
```
2. 在项目目录启动服务：
```bash
cargo run --quiet
```
3. 用 MCP 客户端调用工具 `validateMermaid`，示例参数：
```json
{
  "diagram": "graph TD\nA[Start] --> B[End]",
  "format": "png"
}
```
4. 校验 Markdown 预览兼容性（GitHub）：
```json
{
  "markdown": "## Diagram\n\n```mermaid\ngraph TD\nA[main.rs] --> B[server]\n```\n"
}
```
5. 大文件推荐两阶段调用（避免上下文爆炸）：
```json
{
  "filePath": "/Users/pittcat/Dev/Rust/mermaid_validator_rs/interact.md"
}
```
先调用 `scanMermaidBlocks` 获取 `blockIndex`，再调用：
```json
{
  "filePath": "/Users/pittcat/Dev/Rust/mermaid_validator_rs/interact.md",
  "blockIndex": 2
}
```

## Claude JSON 配置

Claude Desktop 配置文件（macOS）通常在：

- `~/Library/Application Support/Claude/claude_desktop_config.json`

### 配置方式 A：直接用 `cargo run`（开发调试推荐）

```json
{
  "mcpServers": {
    "mermaid-validator-rs": {
      "command": "cargo",
      "args": [
        "run",
        "--quiet",
        "--manifest-path",
        "/Users/pittcat/Dev/Rust/mermaid_validator_rs/Cargo.toml"
      ],
      "env": {
        "MERMAID_TIMEOUT": "30s",
        "MERMAID_CLI": "mmdc"
      }
    }
  }
}
```

### 配置方式 B：用已安装二进制（稳定运行推荐）

先安装：
```bash
cargo install --path .
```

然后配置：
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

### Claude 侧验证

1. 重启 Claude Desktop
2. 确认出现 MCP 工具 `validateMermaid` 和 `validateMermaidPreview`
3. 用以下错误输入测试定位信息是否返回：
```json
{
  "diagram": "graph m\nA[main.rs] --> B[server",
  "format": "svg"
}
```
4. 期望输出包含：
- `Mermaid diagram is invalid`
- `Error location: line X, column Y`
- `Error snippet: ...`
- `Error reason: ...`

## 语法校验说明

`validateMermaid`：校验单段 Mermaid 文本。

`validateMermaidPreview`：按 GitHub 预览语义校验 Markdown。
- 检查 Mermaid fence 是否闭合
- 提取并校验多个 Mermaid 代码块
- 仅 `error` 计入失败（`valid = false`）

`scanMermaidBlocks` + `validateMermaidBlock`：按文件路径进行分阶段校验，适合大文件与长上下文场景。

## 可选配置

如需自定义 mmdc 路径，可设置环境变量：

- `MERMAID_CLI`（默认 `mmdc`）

## 超时控制

通过环境变量控制渲染超时：

- `MERMAID_TIMEOUT`
  - 支持秒或毫秒格式，例如 `10` / `10s` / `250ms`
  - 默认 30 秒

## 测试

```bash
cargo test
```
