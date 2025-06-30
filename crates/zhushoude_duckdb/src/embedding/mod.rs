//! 语义搜索模块
//!
//! 提供中文语义模型集成和向量化服务

pub mod engine;
pub mod model;
pub mod inference;
pub mod cache;
pub mod chinese;
pub mod provider;
pub mod bge;

pub use engine::*;
pub use model::*;
pub use inference::*;
pub use cache::*;
pub use chinese::*;
pub use provider::{EmbeddingProvider, EmbeddingProviderConfig, BGEConfig, ModelInfo, ProviderStats};
pub use bge::{BGEEmbeddingProvider, BGEModel, BGETokenizer};
