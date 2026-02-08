use mermaid_validator::cli_runner::{render_diagram, OutputFormat};
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("🔥 Running flamegraph profiling workload...");
    println!();

    let timeout = Duration::from_secs(5);

    // Simple diagram
    println!("1. Rendering simple diagram...");
    let simple = "graph TD\nA-->B";
    match render_diagram(simple, OutputFormat::Svg, timeout).await {
        Ok(_) => println!("   ✓ Simple diagram rendered successfully"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Complex diagram
    println!("2. Rendering complex diagram...");
    let complex = r#"graph TB
    A[Start] --> B{Decision}
    B -->|Yes| C[Process 1]
    B -->|No| D[Process 2]
    C --> E[End]
    D --> E
    style A fill:#f9f,stroke:#333,stroke-width:4px"#;
    match render_diagram(complex, OutputFormat::Svg, timeout).await {
        Ok(_) => println!("   ✓ Complex diagram rendered successfully"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Multiple diagrams (simulating markdown validation)
    println!("3. Rendering multiple diagrams...");
    for i in 0..5 {
        let diagram = format!("graph TD\nA{}-->B{}", i, i + 1);
        match render_diagram(&diagram, OutputFormat::Svg, timeout).await {
            Ok(_) => println!("   ✓ Diagram {} rendered", i),
            Err(e) => println!("   ✗ Error in diagram {}: {}", i, e),
        }
    }

    println!();
    println!("✅ Profiling workload complete!");
}
