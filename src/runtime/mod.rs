use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeType {
    Node,
    Bun,
    TypeScript,
}

impl RuntimeType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "node" | "nodejs" => Some(Self::Node),
            "bun" => Some(Self::Bun),
            "typescript" | "ts" => Some(Self::TypeScript),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Node => "node".to_string(),
            Self::Bun => "bun".to_string(),
            Self::TypeScript => "typescript".to_string(),
        }
    }

    pub fn get_file_extension(&self) -> &'static str {
        match self {
            Self::Node => "js",
            Self::Bun => "js",
            Self::TypeScript => "ts",
        }
    }

    pub fn get_run_command(&self) -> Vec<&'static str> {
        match self {
            Self::Node => vec!["node", "index.js"],
            Self::Bun => vec!["bun", "run", "index.js"],
            Self::TypeScript => vec!["npx", "ts-node", "index.ts"],
        }
    }

    pub fn get_docker_image(&self) -> &'static str {
        match self {
            Self::Node => "node:18-alpine",
            Self::Bun => "oven/bun:1-alpine",
            Self::TypeScript => "node:18-alpine",
        }
    }

    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Node | Self::Bun | Self::TypeScript)
    }
}

pub fn validate_runtime(runtime: &str) -> Result<RuntimeType, String> {
    RuntimeType::from_string(runtime)
        .ok_or_else(|| format!("Unsupported runtime: {}. Supported runtimes: node, bun, typescript", runtime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_from_string() {
        assert!(matches!(RuntimeType::from_string("node"), Some(RuntimeType::Node)));
        assert!(matches!(RuntimeType::from_string("nodejs"), Some(RuntimeType::Node)));
        assert!(matches!(RuntimeType::from_string("NODE"), Some(RuntimeType::Node)));
        assert!(matches!(RuntimeType::from_string("bun"), Some(RuntimeType::Bun)));
        assert!(matches!(RuntimeType::from_string("BUN"), Some(RuntimeType::Bun)));
        assert!(matches!(RuntimeType::from_string("typescript"), Some(RuntimeType::TypeScript)));
        assert!(matches!(RuntimeType::from_string("ts"), Some(RuntimeType::TypeScript)));
        assert!(matches!(RuntimeType::from_string("TypeScript"), Some(RuntimeType::TypeScript)));
        assert!(RuntimeType::from_string("python").is_none());
        assert!(RuntimeType::from_string("rust").is_none());
        assert!(RuntimeType::from_string("").is_none());
    }

    #[test]
    fn test_runtime_to_string() {
        assert_eq!(RuntimeType::Node.to_string(), "node");
        assert_eq!(RuntimeType::Bun.to_string(), "bun");
        assert_eq!(RuntimeType::TypeScript.to_string(), "typescript");
    }

    #[test]
    fn test_runtime_commands() {
        assert_eq!(RuntimeType::Node.get_run_command(), vec!["node", "index.js"]);
        assert_eq!(RuntimeType::Bun.get_run_command(), vec!["bun", "run", "index.js"]);
        assert_eq!(RuntimeType::TypeScript.get_run_command(), vec!["npx", "ts-node", "index.ts"]);
    }

    #[test]
    fn test_file_extensions() {
        assert_eq!(RuntimeType::Node.get_file_extension(), "js");
        assert_eq!(RuntimeType::Bun.get_file_extension(), "js");
        assert_eq!(RuntimeType::TypeScript.get_file_extension(), "ts");
    }

    #[test]
    fn test_docker_images() {
        assert_eq!(RuntimeType::Node.get_docker_image(), "node:18-alpine");
        assert_eq!(RuntimeType::Bun.get_docker_image(), "oven/bun:1-alpine");
        assert_eq!(RuntimeType::TypeScript.get_docker_image(), "node:18-alpine");
    }

    #[test]
    fn test_is_supported() {
        assert!(RuntimeType::Node.is_supported());
        assert!(RuntimeType::Bun.is_supported());
        assert!(RuntimeType::TypeScript.is_supported());
    }

    #[test]
    fn test_validate_runtime() {
        assert!(validate_runtime("node").is_ok());
        assert!(validate_runtime("bun").is_ok());
        assert!(validate_runtime("typescript").is_ok());
        
        let error = validate_runtime("python").unwrap_err();
        assert!(error.contains("Unsupported runtime"));
        assert!(error.contains("python"));
    }

    #[test]
    fn test_serde_serialization() {
        let runtime = RuntimeType::Node;
        let serialized = serde_json::to_string(&runtime).unwrap();
        let deserialized: RuntimeType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, RuntimeType::Node));
    }
}