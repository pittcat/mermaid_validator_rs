use base64::Engine;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::to_value;

use crate::{
    cli_runner::{render_diagram, timeout_from_env, OutputFormat},
    preview_validator::{
        scan_markdown_for_mermaid, validate_markdown_for_github, validate_mermaid_block_in_markdown,
    },
    response_builder::{invalid_result, valid_result},
};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateParams {
    pub diagram: String,
    #[serde(default)]
    pub format: Option<OutputFormat>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidatePreviewParams {
    pub markdown: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScanMermaidBlocksParams {
    pub file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateMermaidBlockParams {
    pub file_path: String,
    pub block_index: u32,
}

#[derive(Clone)]
pub struct MermaidServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MermaidServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "validateMermaid",
        description = "Validates a Mermaid diagram and returns the rendered image (PNG or SVG) if valid"
    )]
    async fn validate_mermaid(
        &self,
        params: Parameters<ValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let format = params.format.unwrap_or_default();
        let timeout = timeout_from_env();
        let diagram = match normalize_diagram(&params.diagram) {
            Ok(diagram) => diagram,
            Err(message) => return Ok(invalid_result(&message)),
        };

        match render_diagram(&diagram, format, timeout).await {
            Ok(output) => {
                let encoded = base64::engine::general_purpose::STANDARD.encode(output);
                Ok(valid_result(format, encoded))
            }
            Err(err) => {
                let message = err.to_error_message();
                Ok(invalid_result(&message))
            }
        }
    }

    #[tool(
        name = "validateMermaidPreview",
        description = "Validates Mermaid preview compatibility for GitHub markdown, including fence issues and parse errors"
    )]
    async fn validate_mermaid_preview(
        &self,
        params: Parameters<ValidatePreviewParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let timeout = timeout_from_env();
        let result = validate_markdown_for_github(&params.markdown, timeout).await;
        let status_text = if result.valid {
            format!(
                "Mermaid preview is valid for GitHub ({} block(s) checked)",
                result.mermaid_block_count
            )
        } else {
            format!(
                "Mermaid preview is invalid for GitHub ({} error(s), {} block(s) checked)",
                result.error_count, result.mermaid_block_count
            )
        };

        let mut content = vec![Content::text(status_text)];
        for issue in &result.issues {
            let mut line = format!("[{}] {}: {}", issue.severity, issue.code, issue.message);
            if let Some(block_index) = issue.block_index {
                line.push_str(&format!(" (block #{block_index})"));
            }
            if let Some(line_no) = issue.line {
                if let Some(column_no) = issue.column {
                    line.push_str(&format!(" at line {line_no}, column {column_no}"));
                } else {
                    line.push_str(&format!(" at line {line_no}"));
                }
            }
            content.push(Content::text(line));
            if let Some(snippet) = &issue.snippet {
                content.push(Content::text(format!("Snippet: {snippet}")));
            }
        }

        Ok(CallToolResult {
            content,
            structured_content: Some(
                to_value(result).map_err(|err| McpError::internal_error(err.to_string(), None))?,
            ),
            is_error: Some(false),
            meta: None,
        })
    }

    #[tool(
        name = "scanMermaidBlocks",
        description = "Scans a markdown file and returns Mermaid block indexes and locations for GitHub preview validation"
    )]
    async fn scan_mermaid_blocks(
        &self,
        params: Parameters<ScanMermaidBlocksParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let markdown = match tokio::fs::read_to_string(&params.file_path).await {
            Ok(content) => content,
            Err(err) => {
                return Ok(invalid_result(&format!(
                    "Failed to read markdown file {}: {}",
                    params.file_path, err
                )))
            }
        };

        let result = scan_markdown_for_mermaid(&markdown);
        let summary = format!(
            "GitHub scan complete: {} block(s), {} error(s)",
            result.mermaid_block_count, result.error_count
        );

        let mut content = vec![Content::text(summary)];
        for block in &result.blocks {
            content.push(Content::text(format!(
                "Block #{} lines {}-{} ({} lines): {}",
                block.index, block.start_line, block.end_line, block.line_count, block.first_line
            )));
        }
        for issue in &result.issues {
            let mut line = format!("[{}] {}: {}", issue.severity, issue.code, issue.message);
            if let Some(line_no) = issue.line {
                line.push_str(&format!(" at line {line_no}"));
            }
            content.push(Content::text(line));
        }

        Ok(CallToolResult {
            content,
            structured_content: Some(
                to_value(result).map_err(|err| McpError::internal_error(err.to_string(), None))?,
            ),
            is_error: Some(false),
            meta: None,
        })
    }

    #[tool(
        name = "validateMermaidBlock",
        description = "Validates one Mermaid block in a markdown file by block index using GitHub preview rules"
    )]
    async fn validate_mermaid_block(
        &self,
        params: Parameters<ValidateMermaidBlockParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;
        let markdown = match tokio::fs::read_to_string(&params.file_path).await {
            Ok(content) => content,
            Err(err) => {
                return Ok(invalid_result(&format!(
                    "Failed to read markdown file {}: {}",
                    params.file_path, err
                )))
            }
        };
        let timeout = timeout_from_env();
        let result = validate_mermaid_block_in_markdown(&markdown, params.block_index, timeout).await;

        let summary = if result.valid {
            format!(
                "Block #{} is valid for GitHub preview",
                params.block_index
            )
        } else {
            format!(
                "Block #{} is invalid for GitHub preview ({} issue(s))",
                params.block_index,
                result.issues.len()
            )
        };

        let mut content = vec![Content::text(summary)];
        for issue in &result.issues {
            let mut line = format!("[{}] {}: {}", issue.severity, issue.code, issue.message);
            if let Some(line_no) = issue.line {
                if let Some(column_no) = issue.column {
                    line.push_str(&format!(" at line {line_no}, column {column_no}"));
                } else {
                    line.push_str(&format!(" at line {line_no}"));
                }
            }
            content.push(Content::text(line));
            if let Some(snippet) = &issue.snippet {
                content.push(Content::text(format!("Snippet: {snippet}")));
            }
        }

        Ok(CallToolResult {
            content,
            structured_content: Some(
                to_value(result).map_err(|err| McpError::internal_error(err.to_string(), None))?,
            ),
            is_error: Some(false),
            meta: None,
        })
    }
}

