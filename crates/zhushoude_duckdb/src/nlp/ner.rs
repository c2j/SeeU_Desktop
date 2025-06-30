//! 中文命名实体识别（NER）模块
//! 
//! 基于规则和模式匹配的中文实体识别功能

use crate::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 实体类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// 人名
    Person,
    /// 地名
    Location,
    /// 机构名
    Organization,
    /// 时间
    Time,
    /// 概念/术语
    Concept,
    /// 技术名词
    Technology,
    /// 产品名称
    Product,
    /// 其他
    Other,
}

impl EntityType {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Person => "PERSON",
            EntityType::Location => "LOCATION", 
            EntityType::Organization => "ORGANIZATION",
            EntityType::Time => "TIME",
            EntityType::Concept => "CONCEPT",
            EntityType::Technology => "TECHNOLOGY",
            EntityType::Product => "PRODUCT",
            EntityType::Other => "OTHER",
        }
    }
}

/// 识别的实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体文本
    pub text: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 在原文中的起始位置
    pub start: usize,
    /// 在原文中的结束位置
    pub end: usize,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 额外属性
    pub properties: HashMap<String, String>,
}

/// 中文命名实体识别器
pub struct ChineseNER {
    /// 人名模式
    person_patterns: Vec<Regex>,
    /// 地名模式
    location_patterns: Vec<Regex>,
    /// 机构名模式
    organization_patterns: Vec<Regex>,
    /// 时间模式
    time_patterns: Vec<Regex>,
    /// 概念模式
    concept_patterns: Vec<Regex>,
    /// 技术名词模式
    technology_patterns: Vec<Regex>,
    /// 产品名称模式
    product_patterns: Vec<Regex>,
    /// 常见人名词典
    person_dict: Vec<String>,
    /// 常见地名词典
    location_dict: Vec<String>,
    /// 常见机构词典
    organization_dict: Vec<String>,
    /// 技术术语词典
    technology_dict: Vec<String>,
}

impl ChineseNER {
    /// 创建新的中文NER实例
    pub fn new() -> Result<Self> {
        let mut ner = Self {
            person_patterns: Vec::new(),
            location_patterns: Vec::new(),
            organization_patterns: Vec::new(),
            time_patterns: Vec::new(),
            concept_patterns: Vec::new(),
            technology_patterns: Vec::new(),
            product_patterns: Vec::new(),
            person_dict: Vec::new(),
            location_dict: Vec::new(),
            organization_dict: Vec::new(),
            technology_dict: Vec::new(),
        };
        
        ner.initialize_patterns()?;
        ner.initialize_dictionaries();
        
        Ok(ner)
    }
    
