use crate::{BridgeError, BridgeResult, SubprocessConfig};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command as TokioCommand;
use tracing::{debug, error, info};

/// Spawn a subprocess and handle JSON I/O
pub async fn spawn_subprocess<I, O>(
    command: &str,
    args: &[&str],
    input: &I,
    config: &SubprocessConfig,
) -> BridgeResult<O>
where
    I: Serialize,
    O: for<'de> Deserialize<'de>,
{
    let start_time = Instant::now();
    
    // Serialize input to JSON
    let input_json = serde_json::to_string(input)?;
    if config.log_io {
        debug!("Input JSON: {}", input_json);
    }

    // Build command
    let mut cmd = TokioCommand::new(command);
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Set working directory if specified
    if let Some(ref dir) = config.working_dir {
        cmd.current_dir(dir);
    }

    // Set environment variables
    for (key, value) in &config.env_vars {
        cmd.env(key, value);
    }

    info!("Spawning subprocess: {} {}", command, args.join(" "));

    // Spawn the process
    let mut child = cmd.spawn().map_err(BridgeError::Io)?;

    // Write input to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = tokio::io::BufWriter::new(stdin);
        stdin.write_all(input_json.as_bytes()).await?;
        stdin.flush().await?;
    }

    // Wait for process with timeout
    let timeout_duration = config.timeout;
    let output = match tokio::time::timeout(timeout_duration, child.wait_with_output()).await {
        Ok(result) => result.map_err(BridgeError::Io)?,
        Err(_) => {
            child.kill().await.ok();
            return Err(BridgeError::Timeout { duration: timeout_duration });
        }
    };

    let duration = start_time.elapsed();

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Process failed with stderr: {}", stderr);
        return Err(BridgeError::ProcessFailed(
            output.status.code().unwrap_or(-1),
        ));
    }

    // Parse output JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    if config.log_io {
        debug!("Output JSON: {}", stdout);
    }

    let parsed: O = serde_json::from_str(&stdout)?;
    Ok(parsed)
}

/// Synchronous version for blocking contexts
pub fn spawn_subprocess_sync<I, O>(
    command: &str,
    args: &[&str],
    input: &I,
    config: &SubprocessConfig,
) -> BridgeResult<O>
where
    I: Serialize,
    O: for<'de> Deserialize<'de>,
{
    let rt = tokio::runtime::Runtime::new().map_err(BridgeError::Io)?;
    rt.block_on(spawn_subprocess(command, args, input, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestInput {
        message: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestOutput {
        response: String,
    }

    #[tokio::test]
    async fn test_echo_subprocess() {
        let config = SubprocessConfig::default();
        let input = TestInput {
            message: "hello".to_string(),
        };
        
        // Use Python as a simple echo service
        let result = spawn_subprocess::<TestInput, TestOutput>(
            "python3",
            &["-c", "import json, sys; data=json.load(sys.stdin); print(json.dumps({'response': data['message']}))"],
            &input,
            &config,
        ).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.response, "hello");
    }
}
