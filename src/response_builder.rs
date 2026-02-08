use rmcp::model::{CallToolResult, Content};

use crate::cli_runner::OutputFormat;

pub fn valid_result(format: OutputFormat, base64_data: String) -> CallToolResult {
    CallToolResult {
        content: vec![
            Content::text("Mermaid diagram is valid"),
            Content::image(base64_data, format.mime_type()),
        ],
        structured_content: None,
        is_error: None,
        meta: None,
    }
}

pub fn invalid_result(error_message: &str) -> CallToolResult {
    let (main_error, details) = split_error_details(error_message);
    let context = parse_error_context(details.as_deref().unwrap_or(error_message));
    let mut content = vec![
        Content::text("Mermaid diagram is invalid"),
        Content::text(main_error),
    ];

    if let Some(location) = context.location_text() {
        content.push(Content::text(location));
    }
    if let Some(snippet) = context.snippet_text() {
        content.push(Content::text(snippet));
    }
    if let Some(reason) = context.reason_text() {
        content.push(Content::text(reason));
    }

    if let Some(details) = details {
        content.push(Content::text(format!("Detailed error output:\n{details}")));
    }

    CallToolResult {
        content,
        structured_content: None,
        is_error: None,
        meta: None,
    }
}

pub fn processing_error(error_message: &str) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(format!(
            "Error processing Mermaid diagram: {error_message}"
        ))],
        structured_content: None,
        is_error: None,
        meta: None,
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

impl ErrorContext {
    fn location_text(&self) -> Option<String> {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                Some(format!("Error location: line {line}, column {column}"))
            }
            (Some(line), None) => Some(format!("Error location: line {line}")),
            _ => None,
        }
    }

    fn snippet_text(&self) -> Option<String> {
        self.snippet
            .as_ref()
            .map(|snippet| format!("Error snippet: {snippet}"))
    }

    fn reason_text(&self) -> Option<String> {
        self.reason
            .as_ref()
            .map(|reason| format!("Error reason: {reason}"))
    }
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
    fn split_details_none() {
        let (main, details) = split_error_details("simple error");
        assert_eq!(main, "simple error");
        assert!(details.is_none());
    }

    #[test]
    fn split_details_with_extra() {
        let (main, details) = split_error_details("main error\n\nError details:\nline1\nline2");
        assert_eq!(main, "main error");
        assert_eq!(details.unwrap(), "line1\nline2");
    }

    #[test]
    fn parse_context_from_parse_error() {
        let message = "mermaid error: Parse error on line 2:\n\
graph m  A[main.rs] --> B[ser\n\
----------^\n\
Expecting 'SEMI', 'NEWLINE', 'EOF', got 'NODE_STRING'";
        let context = parse_error_context(message);
        assert_eq!(context.line, Some(2));
        assert_eq!(context.column, Some(11));
        assert!(context.snippet.unwrap().starts_with("graph m"));
        assert!(context.reason.unwrap().contains("Expecting 'SEMI'"));
    }

    #[test]
    fn parse_context_unknown_diagram() {
        let message = "UnknownDiagramError: No diagram type detected matching given configuration";
        let context = parse_error_context(message);
        assert!(context.line.is_none());
        assert!(context.column.is_none());
        assert!(context.reason.unwrap().contains("UnknownDiagramError"));
    }
}
