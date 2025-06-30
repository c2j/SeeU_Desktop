use crate::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 文本分块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// 分块大小（字符数）
    pub chunk_size: usize,
    /// 分块重叠大小（字符数）
    pub overlap_size: usize,
    /// 最小分块大小
    pub min_chunk_size: usize,
    /// 是否保持语义边界
    pub preserve_semantic_boundaries: bool,
    /// 分块策略
    pub strategy: ChunkingStrategy,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024,
            overlap_size: 128,
            min_chunk_size: 100,
            preserve_semantic_boundaries: true,
            strategy: ChunkingStrategy::Semantic,
        }
    }
}

/// 分块策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    /// 固定大小分块
    FixedSize,
    /// 语义边界分块
    Semantic,
    /// 段落分块
    Paragraph,
    /// 句子分块
    Sentence,
    /// 混合策略
    Hybrid,
}

/// 文本分块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    /// 分块ID
    pub id: String,
    /// 文档ID
    pub document_id: String,
    /// 分块索引
    pub chunk_index: usize,
    /// 分块内容
    pub content: String,
    /// 在原文档中的起始位置
    pub start_offset: usize,
    /// 在原文档中的结束位置
    pub end_offset: usize,
    /// 分块元数据
    pub metadata: ChunkMetadata,
}

/// 分块元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// 字符数
    pub char_count: usize,
    /// 词数（估算）
    pub word_count: usize,
    /// 句子数
    pub sentence_count: usize,
    /// 段落数
    pub paragraph_count: usize,
    /// 语言
    pub language: String,
    /// 内容类型
    pub content_type: String,
    /// 质量分数
    pub quality_score: f32,
    /// 额外属性
    pub properties: HashMap<String, String>,
}

/// 文本分块器
pub struct TextChunker {
    config: ChunkingConfig,
}

impl TextChunker {
    /// 创建新的文本分块器
    pub fn new(config: ChunkingConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建分块器
    pub fn default() -> Self {
        Self::new(ChunkingConfig::default())
    }

    /// 对文本进行分块
    pub fn chunk_text(&self, document_id: &str, text: &str) -> Result<Vec<TextChunk>> {
        match self.config.strategy {
            ChunkingStrategy::FixedSize => self.chunk_fixed_size(document_id, text),
            ChunkingStrategy::Semantic => self.chunk_semantic(document_id, text),
            ChunkingStrategy::Paragraph => self.chunk_paragraph(document_id, text),
            ChunkingStrategy::Sentence => self.chunk_sentence(document_id, text),
            ChunkingStrategy::Hybrid => self.chunk_hybrid(document_id, text),
        }
    }

    /// 固定大小分块
    fn chunk_fixed_size(&self, document_id: &str, text: &str) -> Result<Vec<TextChunk>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        
        let mut start = 0;
        let mut chunk_index = 0;
        
        while start < total_chars {
            let end = std::cmp::min(start + self.config.chunk_size, total_chars);
            let chunk_content: String = chars[start..end].iter().collect();
            
            // 跳过太小的分块
            if chunk_content.trim().len() < self.config.min_chunk_size {
                break;
            }
            
            let chunk = TextChunk {
                id: format!("{}_{}", document_id, chunk_index),
                document_id: document_id.to_string(),
                chunk_index,
                content: chunk_content.clone(),
                start_offset: start,
                end_offset: end,
                metadata: self.create_metadata(&chunk_content),
            };
            
            chunks.push(chunk);
            
            // 计算下一个分块的起始位置（考虑重叠）
            start = if end >= total_chars {
                total_chars
            } else {
                std::cmp::max(start + self.config.chunk_size - self.config.overlap_size, start + 1)
            };
            
            chunk_index += 1;
        }
        
        Ok(chunks)
    }

