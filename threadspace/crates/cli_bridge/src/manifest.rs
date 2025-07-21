use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manifest structure for CLI tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub language: String,
    pub entry_point: String,
    pub capabilities: Vec<String>,
    pub schema: Option<String>,
    pub timeout_sec: u64,
    pub requirements: HashMap<String, String>,
    pub inputs: HashMap<String, InputSchema>,
    pub outputs: HashMap<String, OutputSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSchema {
    pub r#type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    pub r#type: String,
    pub description: String,
}

/// Load manifest from JSON file
pub fn load_manifest<P: AsRef<Path>>(path: P) -> BridgeResult<Manifest> {
    let content = fs::read_to_string(path)?;
    let manifest: Manifest = serde_json::from_str(&content)?;
    Ok(manifest)
}

/// Save manifest to JSON file
pub fn save_manifest<P: AsRef<Path>>(manifest: &Manifest, path: P) -> BridgeResult<()> {
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(path, content)?;
    Ok(())
}

/// Generate default manifest for codexify
pub fn generate_codexify_manifest() -> Manifest {
    let mut requirements = HashMap::new();
    requirements.insert("python".to_string(), ">=3.10".to_string());
    requirements.insert("openai".to_string(), "*".to_string());
    requirements.insert("chromadb".to_string(), "*".to_string());
    requirements.insert("tiktoken".to_string(), "*".to_string());

    let mut inputs = HashMap::new();
    inputs.insert(
        "file_path".to_string(),
        InputSchema {
            r#type: "string".to_string(),
            description: "Path to the file or folder to codexify".to_string(),
            required: true,
            default: None,
        },
    );
    inputs.insert(
        "tags".to_string(),
        InputSchema {
            r#type: "array".to_string(),
            description: "Optional tags to associate with this node".to_string(),
            required: false,
            default: Some(serde_json::Value::Array(vec![])),
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "node_id".to_string(),
        OutputSchema {
            r#type: "string".to_string(),
            description: "ID of the created or updated Codex node".to_string(),
        },
    );
    outputs.insert(
        "summary".to_string(),
        OutputSchema {
            r#type: "string".to_string(),
            description: "Summary of the codified input".to_string(),
        },
    );

    Manifest {
        name: "codexify".to_string(),
        version: "1.0.0".to_string(),
        description: "Converts local files and memory logs into structured knowledge graph entries".to_string(),
        language: "python".to_string(),
        entry_point: "guardian-backend_v2/codexify/main.py".to_string(),
        capabilities: vec![
            "fs:read".to_string(),
            "fs:write".to_string(),
            "memory:index".to_string(),
            "llm".to_string(),
        ],
        schema: Some("schemas/codexify.json".to_string()),
        timeout_sec: 60,
        requirements,
        inputs,
        outputs,
    }
}

/// Generate default manifest for ritual_engine
pub fn generate_ritual_engine_manifest() -> Manifest {
    let mut requirements = HashMap::new();
    requirements.insert("python".to_string(), ">=3.10".to_string());
    requirements.insert("requests".to_string(), "*".to_string());
    requirements.insert("pyyaml".to_string(), "*".to_string());

    let mut inputs = HashMap::new();
    inputs.insert(
        "ritual_type".to_string(),
        InputSchema {
            r#type: "string".to_string(),
            description: "Type of ritual to perform".to_string(),
            required: true,
            default: None,
        },
    );
    inputs.insert(
        "parameters".to_string(),
        InputSchema {
            r#type: "object".to_string(),
            description: "Parameters for the ritual".to_string(),
            required: true,
            default: None,
        },
    );

    let mut outputs = HashMap::new();
    outputs.insert(
        "ritual_id".to_string(),
        OutputSchema {
            r#type: "string".to_string(),
            description: "ID of the created ritual".to_string(),
        },
    );
    outputs.insert(
        "status".to_string(),
        OutputSchema {
            r#type: "string".to_string(),
            description: "Status of the ritual execution".to_string(),
        },
    );

    Manifest {
        name: "ritual_engine".to_string(),
        version: "1.0.0".to_string(),
        description: "Executes memory and ritual operations for ThreadSpace".to_string(),
        language: "python".to_string(),
        entry_point: "guardian-backend_v2/ritual_engine/main.py".to_string(),
        capabilities: vec![
            "memory:process".to_string(),
            "ritual:execute".to_string(),
            "sync:memory".to_string(),
        ],
        schema: Some("schemas/ritual_engine.json".to_string()),
        timeout_sec: 120,
        requirements,
        inputs,
        outputs,
    }
}

/// Generate all default manifests
pub fn generate_all_manifests() -> Vec<(String, Manifest)> {
    vec![
        ("codexify".to_string(), generate_codexify_manifest()),
        ("ritual_engine".to_string(), generate_ritual_engine_manifest()),
    ]
}

/// Create manifest directory structure
pub fn create_manifest_structure() -> BridgeResult<()> {
    let manifests_dir = "threadspace/crates/cli_bridge/manifests";
    fs::create_dir_all(manifests_dir)?;

    for (name, manifest) in generate_all_manifests() {
        let path = format!("{}/{}.json", manifests_dir, name);
        save_manifest(&manifest, &path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_serialization() {
        let manifest = generate_codexify_manifest();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("codexify"));
        assert!(json.contains("knowledge graph"));
    }

    #[test]
    fn test_manifest_save_load() {
        let dir = tempdir().unwrap();
        let manifest = generate_codexify_manifest();
        let path = dir.path().join("test_manifest.json");
        
        save_manifest(&manifest, &path).unwrap();
        let loaded = load_manifest(&path).unwrap();
        
        assert_eq!(loaded.name, manifest.name);
        assert_eq!(loaded.version, manifest.version);
    }
}