fn normalize_diagram(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if let Some(result) = strip_standalone_fenced_mermaid(trimmed) {
        return result;
    }
    if let Some(result) = extract_mermaid_block_from_markdown(trimmed) {
        return result;
    }
    Ok(trimmed.to_string())
}

fn strip_standalone_fenced_mermaid(input: &str) -> Option<Result<String, String>> {
    if !is_fence_start(input.lines().next()?.trim()) {
        return None;
    }

    let mut lines = input.lines();
    let first = lines.next()?.trim();
    let mut body: Vec<&str> = lines.collect();
    if body.is_empty() {
        return Some(Err("Invalid Mermaid code block: empty fenced content".to_string()));
    }
    if body.last()?.trim() != "```" {
        return Some(Err(
            "Invalid Mermaid code block: missing closing fence ```".to_string(),
        ));
    }
    if !is_mermaid_fence(first) && first != "```" {
        return Some(Err(
            "Invalid Mermaid code block: opening fence must be ```mermaid or ```".to_string(),
        ));
    }
    body.pop();
    Some(Ok(body.join("\n")))
}

fn extract_mermaid_block_from_markdown(input: &str) -> Option<Result<String, String>> {
    let lines: Vec<&str> = input.lines().collect();
    let mut start_idx = None;

    for (idx, line) in lines.iter().enumerate() {
        if is_mermaid_fence(line.trim()) {
            start_idx = Some(idx);
            break;
        }
    }

    let start = start_idx?;
    let mut end = None;
    for (idx, line) in lines.iter().enumerate().skip(start + 1) {
        if line.trim() == "```" {
            end = Some(idx);
            break;
        }
    }

    let end = match end {
        Some(end) => end,
        None => {
            return Some(Err(
                "Invalid Mermaid code block: missing closing fence ```".to_string(),
            ))
        }
    };

    let body = lines[start + 1..end].join("\n");
    Some(Ok(body))
}

fn is_mermaid_fence(line: &str) -> bool {
    line.eq_ignore_ascii_case("```mermaid")
}

fn is_fence_start(line: &str) -> bool {
    line.starts_with("```")
}

#[tool_handler]
impl ServerHandler for MermaidServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.server_info = Implementation {
            name: "Mermaid Validator".to_string(),
            version: "0.6.0".to_string(),
            title: None,
            icons: None,
            website_url: None,
        };
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_diagram;

    #[test]
    fn normalize_plain_diagram() {
        let input = "graph TD\nA-->B";
        let normalized = normalize_diagram(input).unwrap();
        assert_eq!(normalized, input);
    }

    #[test]
    fn normalize_standalone_fenced_mermaid() {
        let input = "```mermaid\ngraph TD\nA-->B\n```";
        let normalized = normalize_diagram(input).unwrap();
        assert_eq!(normalized, "graph TD\nA-->B");
    }

    #[test]
    fn normalize_embedded_markdown_mermaid() {
        let input = "# title\n\n```mermaid\ngraph TD\nA-->B\n```\n\ntext";
        let normalized = normalize_diagram(input).unwrap();
        assert_eq!(normalized, "graph TD\nA-->B");
    }

    #[test]
    fn normalize_reports_missing_closing_fence() {
        let input = "```mermaid\ngraph TD\nA-->B\n``";
        let error = normalize_diagram(input).unwrap_err();
        assert!(error.contains("missing closing fence"));
    }
}