    /// 初始化正则表达式模式
    fn initialize_patterns(&mut self) -> Result<()> {
        // 人名模式
        self.person_patterns = vec![
            Regex::new(r"[王李张刘陈杨黄赵吴周徐孙马朱胡郭何高林罗郑梁谢宋唐许韩冯邓曹彭曾萧田董袁潘于蒋蔡余杜叶程苏魏吕丁任沈姚卢姜崔钟谭陆汪范金石廖贾夏韦付方白邹孟熊秦邱江尹薛闫段雷侯龙史陶黎贺顾毛郝龚邵万钱严覃武戴莫孔向汤][一-龯]{1,3}")?,
            Regex::new(r"[A-Z][a-z]+\s+[A-Z][a-z]+")?, // 英文人名
        ];
        
        // 地名模式
        self.location_patterns = vec![
            Regex::new(r"[一-龯]{2,8}[省市区县镇村街道路巷弄]")?,
            Regex::new(r"[一-龯]{2,6}[大学学院]")?,
            Regex::new(r"[一-龯]{2,8}[公司企业集团]")?,
        ];
        
        // 机构名模式
        self.organization_patterns = vec![
            Regex::new(r"[一-龯]{2,10}[公司企业集团有限责任股份]")?,
            Regex::new(r"[一-龯]{2,8}[大学学院研究所实验室]")?,
            Regex::new(r"[一-龯]{2,8}[部委局署厅处科]")?,
            Regex::new(r"[一-龯]{2,8}[银行保险证券基金]")?,
        ];
        
        // 时间模式
        self.time_patterns = vec![
            Regex::new(r"\d{4}年\d{1,2}月\d{1,2}日")?,
            Regex::new(r"\d{4}-\d{1,2}-\d{1,2}")?,
            Regex::new(r"\d{1,2}月\d{1,2}日")?,
            Regex::new(r"[今明昨前后][天日年月周]")?,
            Regex::new(r"[上下][午周月年]")?,
        ];
        
        // 概念模式
        self.concept_patterns = vec![
            Regex::new(r"[一-龯]{2,6}[理论方法技术算法模型]")?,
            Regex::new(r"[一-龯]{2,6}[系统平台框架架构]")?,
        ];
        
        // 技术名词模式
        self.technology_patterns = vec![
            Regex::new(r"[A-Z]{2,10}")?, // 技术缩写
            Regex::new(r"[一-龯]{2,8}[编程语言数据库]")?,
        ];
        
        // 产品名称模式（更精确的匹配）
        self.product_patterns = vec![
            Regex::new(r"[A-Za-z0-9]{2,20}")?, // 英文产品名
            Regex::new(r"[一-龯]{2,6}(软件|应用|程序|工具|系统|平台)")?, // 中文产品名，使用完整词汇
        ];
        
        Ok(())
    }
    
    /// 初始化词典
    fn initialize_dictionaries(&mut self) {
        // 常见人名
        self.person_dict = vec![
            "张三".to_string(), "李四".to_string(), "王五".to_string(),
            "赵六".to_string(), "孙七".to_string(), "周八".to_string(),
        ];
        
        // 常见地名
        self.location_dict = vec![
            "北京".to_string(), "上海".to_string(), "广州".to_string(),
            "深圳".to_string(), "杭州".to_string(), "南京".to_string(),
            "成都".to_string(), "武汉".to_string(), "西安".to_string(),
        ];
        
        // 常见机构
        self.organization_dict = vec![
            "清华大学".to_string(), "北京大学".to_string(), "中科院".to_string(),
            "阿里巴巴".to_string(), "腾讯".to_string(), "百度".to_string(),
            "华为".to_string(), "小米".to_string(), "字节跳动".to_string(),
        ];
        
        // 技术术语
        self.technology_dict = vec![
            "人工智能".to_string(), "机器学习".to_string(), "深度学习".to_string(),
            "神经网络".to_string(), "自然语言处理".to_string(), "计算机视觉".to_string(),
            "大数据".to_string(), "云计算".to_string(), "区块链".to_string(),
            "物联网".to_string(), "5G".to_string(), "边缘计算".to_string(),
            "Python".to_string(), "Java".to_string(), "JavaScript".to_string(),
            "Rust".to_string(), "Go".to_string(), "C++".to_string(),
            "MySQL".to_string(), "PostgreSQL".to_string(), "MongoDB".to_string(),
            "Redis".to_string(), "Docker".to_string(), "Kubernetes".to_string(),
        ];
    }
    
