use std::{env, process::Stdio, time::Duration};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    time,
};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Svg,
    Png,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Svg => "svg",
            OutputFormat::Png => "png",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            OutputFormat::Svg => "image/svg+xml",
            OutputFormat::Png => "image/png",
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Png
    }
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("mermaid-cli process exited with code {code}")]
    ProcessExit { code: i32, stderr: String },
    #[error("{message}")]
    Spawn { message: String },
    #[error("{message}")]
    Io { message: String },
    #[error("mermaid-cli process timed out after {seconds}s")]
    Timeout { seconds: u64 },
}

impl RenderError {
    pub fn to_error_message(&self) -> String {
        match self {
            RenderError::ProcessExit { code, stderr } => {
                let mut message = format!("mermaid-cli process exited with code {code}");
                if !stderr.trim().is_empty() {
                    message.push_str("\n\nError details:\n");
                    message.push_str(stderr.trim_end());
                }
                message
            }
            RenderError::Spawn { message } => message.clone(),
            RenderError::Io { message } => message.clone(),
            RenderError::Timeout { seconds } => {
                format!("mermaid-cli process timed out after {seconds}s")
            }
        }
    }

    fn spawn(err: std::io::Error) -> Self {
        RenderError::Spawn {
            message: err.to_string(),
        }
    }

    fn io(context: &str, err: std::io::Error) -> Self {
        RenderError::Io {
            message: format!("{context}: {err}"),
        }
    }

    fn process_exit(code: i32, stderr: String) -> Self {
        RenderError::ProcessExit { code, stderr }
    }

    fn timeout(duration: Duration) -> Self {
        RenderError::Timeout {
            seconds: duration.as_secs(),
        }
    }
}

pub fn timeout_from_env() -> Duration {
    env::var("MERMAID_TIMEOUT")
        .ok()
        .and_then(|value| parse_timeout(value.trim()))
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_TIMEOUT_SECS))
}

fn parse_timeout(value: &str) -> Option<Duration> {
    if value.is_empty() {
        return None;
    }
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.trim().parse::<u64>().ok().map(Duration::from_millis);
    }
    if let Some(sec) = value.strip_suffix('s') {
        return sec.trim().parse::<u64>().ok().map(Duration::from_secs);
    }
    value.trim().parse::<u64>().ok().map(Duration::from_secs)
}

pub async fn render_diagram(
    diagram: &str,
    format: OutputFormat,
    timeout: Duration,
) -> Result<Vec<u8>, RenderError> {
    let mut command = Command::new(mermaid_cli_command());
    command
        .arg("-i")
        .arg("/dev/stdin")
        .arg("-o")
        .arg("-")
        .arg("-e")
        .arg(format.as_str());

    if format == OutputFormat::Png {
        command.arg("-b").arg("transparent");
    }

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(RenderError::spawn)?;
    let mut stdin = child.stdin.take().ok_or_else(|| RenderError::Io {
        message: "Failed to open mermaid-cli stdin".to_string(),
    })?;
    let mut stdout = child.stdout.take().ok_or_else(|| RenderError::Io {
        message: "Failed to capture mermaid-cli stdout".to_string(),
    })?;
    let mut stderr = child.stderr.take().ok_or_else(|| RenderError::Io {
        message: "Failed to capture mermaid-cli stderr".to_string(),
    })?;

    let input = diagram.as_bytes().to_vec();
    let stdin_handle = tokio::spawn(async move {
        stdin.write_all(&input).await?;
        stdin.shutdown().await?;
        Ok::<(), std::io::Error>(())
    });

    let stdout_handle = tokio::spawn(async move {
        let mut buffer = Vec::new();
        stdout.read_to_end(&mut buffer).await?;
        Ok::<Vec<u8>, std::io::Error>(buffer)
    });

    let stderr_handle = tokio::spawn(async move {
        let mut buffer = Vec::new();
        stderr.read_to_end(&mut buffer).await?;
        Ok::<Vec<u8>, std::io::Error>(buffer)
    });

    let status_result: Result<std::process::ExitStatus, RenderError> = tokio::select! {
        biased;
        _ = time::sleep(timeout) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            Err(RenderError::timeout(timeout))
        }
        status = child.wait() => {
            status.map_err(|err| RenderError::io("Failed to wait for mermaid-cli", err))
        }
    };

    stdin_handle
        .await
        .map_err(|err| RenderError::Io {
            message: format!("Failed to join stdin writer: {err}"),
        })?
        .map_err(|err| RenderError::io("Failed to write mermaid-cli stdin", err))?;

    let stdout_bytes = stdout_handle
        .await
        .map_err(|err| RenderError::Io {
            message: format!("Failed to join stdout reader: {err}"),
        })?
        .map_err(|err| RenderError::io("Failed to read mermaid-cli stdout", err))?;

    let stderr_bytes = stderr_handle
        .await
        .map_err(|err| RenderError::Io {
            message: format!("Failed to join stderr reader: {err}"),
        })?
        .map_err(|err| RenderError::io("Failed to read mermaid-cli stderr", err))?;

    let stderr_text = String::from_utf8_lossy(&stderr_bytes).to_string();

    let status = match status_result {
        Ok(status) => status,
        Err(err) => return Err(err),
    };

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(RenderError::process_exit(code, stderr_text));
    }

    Ok(stdout_bytes)
}

fn mermaid_cli_command() -> String {
    env::var("MERMAID_CLI").unwrap_or_else(|_| "mmdc".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timeout_seconds() {
        assert_eq!(parse_timeout("5").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_timeout("8s").unwrap(), Duration::from_secs(8));
    }

    #[test]
    fn parse_timeout_millis() {
        assert_eq!(parse_timeout("250ms").unwrap(), Duration::from_millis(250));
    }
}
