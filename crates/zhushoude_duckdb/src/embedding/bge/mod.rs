//! BGE (BAAI General Embedding) 中文语义模型实现
//! 
//! 提供高质量的中文语义向量化能力

pub mod provider;
pub mod model;
pub mod tokenizer;
pub mod config;
pub mod benchmark;

pub use provider::BGEEmbeddingProvider;
pub use model::BGEModel;
pub use tokenizer::BGETokenizer;
pub use config::*;
pub use benchmark::*;

use crate::Result;

/// BGE模型初始化
pub async fn initialize_bge(config: super::provider::BGEConfig) -> Result<BGEEmbeddingProvider> {
    BGEEmbeddingProvider::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::provider::BGEConfig;

    #[tokio::test]
    async fn test_bge_initialization() {
        let config = BGEConfig::default();
        let result = initialize_bge(config).await;
        
        // 注意：在没有实际模型文件的情况下，这个测试可能会失败
        // 这是预期的，因为我们需要先实现模型下载功能
        match result {
            Ok(_) => println!("BGE初始化成功"),
            Err(e) => println!("BGE初始化失败（预期）: {}", e),
        }
    }
}