    /// 识别文本中的实体
    pub fn extract_entities(&self, text: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();

        // 先进行基于词典的识别（优先级更高）
        entities.extend(self.extract_by_dictionary(text, &self.person_dict, EntityType::Person));
        entities.extend(self.extract_by_dictionary(text, &self.location_dict, EntityType::Location));
        entities.extend(self.extract_by_dictionary(text, &self.organization_dict, EntityType::Organization));
        entities.extend(self.extract_by_dictionary(text, &self.technology_dict, EntityType::Technology));

        // 然后进行基于模式的识别
        entities.extend(self.extract_by_patterns(text, &self.person_patterns, EntityType::Person)?);
        entities.extend(self.extract_by_patterns(text, &self.location_patterns, EntityType::Location)?);
        entities.extend(self.extract_by_patterns(text, &self.organization_patterns, EntityType::Organization)?);
        entities.extend(self.extract_by_patterns(text, &self.time_patterns, EntityType::Time)?);
        entities.extend(self.extract_by_patterns(text, &self.concept_patterns, EntityType::Concept)?);
        entities.extend(self.extract_by_patterns(text, &self.technology_patterns, EntityType::Technology)?);
        entities.extend(self.extract_by_patterns(text, &self.product_patterns, EntityType::Product)?);

        // 去重和排序（词典匹配的实体会有更高的置信度）
        entities.sort_by_key(|e| e.start);
        entities = self.remove_overlapping_entities(entities);

        Ok(entities)
    }
    
    /// 基于正则模式提取实体
    fn extract_by_patterns(&self, text: &str, patterns: &[Regex], entity_type: EntityType) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        
        for pattern in patterns {
            for mat in pattern.find_iter(text) {
                let entity_text = mat.as_str().to_string();
                let confidence = self.calculate_pattern_confidence(&entity_text, &entity_type);
                
                if confidence > 0.3 { // 置信度阈值
                    entities.push(Entity {
                        text: entity_text,
                        entity_type: entity_type.clone(),
                        start: mat.start(),
                        end: mat.end(),
                        confidence,
                        properties: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(entities)
    }
    
    /// 基于词典提取实体
    fn extract_by_dictionary(&self, text: &str, dictionary: &[String], entity_type: EntityType) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for term in dictionary {
            let mut start = 0;
            while let Some(pos) = text[start..].find(term) {
                let actual_start = start + pos;
                let actual_end = actual_start + term.len();
                
                entities.push(Entity {
                    text: term.clone(),
                    entity_type: entity_type.clone(),
                    start: actual_start,
                    end: actual_end,
                    confidence: 0.9, // 词典匹配高置信度
                    properties: HashMap::new(),
                });
                
                start = actual_end;
            }
        }
        
        entities
    }
    
    /// 计算模式匹配的置信度
    fn calculate_pattern_confidence(&self, text: &str, entity_type: &EntityType) -> f64 {
        let base_confidence = match entity_type {
            EntityType::Time => 0.8, // 时间模式通常很准确
            EntityType::Person => 0.6,
            EntityType::Location => 0.7,
            EntityType::Organization => 0.7,
            EntityType::Technology => 0.5,
            EntityType::Concept => 0.5,
            EntityType::Product => 0.4,
            EntityType::Other => 0.3,
        };
        
        // 根据文本长度调整置信度
        let length_factor = match text.chars().count() {
            1 => 0.3,
            2 => 0.6,
            3..=6 => 1.0,
            7..=10 => 0.9,
            _ => 0.7,
        };
        
        base_confidence * length_factor
    }
    
    /// 移除重叠的实体，保留置信度更高的
    fn remove_overlapping_entities(&self, mut entities: Vec<Entity>) -> Vec<Entity> {
        entities.sort_by(|a, b| {
            a.start.cmp(&b.start)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        let mut result = Vec::new();
        
        for entity in entities {
            let overlaps = result.iter().any(|existing: &Entity| {
                (entity.start < existing.end && entity.end > existing.start)
            });
            
            if !overlaps {
                result.push(entity);
            }
        }
        
        result
    }
}

impl Default for ChineseNER {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            person_patterns: Vec::new(),
            location_patterns: Vec::new(),
            organization_patterns: Vec::new(),
            time_patterns: Vec::new(),
            concept_patterns: Vec::new(),
            technology_patterns: Vec::new(),
            product_patterns: Vec::new(),
            person_dict: Vec::new(),
            location_dict: Vec::new(),
            organization_dict: Vec::new(),
            technology_dict: Vec::new(),
        })
    }
}