    /// 语义边界分块
    fn chunk_semantic(&self, document_id: &str, text: &str) -> Result<Vec<TextChunk>> {
        // 首先按段落分割
        let paragraphs = self.split_paragraphs(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut chunk_index = 0;
        
        for paragraph in paragraphs {
            let paragraph_trimmed = paragraph.trim();
            if paragraph_trimmed.is_empty() {
                continue;
            }
            
            // 检查添加这个段落是否会超过分块大小
            let potential_chunk = if current_chunk.is_empty() {
                paragraph_trimmed.to_string()
            } else {
                format!("{}\n\n{}", current_chunk, paragraph_trimmed)
            };
            
            if potential_chunk.chars().count() <= self.config.chunk_size {
                current_chunk = potential_chunk;
            } else {
                // 如果当前分块不为空，先保存它
                if !current_chunk.is_empty() && current_chunk.trim().len() >= self.config.min_chunk_size {
                    let chunk = TextChunk {
                        id: format!("{}_{}", document_id, chunk_index),
                        document_id: document_id.to_string(),
                        chunk_index,
                        content: current_chunk.clone(),
                        start_offset: current_start,
                        end_offset: current_start + current_chunk.chars().count(),
                        metadata: self.create_metadata(&current_chunk),
                    };
                    chunks.push(chunk);
                    chunk_index += 1;
                }
                
                // 如果单个段落太大，需要进一步分割
                if paragraph_trimmed.chars().count() > self.config.chunk_size {
                    let sub_chunks = self.chunk_large_paragraph(document_id, paragraph_trimmed, &mut chunk_index)?;
                    chunks.extend(sub_chunks);
                    current_chunk = String::new();
                } else {
                    current_chunk = paragraph_trimmed.to_string();
                }
                
                current_start = text.find(paragraph_trimmed).unwrap_or(0);
            }
        }
        
        // 处理最后一个分块
        if !current_chunk.is_empty() && current_chunk.trim().len() >= self.config.min_chunk_size {
            let chunk = TextChunk {
                id: format!("{}_{}", document_id, chunk_index),
                document_id: document_id.to_string(),
                chunk_index,
                content: current_chunk.clone(),
                start_offset: current_start,
                end_offset: current_start + current_chunk.chars().count(),
                metadata: self.create_metadata(&current_chunk),
            };
            chunks.push(chunk);
        }
        
        Ok(chunks)
    }

    /// 段落分块
    fn chunk_paragraph(&self, document_id: &str, text: &str) -> Result<Vec<TextChunk>> {
        let paragraphs = self.split_paragraphs(text);
        let mut chunks = Vec::new();
        let mut current_offset = 0;
        
        for (index, paragraph) in paragraphs.iter().enumerate() {
            let paragraph_trimmed = paragraph.trim();
            if paragraph_trimmed.len() < self.config.min_chunk_size {
                current_offset += paragraph.chars().count();
                continue;
            }
            
            let chunk = TextChunk {
                id: format!("{}_{}", document_id, index),
                document_id: document_id.to_string(),
                chunk_index: index,
                content: paragraph_trimmed.to_string(),
                start_offset: current_offset,
                end_offset: current_offset + paragraph.chars().count(),
                metadata: self.create_metadata(paragraph_trimmed),
            };
            
            chunks.push(chunk);
            current_offset += paragraph.chars().count();
        }
        
        Ok(chunks)
    }

    /// 句子分块
    fn chunk_sentence(&self, document_id: &str, text: &str) -> Result<Vec<TextChunk>> {
        let sentences = self.split_sentences(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut chunk_index = 0;
        
        for sentence in sentences {
            let sentence_trimmed = sentence.trim();
            if sentence_trimmed.is_empty() {
                continue;
            }
            
            let potential_chunk = if current_chunk.is_empty() {
                sentence_trimmed.to_string()
            } else {
                format!("{}。{}", current_chunk, sentence_trimmed)
            };
            
            if potential_chunk.chars().count() <= self.config.chunk_size {
                current_chunk = potential_chunk;
            } else {
                if !current_chunk.is_empty() && current_chunk.trim().len() >= self.config.min_chunk_size {
                    let chunk = TextChunk {
                        id: format!("{}_{}", document_id, chunk_index),
                        document_id: document_id.to_string(),
                        chunk_index,
                        content: current_chunk.clone(),
                        start_offset: current_start,
                        end_offset: current_start + current_chunk.chars().count(),
                        metadata: self.create_metadata(&current_chunk),
                    };
                    chunks.push(chunk);
                    chunk_index += 1;
                }
                
                current_chunk = sentence_trimmed.to_string();
                current_start = text.find(sentence_trimmed).unwrap_or(0);
            }
        }
        
        // 处理最后一个分块
        if !current_chunk.is_empty() && current_chunk.trim().len() >= self.config.min_chunk_size {
            let chunk = TextChunk {
                id: format!("{}_{}", document_id, chunk_index),
                document_id: document_id.to_string(),
                chunk_index,
                content: current_chunk.clone(),
                start_offset: current_start,
                end_offset: current_start + current_chunk.chars().count(),
                metadata: self.create_metadata(&current_chunk),
            };
            chunks.push(chunk);
        }
        
        Ok(chunks)
    }

    /// 混合策略分块
    fn chunk_hybrid(&self, document_id: &str, text: &str) -> Result<Vec<TextChunk>> {
        // 先尝试语义分块，如果分块太大则使用固定大小分块
        let semantic_chunks = self.chunk_semantic(document_id, text)?;
        let mut final_chunks = Vec::new();
        
        for chunk in semantic_chunks {
            if chunk.content.chars().count() <= self.config.chunk_size {
                final_chunks.push(chunk);
            } else {
                // 对大分块进行固定大小分割
                let sub_chunker = TextChunker::new(ChunkingConfig {
                    strategy: ChunkingStrategy::FixedSize,
                    ..self.config.clone()
                });
                let sub_chunks = sub_chunker.chunk_fixed_size(&chunk.id, &chunk.content)?;
                final_chunks.extend(sub_chunks);
            }
        }
        
        Ok(final_chunks)
    }

    /// 分割大段落
    fn chunk_large_paragraph(&self, document_id: &str, paragraph: &str, chunk_index: &mut usize) -> Result<Vec<TextChunk>> {
        let sentences = self.split_sentences(paragraph);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;

        for sentence in sentences {
            let sentence_trimmed = sentence.trim();
            if sentence_trimmed.is_empty() {
                continue;
            }

            let potential_chunk = if current_chunk.is_empty() {
                sentence_trimmed.to_string()
            } else {
                format!("{}。{}", current_chunk, sentence_trimmed)
            };

            if potential_chunk.chars().count() <= self.config.chunk_size {
                current_chunk = potential_chunk;
            } else {
                if !current_chunk.is_empty() {
                    let chunk = TextChunk {
                        id: format!("{}_{}", document_id, *chunk_index),
                        document_id: document_id.to_string(),
                        chunk_index: *chunk_index,
                        content: current_chunk.clone(),
                        start_offset: current_start,
                        end_offset: current_start + current_chunk.chars().count(),
                        metadata: self.create_metadata(&current_chunk),
                    };
                    chunks.push(chunk);
                    *chunk_index += 1;
                }

                current_chunk = sentence_trimmed.to_string();
                current_start = paragraph.find(sentence_trimmed).unwrap_or(0);
            }
        }

        if !current_chunk.is_empty() && current_chunk.trim().len() >= self.config.min_chunk_size {
            let chunk = TextChunk {
                id: format!("{}_{}", document_id, *chunk_index),
                document_id: document_id.to_string(),
                chunk_index: *chunk_index,
                content: current_chunk.clone(),
                start_offset: current_start,
                end_offset: current_start + current_chunk.chars().count(),
                metadata: self.create_metadata(&current_chunk),
            };
            chunks.push(chunk);
            *chunk_index += 1;
        }

        Ok(chunks)
    }

    /// 分割段落
    fn split_paragraphs(&self, text: &str) -> Vec<String> {
        text.split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 分割句子（中文优化）
    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            current_sentence.push(ch);

            // 中文句子结束标点
            if matches!(ch, '。' | '！' | '？' | '；' | '.' | '!' | '?') {
                // 检查下一个字符是否是引号或括号
                let next_char = chars.get(i + 1);
                if let Some(&next_ch) = next_char {
                    if matches!(next_ch, '"' | '"' | ')' | '）' | ']' | '】') {
                        continue;
                    }
                }

                sentences.push(current_sentence.trim().to_string());
                current_sentence.clear();
            }
        }

        // 添加最后一个句子
        if !current_sentence.trim().is_empty() {
            sentences.push(current_sentence.trim().to_string());
        }

        sentences.into_iter().filter(|s| !s.is_empty()).collect()
    }

    /// 创建分块元数据
    fn create_metadata(&self, content: &str) -> ChunkMetadata {
        let char_count = content.chars().count();
        let word_count = self.estimate_word_count(content);
        let sentence_count = self.count_sentences(content);
        let paragraph_count = self.count_paragraphs(content);
        let language = self.detect_language(content);
        let content_type = self.detect_content_type(content);
        let quality_score = self.calculate_quality_score(content);

        ChunkMetadata {
            char_count,
            word_count,
            sentence_count,
            paragraph_count,
            language,
            content_type,
            quality_score,
            properties: HashMap::new(),
        }
    }

    /// 估算词数
    fn estimate_word_count(&self, content: &str) -> usize {
        // 中文按字符数估算，英文按空格分割
        let chinese_chars = content.chars().filter(|c| self.is_chinese_char(*c)).count();
        let english_words = content.split_whitespace()
            .filter(|word| word.chars().any(|c| c.is_ascii_alphabetic()))
            .count();

        chinese_chars + english_words
    }

    /// 判断是否为中文字符
    fn is_chinese_char(&self, ch: char) -> bool {
        matches!(ch as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
    }

    /// 计算句子数
    fn count_sentences(&self, content: &str) -> usize {
        content.chars()
            .filter(|&c| matches!(c, '。' | '！' | '？' | '.' | '!' | '?'))
            .count()
    }

    /// 计算段落数
    fn count_paragraphs(&self, content: &str) -> usize {
        content.split("\n\n").filter(|s| !s.trim().is_empty()).count()
    }

    /// 检测语言
    fn detect_language(&self, content: &str) -> String {
        let chinese_chars = content.chars().filter(|c| self.is_chinese_char(*c)).count();
        let total_chars = content.chars().filter(|c| c.is_alphabetic()).count();

        if chinese_chars > total_chars / 2 {
            "zh".to_string()
        } else {
            "en".to_string()
        }
    }

    /// 检测内容类型
    fn detect_content_type(&self, content: &str) -> String {
        if content.contains("```") || content.contains("function") || content.contains("class") {
            "code".to_string()
        } else if content.contains("# ") || content.contains("## ") {
            "markdown".to_string()
        } else {
            "text".to_string()
        }
    }

    /// 计算质量分数
    fn calculate_quality_score(&self, content: &str) -> f32 {
        let mut score = 1.0;

        // 长度惩罚
        let char_count = content.chars().count();
        if char_count < 50 {
            score *= 0.5;
        } else if char_count > 2000 {
            score *= 0.8;
        }

        // 重复内容惩罚
        let unique_chars: std::collections::HashSet<char> = content.chars().collect();
        let diversity = unique_chars.len() as f32 / char_count as f32;
        score *= diversity.min(1.0);

        // 标点符号比例
        let punct_count = content.chars().filter(|c| c.is_ascii_punctuation()).count();
        let punct_ratio = punct_count as f32 / char_count as f32;
        if punct_ratio > 0.3 {
            score *= 0.7;
        }

        score.max(0.1).min(1.0)
    }

    /// 获取分块统计信息
    pub fn get_chunking_stats(&self, chunks: &[TextChunk]) -> ChunkingStats {
        let total_chunks = chunks.len();
        let total_chars: usize = chunks.iter().map(|c| c.metadata.char_count).sum();
        let avg_chunk_size = if total_chunks > 0 { total_chars / total_chunks } else { 0 };
        let min_chunk_size = chunks.iter().map(|c| c.metadata.char_count).min().unwrap_or(0);
        let max_chunk_size = chunks.iter().map(|c| c.metadata.char_count).max().unwrap_or(0);
        let avg_quality_score = if total_chunks > 0 {
            chunks.iter().map(|c| c.metadata.quality_score).sum::<f32>() / total_chunks as f32
        } else {
            0.0
        };

        ChunkingStats {
            total_chunks,
            total_chars,
            avg_chunk_size,
            min_chunk_size,
            max_chunk_size,
            avg_quality_score,
        }
    }
}

/// 分块统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingStats {
    pub total_chunks: usize,
    pub total_chars: usize,
    pub avg_chunk_size: usize,
    pub min_chunk_size: usize,
    pub max_chunk_size: usize,
    pub avg_quality_score: f32,
}
