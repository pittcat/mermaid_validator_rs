use std::time::Duration;

use mermaid_validator::cli_runner::{render_diagram, OutputFormat, RenderError};
use mermaid_validator::response_builder::invalid_result;

const SIMPLE_DIAGRAM: &str = "graph TB\nA-->B";
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
async fn render_svg() {
    if !mmdc_available() {
        eprintln!("mmdc not available; skipping render_svg");
        return;
    }
    let output = render_diagram(SIMPLE_DIAGRAM, OutputFormat::Svg, Duration::from_secs(10))
        .await
        .expect("SVG render failed");
    let svg = String::from_utf8(output).expect("SVG output was not valid UTF-8");
    assert!(svg.contains("<svg"));
}

#[tokio::test]
async fn render_png() {
    if !mmdc_available() {
        eprintln!("mmdc not available; skipping render_png");
        return;
    }
    let output = render_diagram(SIMPLE_DIAGRAM, OutputFormat::Png, Duration::from_secs(10))
        .await
        .expect("PNG render failed");
    let png_signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    assert!(output.starts_with(&png_signature));
}

#[test]
fn invalid_diagram() {
    let result = invalid_result("main error\n\nError details:\nextra info");
    assert_eq!(
        result.content[0].as_text().unwrap().text,
        "Mermaid diagram is invalid"
    );
    assert_eq!(result.content[1].as_text().unwrap().text, "main error");
    assert_eq!(
        result.content[2].as_text().unwrap().text,
        "Detailed error output:\nextra info"
    );
}

#[tokio::test]
async fn timeout_handling() {
    if !mmdc_available() {
        eprintln!("mmdc not available; skipping timeout_handling");
        return;
    }
    let result = render_diagram(SIMPLE_DIAGRAM, OutputFormat::Svg, Duration::from_millis(0)).await;
    assert!(matches!(result, Err(RenderError::Timeout { .. })));
}
