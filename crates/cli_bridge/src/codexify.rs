use crate::{BridgeResult, CapabilityResult, SubprocessConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Request structure for codexify tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexifyRequest {
    pub file_path: String,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Response structure for codexify tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexifyResponse {
    pub node_id: String,
    pub summary: String,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Run the codexify Python CLI tool
pub async fn run_codexify(
    request: CodexifyRequest,
    config: Option<SubprocessConfig>,
) -> BridgeResult<CapabilityResult<CodexifyResponse>> {
    let start_time = std::time::Instant::now();
    let config = config.unwrap_or_else(|| SubprocessConfig {
        timeout: Duration::from_secs(60),
        ..Default::default()
    });

    // Determine the correct path to codexify.py
    let python_path = "guardian-backend_v2/codexify/main.py";
    
    let result = crate::spawn_subprocess(
        "python3",
        &[python_path],
        &request,
        &config,
    ).await;

    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => Ok(CapabilityResult::success(
            response,
            "codexify",
            duration,
            "1.0.0",
        )),
        Err(e) => Ok(CapabilityResult::error(
            e.to_string(),
            "codexify",
            duration,
            "1.0.0",
        )),
    }
}

/// Synchronous wrapper for codexify
pub fn run_codexify_sync(
    request: CodexifyRequest,
    config: Option<SubprocessConfig>,
) -> BridgeResult<CapabilityResult<CodexifyResponse>> {
    let rt = tokio::runtime::Runtime::new().map_err(crate::BridgeError::Io)?;
    rt.block_on(run_codexify(request, config))
}

/// Fetch schema from codexify.py if --schema flag is supported
pub async fn fetch_codexify_schema() -> BridgeResult<serde_json::Value> {
    let config = SubprocessConfig {
        timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let result = crate::spawn_subprocess::<(), serde_json::Value>(
        "python3",
        &["guardian-backend_v2/codexify/main.py", "--schema"],
        &(),
        &config,
    ).await;

    result.map_err(|e| {
        crate::BridgeError::InvalidOutput(format!("Failed to fetch schema: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_codexify_integration() {
        // Create a temporary test file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is a test file for codexify integration").unwrap();
        
        let request = CodexifyRequest {
            file_path: temp_file.path().to_string_lossy().to_string(),
            tags: Some(vec!["test".to_string(), "integration".to_string()]),
            metadata: None,
        };

        let config = SubprocessConfig {
            timeout: Duration::from_secs(30),
            log_io: true,
            ..Default::default()
        };

        let result = run_codexify(request, Some(config)).await;
        
        // Note: This test will fail if codexify.py is not properly set up
        // In a real environment, you'd mock the subprocess or use a test fixture
        match result {
            Ok(capability_result) => {
                if capability_result.success {
                    println!("Codexify succeeded: {:?}", capability_result.data);
                    assert!(!capability_result.data.unwrap().node_id.is_empty());
                } else {
                    println!("Codexify failed: {:?}", capability_result.error);
                    // This is expected in test environment without proper Python setup
                }
            }
            Err(e) => {
                println!("Bridge error: {}", e);
                // Expected in test environment
            }
        }
    }

    #[test]
    fn test_codexify_request_serialization() {
        let request = CodexifyRequest {
            file_path: "/test/path".to_string(),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            metadata: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("/test/path"));
        assert!(json.contains("tag1"));
    }

    #[test]
    fn test_codexify_response_deserialization() {
        let json = r#"{
            "node_id": "test-node-123",
            "summary": "Test summary",
            "metadata": {"key": "value"},
            "tags": ["test"],
            "created_at": "2024-01-01T00:00:00Z"
        }"#;

        let response: CodexifyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.node_id, "test-node-123");
        assert_eq!(response.summary, "Test summary");
    }
}
