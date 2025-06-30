//! 实体关系抽取模块
//! 
//! 基于语法模式和语义规则实现中文文本的实体关系抽取

use crate::nlp::ner::{Entity, EntityType};
use crate::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 关系类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// 包含关系 (A包含B)
    Contains,
    /// 属于关系 (A属于B)
    BelongsTo,
    /// 相关关系 (A与B相关)
    RelatedTo,
    /// 依赖关系 (A依赖B)
    DependsOn,
    /// 位于关系 (A位于B)
    LocatedIn,
    /// 工作于关系 (A工作于B)
    WorksAt,
    /// 创建关系 (A创建B)
    Creates,
    /// 使用关系 (A使用B)
    Uses,
    /// 学习关系 (A学习B)
    Studies,
    /// 研究关系 (A研究B)
    Researches,
    /// 时间关系 (A发生在B时间)
    OccursAt,
    /// 其他关系
    Other,
}

impl RelationType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Contains => "CONTAINS",
            RelationType::BelongsTo => "BELONGS_TO",
            RelationType::RelatedTo => "RELATED_TO",
            RelationType::DependsOn => "DEPENDS_ON",
            RelationType::LocatedIn => "LOCATED_IN",
            RelationType::WorksAt => "WORKS_AT",
            RelationType::Creates => "CREATES",
            RelationType::Uses => "USES",
            RelationType::Studies => "STUDIES",
            RelationType::Researches => "RESEARCHES",
            RelationType::OccursAt => "OCCURS_AT",
            RelationType::Other => "OTHER",
        }
    }
}

/// 实体关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelation {
    /// 主体实体
    pub subject: Entity,
    /// 关系类型
    pub relation_type: RelationType,
    /// 客体实体
    pub object: Entity,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 关系在原文中的位置
    pub relation_span: Option<(usize, usize)>,
    /// 额外属性
    pub properties: HashMap<String, String>,
}

/// 关系模式
#[derive(Debug, Clone)]
struct RelationPattern {
    /// 模式正则表达式
    pattern: Regex,
    /// 关系类型
    relation_type: RelationType,
    /// 主体实体类型约束
    subject_types: Vec<EntityType>,
    /// 客体实体类型约束
    object_types: Vec<EntityType>,
    /// 基础置信度
    base_confidence: f64,
}

/// 中文实体关系抽取器
pub struct ChineseRelationExtractor {
    /// 关系模式列表
    relation_patterns: Vec<RelationPattern>,
}

impl ChineseRelationExtractor {
    /// 创建新的关系抽取器
    pub fn new() -> Result<Self> {
        let mut extractor = Self {
            relation_patterns: Vec::new(),
        };
        
        extractor.initialize_patterns()?;
        
        Ok(extractor)
    }
    
