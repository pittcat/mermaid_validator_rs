use mermaid_validator::server::MermaidServer;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = MermaidServer::new()
        .serve(stdio())
        .await
        .inspect_err(|err| eprintln!("Error starting server: {err}"))?;

    service.waiting().await?;
    Ok(())
}
