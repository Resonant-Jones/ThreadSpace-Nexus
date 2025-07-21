use crate::{BridgeResult, CapabilityResult, SubprocessConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Request structure for ritual_engine tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualEngineRequest {
    pub ritual_type: String,
    pub parameters: HashMap<String, String>,
    pub context: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Response structure for ritual_engine tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualEngineResponse {
    pub ritual_id: String,
    pub status: String,
    pub result: HashMap<String, String>,
    pub logs: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Run the ritual_engine Python CLI tool
pub async fn run_ritual_engine(
    request: RitualEngineRequest,
    config: Option<SubprocessConfig>,
) -> BridgeResult<CapabilityResult<RitualEngineResponse>> {
    let start_time = std::time::Instant::now();
    let config = config.unwrap_or_else(|| SubprocessConfig {
        timeout: Duration::from_secs(120),
        ..Default::default()
    });

    // Determine the correct path to ritual_engine.py
    let python_path = "guardian-backend_v2/ritual_engine/main.py";
    
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
            "ritual_engine",
            duration,
            "1.0.0",
        )),
        Err(e) => Ok(CapabilityResult::error(
            e.to_string(),
            "ritual_engine",
            duration,
            "1.0.0",
        )),
    }
}

/// Synchronous wrapper for ritual_engine
pub fn run_ritual_engine_sync(
    request: RitualEngineRequest,
    config: Option<SubprocessConfig>,
) -> BridgeResult<CapabilityResult<RitualEngineResponse>> {
    let rt = tokio::runtime::Runtime::new().map_err(crate::BridgeError::Io)?;
    rt.block_on(run_ritual_engine(request, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ritual_engine_request_serialization() {
        let request = RitualEngineRequest {
            ritual_type: "memory_sync".to_string(),
            parameters: {
                let mut map = HashMap::new();
                map.insert("target".to_string(), "memory_bank".to_string());
                map
            },
            context: Some("test_context".to_string()),
            metadata: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("memory_sync"));
        assert!(json.contains("test_context"));
    }

    #[test]
    fn test_ritual_engine_response_deserialization() {
        let json = r#"{
            "ritual_id": "ritual-123",
            "status": "completed",
            "result": {"key": "value"},
            "logs": ["log1", "log2"],
            "metadata": {"type": "test"},
            "completed_at": "2024-01-01T00:00:00Z"
        }"#;

        let response: RitualEngineResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.ritual_id, "ritual-123");
        assert_eq!(response.status, "completed");
    }
}
