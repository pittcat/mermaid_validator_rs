use std::time::Duration;

use mermaid_validator::preview_validator::validate_markdown_for_github;

fn mmdc_available() -> bool {
    std::process::Command::new("mmdc")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn preview_reports_unclosed_mermaid_fence() {
    let markdown = "# Title\n\n```mermaid\ngraph TD\nA-->B\n``\n";
    let result = validate_markdown_for_github(markdown, Duration::from_secs(5)).await;
    assert!(!result.valid);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.issues[0].code, "mermaid_unclosed_fence");
}

#[tokio::test]
async fn preview_reports_parse_error_location() {
    if !mmdc_available() {
        eprintln!("mmdc not available; skipping preview_reports_parse_error_location");
        return;
    }

    let markdown = "## Diagram\n\n```mermaid\ngraph TD\nA --> B[bad\n```\n";
    let result = validate_markdown_for_github(markdown, Duration::from_secs(10)).await;
    assert!(!result.valid);
    let parse_issue = result
        .issues
        .iter()
        .find(|issue| issue.code == "mermaid_parse_error")
        .expect("expected mermaid_parse_error issue");
    assert!(parse_issue.line.is_some());
    assert!(parse_issue.block_index.is_some());
}
