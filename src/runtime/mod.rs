use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeType {
    Node,
    Bun,
    TypeScript,
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_serde_serialization() {
        let runtime = RuntimeType::Node;
        let serialized = serde_json::to_string(&runtime).unwrap();
        let deserialized: RuntimeType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, RuntimeType::Node));
    }
}