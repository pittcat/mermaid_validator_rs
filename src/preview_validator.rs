use std::time::Duration;

use schemars::JsonSchema;
use serde::Serialize;

use crate::cli_runner::{render_diagram, OutputFormat};

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PreviewIssue {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MermaidBlockInfo {
    pub index: u32,
    pub start_line: u32,
    pub end_line: u32,
    pub line_count: u32,
    pub char_count: u32,
    pub first_line: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PreviewScanResult {
    pub target: String,
    pub error_count: u32,
    pub mermaid_block_count: u32,
    pub blocks: Vec<MermaidBlockInfo>,
    pub issues: Vec<PreviewIssue>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PreviewValidationResult {
    pub target: String,
    pub valid: bool,
    pub error_count: u32,
    pub mermaid_block_count: u32,
    pub issues: Vec<PreviewIssue>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockValidationResult {
    pub target: String,
    pub block_index: u32,
    pub found: bool,
    pub valid: bool,
    pub issues: Vec<PreviewIssue>,
}

#[derive(Debug, Clone)]
struct MermaidBlock {
    index: u32,
    start_line: u32,
    end_line: u32,
    content: String,
}

#[derive(Debug, Clone)]
struct FenceState<'a> {
    marker: char,
    len: usize,
    is_mermaid: bool,
    start_line: u32,
    content_lines: Vec<&'a str>,
}

pub fn scan_markdown_for_mermaid(markdown: &str) -> PreviewScanResult {
    let (blocks, mut issues) = collect_mermaid_blocks(markdown);

    let has_unclosed_mermaid = issues
        .iter()
        .any(|issue| issue.code == "mermaid_unclosed_fence");

    if blocks.is_empty() && !has_unclosed_mermaid {
        issues.push(PreviewIssue {
            severity: "error".to_string(),
            code: "no_mermaid_blocks".to_string(),
            message: "No mermaid code block found in markdown".to_string(),
            line: None,
            column: None,
            snippet: None,
            block_index: None,
        });
    }

    let block_infos = blocks
        .iter()
        .map(|block| MermaidBlockInfo {
            index: block.index,
            start_line: block.start_line,
            end_line: block.end_line,
            line_count: block.content.lines().count() as u32,
            char_count: block.content.chars().count() as u32,
            first_line: block.content.lines().next().unwrap_or("").to_string(),
        })
        .collect::<Vec<_>>();

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count() as u32;

    PreviewScanResult {
        target: "github".to_string(),
        error_count,
        mermaid_block_count: blocks.len() as u32,
        blocks: block_infos,
        issues,
    }
}

pub async fn validate_markdown_for_github(
    markdown: &str,
    timeout: Duration,
) -> PreviewValidationResult {
    let (blocks, mut issues) = collect_mermaid_blocks(markdown);

    let has_unclosed_mermaid = issues
        .iter()
        .any(|issue| issue.code == "mermaid_unclosed_fence");

    if blocks.is_empty() && !has_unclosed_mermaid {
        issues.push(PreviewIssue {
            severity: "error".to_string(),
            code: "no_mermaid_blocks".to_string(),
            message: "No mermaid code block found in markdown".to_string(),
            line: None,
            column: None,
            snippet: None,
            block_index: None,
        });
    }

    for block in &blocks {
        match render_diagram(&block.content, OutputFormat::Svg, timeout).await {
            Ok(_) => {}
            Err(err) => {
                let error_message = err.to_error_message();
                issues.push(build_mermaid_parse_issue(block, &error_message));
            }
        }
    }

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count() as u32;

    PreviewValidationResult {
        target: "github".to_string(),
        valid: error_count == 0,
        error_count,
        mermaid_block_count: blocks.len() as u32,
        issues,
    }
}

pub async fn validate_mermaid_block_in_markdown(
    markdown: &str,
    block_index: u32,
    timeout: Duration,
) -> BlockValidationResult {
    let (blocks, issues) = collect_mermaid_blocks(markdown);

    let block = blocks.iter().find(|block| block.index == block_index);
    if block.is_none() {
        return BlockValidationResult {
            target: "github".to_string(),
            block_index,
            found: false,
            valid: false,
            issues: vec![PreviewIssue {
                severity: "error".to_string(),
                code: "block_not_found".to_string(),
                message: format!("Mermaid block index {block_index} was not found"),
                line: None,
                column: None,
                snippet: None,
                block_index: Some(block_index),
            }],
        };
    }

    let block = block.unwrap();
    let mut block_issues = issues
        .into_iter()
        .filter(|issue| issue.block_index.is_none())
        .collect::<Vec<_>>();

    match render_diagram(&block.content, OutputFormat::Svg, timeout).await {
        Ok(_) => {}
        Err(err) => block_issues.push(build_mermaid_parse_issue(block, &err.to_error_message())),
    }

    let valid = !block_issues
        .iter()
        .any(|issue| issue.severity.as_str() == "error");

    BlockValidationResult {
        target: "github".to_string(),
        block_index,
        found: true,
        valid,
        issues: block_issues,
    }
}

fn collect_mermaid_blocks(markdown: &str) -> (Vec<MermaidBlock>, Vec<PreviewIssue>) {
    let mut blocks = Vec::new();
    let mut issues = Vec::new();
    let mut fence_state: Option<FenceState<'_>> = None;
    let mut mermaid_index = 0u32;

    for (idx, raw_line) in markdown.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let line = raw_line.trim_start();

        if let Some(state) = fence_state.as_mut() {
            if is_fence_close(line, state.marker, state.len) {
                let closed = fence_state.take().expect("fence state exists");
                if closed.is_mermaid {
                    mermaid_index += 1;
                    blocks.push(MermaidBlock {
                        index: mermaid_index,
                        start_line: closed.start_line,
                        end_line: line_no,
                        content: closed.content_lines.join("\n"),
                    });
                }
            } else {
                state.content_lines.push(raw_line);
            }
            continue;
        }

        if let Some((marker, len, is_mermaid)) = parse_fence_start(line) {
            fence_state = Some(FenceState {
                marker,
                len,
                is_mermaid,
                start_line: line_no,
                content_lines: Vec::with_capacity(8),
            });
        }
    }

    if let Some(unclosed) = fence_state {
        let (code, message) = if unclosed.is_mermaid {
            (
                "mermaid_unclosed_fence",
                "Mermaid code block is missing closing fence ```",
            )
        } else {
            (
                "markdown_unclosed_fence",
                "Markdown code fence is missing closing marker",
            )
        };
        issues.push(PreviewIssue {
            severity: "error".to_string(),
            code: code.to_string(),
            message: message.to_string(),
            line: Some(unclosed.start_line),
            column: None,
            snippet: None,
            block_index: None,
        });
    }

    (blocks, issues)
}

fn parse_fence_start(line: &str) -> Option<(char, usize, bool)> {
    let mut chars = line.chars();
    let marker = chars.next()?;
    if marker != '`' && marker != '~' {
        return None;
    }

    let mut len = 1usize;
    for ch in chars.by_ref() {
        if ch == marker {
            len += 1;
        } else {
            break;
        }
    }
    if len < 3 {
        return None;
    }

    let rest = line[len..].trim();
    Some((marker, len, is_mermaid_lang(rest)))
}

fn is_fence_close(line: &str, marker: char, min_len: usize) -> bool {
    let marker = marker as u8;
    let bytes = line.as_bytes();
    let mut len = 0usize;
    while len < bytes.len() && bytes[len] == marker {
        len += 1;
    }
    if len < min_len {
        return false;
    }
    line[len..].trim().is_empty()
}

fn is_mermaid_lang(lang: &str) -> bool {
    let first = lang.split_whitespace().next().unwrap_or("");
    first.eq_ignore_ascii_case("mermaid")
}

fn build_mermaid_parse_issue(block: &MermaidBlock, error_message: &str) -> PreviewIssue {
    let details = split_error_details(error_message)
        .1
        .unwrap_or_else(|| error_message.to_string());
    let context = parse_error_context(&details);
    let global_line = context.line.map(|line| block.start_line + line);

    PreviewIssue {
        severity: "error".to_string(),
        code: "mermaid_parse_error".to_string(),
        message: context
            .reason
            .clone()
            .unwrap_or_else(|| "Mermaid parse error".to_string()),
        line: global_line,
        column: context.column,
        snippet: context.snippet.clone(),
        block_index: Some(block.index),
    }
}

fn split_error_details(message: &str) -> (String, Option<String>) {
    if let Some((main_error, details)) = message.split_once("\n\nError details:\n") {
        (main_error.to_string(), Some(details.to_string()))
    } else {
        (message.to_string(), None)
    }
}

#[derive(Debug, Default)]
struct ErrorContext {
    line: Option<u32>,
    column: Option<u32>,
    snippet: Option<String>,
    reason: Option<String>,
}

fn parse_error_context(message: &str) -> ErrorContext {
    let mut context = ErrorContext::default();
    let mut take_snippet_line = false;
    let mut take_caret_line = false;

    for line in message.lines() {
        if context.reason.is_none() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && (trimmed.contains("Expecting")
                    || trimmed.contains("UnknownDiagramError")
                    || trimmed.contains("got "))
            {
                context.reason = Some(trimmed.to_string());
            }
        }

        if take_snippet_line {
            context.snippet = Some(line.trim_end().to_string());
            take_snippet_line = false;
            take_caret_line = true;
            continue;
        }

        if take_caret_line {
            if let Some(pos) = line.find('^') {
                context.column = Some((pos + 1) as u32);
            }
            take_caret_line = false;
            continue;
        }

        if let Some(line_number) = extract_line_number(line) {
            context.line = Some(line_number);
            take_snippet_line = true;
        }
    }

    context
}

fn extract_line_number(line: &str) -> Option<u32> {
    const MARKER: &[u8] = b"parse error on line ";
    let bytes = line.as_bytes();

    if bytes.len() < MARKER.len() {
        return None;
    }

    let marker_start = bytes
        .windows(MARKER.len())
        .position(|window| window.eq_ignore_ascii_case(MARKER))?;

    let mut index = marker_start + MARKER.len();
    let mut value: u32 = 0;
    let mut seen_digit = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_digit() {
            seen_digit = true;
            value = value.checked_mul(10)?.checked_add((byte - b'0') as u32)?;
            index += 1;
        } else {
            break;
        }
    }

    seen_digit.then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mermaid_blocks_and_detect_unclosed_fence() {
        let markdown = "text\n```mermaid\ngraph TD\nA-->B\n``\n";
        let (blocks, issues) = collect_mermaid_blocks(markdown);
        assert!(blocks.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "mermaid_unclosed_fence");
        assert_eq!(issues[0].line, Some(2));
    }

    #[test]
    fn parse_multiple_mermaid_blocks() {
        let markdown = "```mermaid\ngraph TD\nA-->B\n```\n\n```mermaid\nflowchart LR\nX-->Y\n```";
        let (blocks, issues) = collect_mermaid_blocks(markdown);
        assert!(issues.is_empty());
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[0].end_line, 4);
        assert_eq!(blocks[1].start_line, 6);
        assert_eq!(blocks[1].end_line, 9);
    }

    #[test]
    fn map_local_error_line_to_markdown_line() {
        let block = MermaidBlock {
            index: 2,
            start_line: 20,
            end_line: 24,
            content: "graph TD\nA-->B".to_string(),
        };
        let issue = build_mermaid_parse_issue(
            &block,
            "mermaid-cli process exited with code 1\n\nError details:\nmermaid error: Parse error on line 2:\nA-->B x\n----^\nExpecting 'SEMI', got 'NODE_STRING'",
        );
        assert_eq!(issue.line, Some(22));
        assert_eq!(issue.block_index, Some(2));
    }

    #[test]
    fn scan_reports_block_info() {
        let markdown = "```mermaid\ngraph TD\nA-->B\n```";
        let scan = scan_markdown_for_mermaid(markdown);
        assert_eq!(scan.mermaid_block_count, 1);
        assert_eq!(scan.blocks[0].line_count, 2);
        assert_eq!(scan.blocks[0].first_line, "graph TD");
    }
}
