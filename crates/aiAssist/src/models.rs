use serde::{Deserialize, Serialize};

/// Available AI models
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub max_tokens: u32,
}

/// Get available models
pub fn get_available_models() -> Vec<AIModel> {
    vec![
        AIModel {
            id: "qwen3:4b".to_string(),
            name: "Qwen 3 (4B)".to_string(),
            provider: "Ollama".to_string(),
            max_tokens: 4096,
        },
        AIModel {
            id: "llama3:8b".to_string(),
            name: "Llama 3 (8B)".to_string(),
            provider: "Ollama".to_string(),
            max_tokens: 4096,
        },
        AIModel {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: "OpenAI".to_string(),
            max_tokens: 4096,
        },
    ]
}
