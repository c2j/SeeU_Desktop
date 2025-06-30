//! 向量搜索模块
//! 
//! 提供向量存储、搜索和索引管理

pub mod storage;
pub mod search;
pub mod index;
pub mod chunk_index;

pub use storage::*;
pub use search::{SemanticSearchEngine, SearchStats};
pub use index::{
    VectorIndexManager, IndexType, IndexMetadata, IndexStats,
    VectorSearchResult, SearchPerformanceStats
};
pub use chunk_index::{
    ChunkVectorIndexManager, ChunkVectorConfig, ChunkIndexType,
    ChunkVectorIndex, ChunkSearchResult, ChunkIndexStats
};