    /// 初始化关系模式
    fn initialize_patterns(&mut self) -> Result<()> {
        // 包含关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)包含(.+)")?,
            relation_type: RelationType::Contains,
            subject_types: vec![EntityType::Organization, EntityType::Location, EntityType::Concept],
            object_types: vec![EntityType::Person, EntityType::Technology, EntityType::Concept],
            base_confidence: 0.8,
        });
        
        // 属于关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)属于(.+)")?,
            relation_type: RelationType::BelongsTo,
            subject_types: vec![EntityType::Person, EntityType::Technology, EntityType::Concept],
            object_types: vec![EntityType::Organization, EntityType::Location, EntityType::Concept],
            base_confidence: 0.8,
        });
        
        // 位于关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)位于(.+)")?,
            relation_type: RelationType::LocatedIn,
            subject_types: vec![EntityType::Person, EntityType::Organization],
            object_types: vec![EntityType::Location],
            base_confidence: 0.9,
        });
        
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)在(.+)")?,
            relation_type: RelationType::LocatedIn,
            subject_types: vec![EntityType::Person, EntityType::Organization],
            object_types: vec![EntityType::Location],
            base_confidence: 0.7,
        });
        
        // 工作关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)工作于(.+)")?,
            relation_type: RelationType::WorksAt,
            subject_types: vec![EntityType::Person],
            object_types: vec![EntityType::Organization],
            base_confidence: 0.9,
        });
        
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)就职于(.+)")?,
            relation_type: RelationType::WorksAt,
            subject_types: vec![EntityType::Person],
            object_types: vec![EntityType::Organization],
            base_confidence: 0.9,
        });
        
        // 创建关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)创建了(.+)")?,
            relation_type: RelationType::Creates,
            subject_types: vec![EntityType::Person, EntityType::Organization],
            object_types: vec![EntityType::Technology, EntityType::Product, EntityType::Concept],
            base_confidence: 0.8,
        });
        
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)开发了(.+)")?,
            relation_type: RelationType::Creates,
            subject_types: vec![EntityType::Person, EntityType::Organization],
            object_types: vec![EntityType::Technology, EntityType::Product],
            base_confidence: 0.8,
        });
        
        // 使用关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)使用(.+)")?,
            relation_type: RelationType::Uses,
            subject_types: vec![EntityType::Person, EntityType::Organization],
            object_types: vec![EntityType::Technology, EntityType::Product],
            base_confidence: 0.7,
        });
        
        // 学习关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)学习(.+)")?,
            relation_type: RelationType::Studies,
            subject_types: vec![EntityType::Person],
            object_types: vec![EntityType::Technology, EntityType::Concept],
            base_confidence: 0.8,
        });
        
        // 研究关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)研究(.+)")?,
            relation_type: RelationType::Researches,
            subject_types: vec![EntityType::Person, EntityType::Organization],
            object_types: vec![EntityType::Technology, EntityType::Concept],
            base_confidence: 0.8,
        });
        
        // 依赖关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)依赖(.+)")?,
            relation_type: RelationType::DependsOn,
            subject_types: vec![EntityType::Technology, EntityType::Product],
            object_types: vec![EntityType::Technology, EntityType::Product],
            base_confidence: 0.8,
        });
        
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)基于(.+)")?,
            relation_type: RelationType::DependsOn,
            subject_types: vec![EntityType::Technology, EntityType::Product, EntityType::Concept],
            object_types: vec![EntityType::Technology, EntityType::Concept],
            base_confidence: 0.7,
        });
        
        // 时间关系模式
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)在(.+?)发生")?,
            relation_type: RelationType::OccursAt,
            subject_types: vec![EntityType::Concept, EntityType::Other],
            object_types: vec![EntityType::Time],
            base_confidence: 0.8,
        });
        
        // 相关关系模式（通用）
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)与(.+?)相关")?,
            relation_type: RelationType::RelatedTo,
            subject_types: vec![],  // 不限制类型
            object_types: vec![],   // 不限制类型
            base_confidence: 0.6,
        });
        
        self.relation_patterns.push(RelationPattern {
            pattern: Regex::new(r"(.+?)和(.+?)有关")?,
            relation_type: RelationType::RelatedTo,
            subject_types: vec![],
            object_types: vec![],
            base_confidence: 0.6,
        });
        
        Ok(())
    }
    
    /// 从文本和实体列表中抽取关系
    pub fn extract_relations(&self, text: &str, entities: &[Entity]) -> Result<Vec<EntityRelation>> {
        let mut relations = Vec::new();

        // 性能优化：限制处理的实体数量
        let max_entities = 50; // 限制最大实体数量以避免性能问题
        let limited_entities = if entities.len() > max_entities {
            // 按置信度排序，取前50个最可信的实体
            let mut sorted_entities = entities.to_vec();
            sorted_entities.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
            sorted_entities.truncate(max_entities);
            sorted_entities
        } else {
            entities.to_vec()
        };

        // 基于模式的关系抽取
        relations.extend(self.extract_by_patterns_optimized(text, &limited_entities)?);

        // 基于实体共现的关系推断（仅在实体数量较少时执行）
        if limited_entities.len() <= 20 {
            relations.extend(self.extract_by_cooccurrence_optimized(text, &limited_entities));
        }

        // 去重和排序
        relations.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal).reverse());
        relations = self.remove_duplicate_relations(relations);

        // 限制返回的关系数量
        relations.truncate(100);

        Ok(relations)
    }
    
    /// 基于模式抽取关系（优化版本）
    fn extract_by_patterns_optimized(&self, text: &str, entities: &[Entity]) -> Result<Vec<EntityRelation>> {
        let mut relations = Vec::new();

        // 限制处理的模式数量，优先处理高置信度模式
        let max_patterns = 8;
        let limited_patterns = &self.relation_patterns[..max_patterns.min(self.relation_patterns.len())];

        for pattern in limited_patterns {
            let mut match_count = 0;
            for mat in pattern.pattern.find_iter(text) {
                match_count += 1;
                if match_count > 20 { // 限制每个模式的匹配数量
                    break;
                }

                let match_text = mat.as_str();

                // 查找匹配范围内的实体
                let entities_in_match = entities.iter()
                    .filter(|e| e.start >= mat.start() && e.end <= mat.end())
                    .collect::<Vec<_>>();

                if entities_in_match.len() >= 2 && entities_in_match.len() <= 5 {
                    // 限制实体对的数量以避免组合爆炸
                    let max_pairs = 3;
                    let mut pair_count = 0;

                    // 尝试匹配主体和客体
                    for i in 0..entities_in_match.len() {
                        for j in (i + 1)..entities_in_match.len() {
                            pair_count += 1;
                            if pair_count > max_pairs {
                                break;
                            }

                            let subject = entities_in_match[i];
                            let object = entities_in_match[j];

                            // 检查实体类型约束
                            if self.check_type_constraints(subject, object, pattern) {
                                let confidence = self.calculate_relation_confidence(
                                    pattern, subject, object, match_text
                                );

                                if confidence > 0.5 {
                                    relations.push(EntityRelation {
                                        subject: subject.clone(),
                                        relation_type: pattern.relation_type.clone(),
                                        object: object.clone(),
                                        confidence,
                                        relation_span: Some((mat.start(), mat.end())),
                                        properties: HashMap::new(),
                                    });
                                }
                            }
                        }
                        if pair_count > max_pairs {
                            break;
                        }
                    }
                }
            }
        }

        Ok(relations)
    }

    /// 基于模式抽取关系（原版本，保留兼容性）
    fn extract_by_patterns(&self, text: &str, entities: &[Entity]) -> Result<Vec<EntityRelation>> {
        self.extract_by_patterns_optimized(text, entities)
    }
    
    /// 基于实体共现抽取关系（优化版本）
    fn extract_by_cooccurrence_optimized(&self, text: &str, entities: &[Entity]) -> Vec<EntityRelation> {
        let mut relations = Vec::new();

        // 简化句子分割，使用更快的方法
        let sentences: Vec<&str> = text.split(&['。', '！', '？', ';', '\n'][..])
            .filter(|s| !s.trim().is_empty())
            .take(20) // 限制处理的句子数量
            .collect();

        for sentence in sentences {
            // 预先计算实体在句子中的位置，避免重复字符串操作
            let mut entities_in_sentence = Vec::new();
            for entity in entities {
                if entity.start < text.len() && entity.end <= text.len() {
                    let entity_text = &text[entity.start..entity.end];
                    if sentence.contains(entity_text) {
                        entities_in_sentence.push(entity);
                    }
                }
            }

            // 限制每个句子中处理的实体数量
            if entities_in_sentence.len() >= 2 && entities_in_sentence.len() <= 6 {
                let max_pairs = 3; // 限制每个句子中的实体对数量
                let mut pair_count = 0;

                for i in 0..entities_in_sentence.len() {
                    for j in (i + 1)..entities_in_sentence.len() {
                        pair_count += 1;
                        if pair_count > max_pairs {
                            break;
                        }

                        let subject = entities_in_sentence[i];
                        let object = entities_in_sentence[j];

                        // 基于实体类型推断可能的关系
                        if let Some(relation_type) = self.infer_relation_type(subject, object) {
                            let confidence = 0.4; // 共现关系置信度较低

                            relations.push(EntityRelation {
                                subject: subject.clone(),
                                relation_type,
                                object: object.clone(),
                                confidence,
                                relation_span: None,
                                properties: HashMap::new(),
                            });
                        }
                    }
                    if pair_count > max_pairs {
                        break;
                    }
                }
            }
        }

        relations
    }

    /// 基于实体共现抽取关系（原版本，保留兼容性）
    fn extract_by_cooccurrence(&self, text: &str, entities: &[Entity]) -> Vec<EntityRelation> {
        self.extract_by_cooccurrence_optimized(text, entities)
    }
    
    /// 检查实体类型约束
    fn check_type_constraints(&self, subject: &Entity, object: &Entity, pattern: &RelationPattern) -> bool {
        let subject_ok = pattern.subject_types.is_empty() || 
            pattern.subject_types.contains(&subject.entity_type);
        let object_ok = pattern.object_types.is_empty() || 
            pattern.object_types.contains(&object.entity_type);
        
        subject_ok && object_ok
    }
    
    /// 计算关系置信度
    fn calculate_relation_confidence(&self, pattern: &RelationPattern, subject: &Entity, object: &Entity, match_text: &str) -> f64 {
        let mut confidence = pattern.base_confidence;
        
        // 根据实体置信度调整
        let entity_confidence = (subject.confidence + object.confidence) / 2.0;
        confidence *= entity_confidence;
        
        // 根据匹配文本长度调整
        let length_factor = match match_text.chars().count() {
            0..=10 => 0.8,
            11..=30 => 1.0,
            31..=50 => 0.9,
            _ => 0.7,
        };
        confidence *= length_factor;
        
        confidence.min(1.0)
    }
    
    /// 基于实体类型推断关系类型
    fn infer_relation_type(&self, subject: &Entity, object: &Entity) -> Option<RelationType> {
        match (&subject.entity_type, &object.entity_type) {
            (EntityType::Person, EntityType::Organization) => Some(RelationType::WorksAt),
            (EntityType::Person, EntityType::Location) => Some(RelationType::LocatedIn),
            (EntityType::Organization, EntityType::Location) => Some(RelationType::LocatedIn),
            (EntityType::Person, EntityType::Technology) => Some(RelationType::Uses),
            (EntityType::Person, EntityType::Concept) => Some(RelationType::Studies),
            (EntityType::Technology, EntityType::Technology) => Some(RelationType::DependsOn),
            _ => Some(RelationType::RelatedTo),
        }
    }
    
    /// 将文本分割为句子
    fn split_into_sentences(&self, text: &str) -> Vec<String> {
        text.split(&['。', '！', '？', '.', '!', '?'][..])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
    
    /// 移除重复的关系
    fn remove_duplicate_relations(&self, mut relations: Vec<EntityRelation>) -> Vec<EntityRelation> {
        relations.sort_by(|a, b| {
            a.subject.text.cmp(&b.subject.text)
                .then_with(|| a.object.text.cmp(&b.object.text))
                .then_with(|| a.relation_type.as_str().cmp(b.relation_type.as_str()))
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        let mut result = Vec::new();
        let mut last_key: Option<(String, String, String)> = None;
        
        for relation in relations {
            let key = (
                relation.subject.text.clone(),
                relation.object.text.clone(),
                relation.relation_type.as_str().to_string(),
            );
            
            if last_key.as_ref() != Some(&key) {
                result.push(relation);
                last_key = Some(key);
            }
        }
        
        result
    }
}

impl Default for ChineseRelationExtractor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            relation_patterns: Vec::new(),
        })
    }
}
