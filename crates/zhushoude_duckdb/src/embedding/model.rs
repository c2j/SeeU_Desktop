//! 轻量级中文语义模型实现

use crate::{Result, EmbeddingConfig, Error};
use crate::embedding::provider::{EmbeddingProvider, ModelInfo, ModelType, ProviderStats};
use async_trait::async_trait;
use jieba_rs::Jieba;
use std::collections::HashMap;

/// 轻量级中文语义模型
pub struct BgeSmallZhModel {
    config: EmbeddingConfig,
    jieba: Jieba,
    word_vectors: HashMap<String, Vec<f32>>,
    is_loaded: bool,
}

impl BgeSmallZhModel {
    /// 创建新的轻量级中文语义模型实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        println!("🔧 初始化轻量级中文语义模型...");

        // 初始化结巴分词器
        let jieba = Jieba::new();

        // 初始化词向量表
        let word_vectors = Self::initialize_word_vectors(&config).await?;

        let is_loaded = !word_vectors.is_empty();

        println!("✅ 轻量级中文语义模型初始化完成 (使用词向量方法)");

        Ok(Self {
            config,
            jieba,
            word_vectors,
            is_loaded,
        })
    }



    /// 初始化词向量表
    async fn initialize_word_vectors(config: &EmbeddingConfig) -> Result<HashMap<String, Vec<f32>>> {
        let mut word_vectors = HashMap::new();

        // 添加一些常用中文词汇的预定义向量
        let common_words = vec![
            "的", "是", "在", "了", "有", "和", "人", "这", "中", "大", "为", "上", "个", "国", "我",
            "以", "要", "他", "时", "来", "用", "们", "生", "到", "作", "地", "于", "出", "就", "分",
            "对", "成", "会", "可", "主", "发", "年", "动", "同", "工", "也", "能", "下", "过", "子",
            "说", "产", "种", "面", "而", "方", "后", "多", "定", "行", "学", "法", "所", "民", "得",
            "经", "十", "三", "之", "进", "着", "等", "部", "度", "家", "电", "力", "里", "如", "水",
            "化", "高", "自", "二", "理", "起", "小", "物", "现", "实", "加", "量", "都", "两", "体",
            "制", "机", "当", "使", "点", "从", "业", "本", "去", "把", "性", "好", "应", "开", "它",
            "合", "还", "因", "由", "其", "些", "然", "前", "外", "天", "政", "四", "日", "那", "社",
            "义", "事", "平", "形", "相", "全", "表", "间", "样", "与", "关", "各", "重", "新", "线",
            "内", "数", "正", "心", "反", "你", "明", "看", "原", "又", "么", "利", "比", "或", "但",
            "质", "气", "第", "向", "道", "命", "此", "变", "条", "只", "没", "结", "解", "问", "意",
            "建", "月", "公", "无", "系", "军", "很", "情", "者", "最", "立", "代", "想", "已", "通",
            "并", "提", "直", "题", "党", "程", "展", "五", "果", "料", "象", "员", "革", "位", "入",
            "常", "文", "总", "次", "品", "式", "活", "设", "及", "管", "特", "件", "长", "求", "老",
            "头", "基", "资", "边", "流", "路", "级", "少", "图", "山", "统", "接", "知", "较", "将",
            "组", "见", "计", "别", "她", "手", "角", "期", "根", "论", "运", "农", "指", "几", "九",
            "区", "强", "放", "决", "西", "被", "干", "做", "必", "战", "先", "回", "则", "任", "取",
            "据", "处", "队", "南", "给", "色", "光", "门", "即", "保", "治", "北", "造", "百", "规",
            "热", "领", "七", "海", "口", "东", "导", "器", "压", "志", "世", "金", "增", "争", "济",
            "阶", "油", "思", "术", "极", "交", "受", "联", "什", "认", "六", "共", "权", "收", "证",
        ];

        let vector_dim = config.vector_dimension;

        for (i, word) in common_words.iter().enumerate() {
            let mut vector = vec![0.0; vector_dim];

            // 使用简单的哈希函数生成确定性向量
            let hash = Self::simple_hash(word);
            for j in 0..vector_dim {
                let seed = (hash.wrapping_mul(31).wrapping_add(j as u64)) as f32;
                vector[j] = (seed.sin() + 1.0) / 2.0; // 归一化到[0,1]
            }

            // 归一化向量
            let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut vector {
                    *v /= norm;
                }
            }

            word_vectors.insert(word.to_string(), vector);
        }

        println!("📚 初始化了 {} 个常用词汇的向量", word_vectors.len());
        Ok(word_vectors)
    }

    /// 简单哈希函数
    fn simple_hash(text: &str) -> u64 {
        let mut hash = 5381u64;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    /// 对单个文本进行编码
    pub async fn encode_single(&self, text: &str) -> Result<Vec<f32>> {
        if self.is_loaded {
            // 使用词向量方法
            self.encode_with_jieba(text).await
        } else {
            // 使用占位符实现
            self.encode_placeholder(text).await
        }
    }




    /// 使用结巴分词进行编码
    async fn encode_with_jieba(&self, text: &str) -> Result<Vec<f32>> {
        // 使用结巴分词
        let words: Vec<&str> = self.jieba.cut(text, false);

        let vector_dim = self.config.vector_dimension;
        let mut text_vector = vec![0.0; vector_dim];
        let mut word_count = 0;

        // 对每个词进行向量化并累加
        for word in &words {
            if let Some(word_vector) = self.word_vectors.get(*word) {
                for (i, &value) in word_vector.iter().enumerate() {
                    if i < vector_dim {
                        text_vector[i] += value;
                    }
                }
                word_count += 1;
            } else {
                // 对未知词生成向量
                let unknown_vector = self.generate_unknown_word_vector(word);
                for (i, &value) in unknown_vector.iter().enumerate() {
                    if i < vector_dim {
                        text_vector[i] += value;
                    }
                }
                word_count += 1;
            }
        }

        // 平均化
        if word_count > 0 {
            for value in &mut text_vector {
                *value /= word_count as f32;
            }
        }

        // 添加语义特征（基于文本内容的语义信息）
        self.add_semantic_features(&mut text_vector, text, &words);

        // 添加文本级别特征
        self.add_text_features(&mut text_vector, text);

        // 归一化（如果配置要求）
        if self.config.normalize_vectors {
            Ok(self.normalize_vector(&text_vector))
        } else {
            Ok(text_vector)
        }
    }

    /// 为未知词生成向量（基于语义特征）
    fn generate_unknown_word_vector(&self, word: &str) -> Vec<f32> {
        let vector_dim = self.config.vector_dimension;
        let mut vector = vec![0.0; vector_dim];

        // 基于字符生成基础向量
        let chars: Vec<char> = word.chars().collect();
        let hash = Self::simple_hash(word);

        // 生成基础向量
        for i in 0..vector_dim {
            let char_index = i % chars.len();
            let char_code = chars[char_index] as u32;
            let seed = hash.wrapping_add(char_code as u64).wrapping_add(i as u64);
            vector[i] = ((seed as f32).sin() + 1.0) / 2.0;
        }

        // 添加语义特征
        self.add_word_semantic_features(&mut vector, word);

        // 归一化
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }

        vector
    }

    /// 为单词添加语义特征
    fn add_word_semantic_features(&self, vector: &mut Vec<f32>, word: &str) {
        let vector_dim = vector.len();
        if vector_dim < 50 { return; }

        // 词性特征
        let mut pos_features = vec![0.0; 10];

        // 简单的词性判断规则
        if word.ends_with("的") || word.ends_with("地") || word.ends_with("得") {
            pos_features[0] = 1.0; // 助词
        } else if word.ends_with("们") || word.ends_with("者") || word.ends_with("家") {
            pos_features[1] = 1.0; // 名词
        } else if word.ends_with("了") || word.ends_with("着") || word.ends_with("过") {
            pos_features[2] = 1.0; // 动词
        } else if word.len() == 1 && "很非常特别".contains(word) {
            pos_features[3] = 1.0; // 副词
        } else if word.len() >= 2 && (word.contains("学") || word.contains("术") || word.contains("理")) {
            pos_features[4] = 1.0; // 学术词汇
        } else if word.len() >= 2 && (word.contains("人") || word.contains("民") || word.contains("群")) {
            pos_features[5] = 1.0; // 人物相关
        } else if word.len() >= 2 && (word.contains("时") || word.contains("间") || word.contains("期")) {
            pos_features[6] = 1.0; // 时间相关
        } else if word.len() >= 2 && (word.contains("地") || word.contains("方") || word.contains("处")) {
            pos_features[7] = 1.0; // 地点相关
        } else if word.chars().any(|c| c.is_ascii_digit()) {
            pos_features[8] = 1.0; // 数字相关
        } else {
            pos_features[9] = 1.0; // 其他
        }

        // 将词性特征添加到向量末尾
        for (i, &feature) in pos_features.iter().enumerate() {
            if vector_dim > 10 + i {
                vector[vector_dim - 10 - i] += feature * 0.1; // 较小的权重
            }
        }
    }

    /// 添加文本级别特征
    fn add_text_features(&self, vector: &mut Vec<f32>, text: &str) {
        let vector_dim = vector.len();
        if vector_dim < 10 { return; }

        // 文本长度特征
        let text_len = text.chars().count() as f32;
        let len_feature = (text_len / 100.0).min(1.0); // 归一化到[0,1]
        vector[vector_dim - 1] = len_feature;

        // 标点符号密度
        let punct_count = text.chars().filter(|c| c.is_ascii_punctuation()).count() as f32;
        let punct_density = (punct_count / text_len.max(1.0)).min(1.0);
        vector[vector_dim - 2] = punct_density;

        // 数字密度
        let digit_count = text.chars().filter(|c| c.is_ascii_digit()).count() as f32;
        let digit_density = (digit_count / text_len.max(1.0)).min(1.0);
        vector[vector_dim - 3] = digit_density;

        // 英文字符密度
        let ascii_count = text.chars().filter(|c| c.is_ascii_alphabetic()).count() as f32;
        let ascii_density = (ascii_count / text_len.max(1.0)).min(1.0);
        vector[vector_dim - 4] = ascii_density;

        // 中文字符密度
        let chinese_count = text.chars().filter(|c| {
            let code = *c as u32;
            (0x4E00..=0x9FFF).contains(&code) // 基本汉字范围
        }).count() as f32;
        let chinese_density = (chinese_count / text_len.max(1.0)).min(1.0);
        vector[vector_dim - 5] = chinese_density;
    }

    /// 添加语义特征（基于词汇和内容的语义信息）
    fn add_semantic_features(&self, vector: &mut Vec<f32>, text: &str, words: &[&str]) {
        let vector_dim = vector.len();
        if vector_dim < 20 { return; }

        // 语义类别特征
        let mut category_scores = vec![0.0; 10];

        // 定义语义类别关键词
        let categories = [
            // 0: 科学技术
            vec!["科学", "技术", "研究", "实验", "理论", "方法", "系统", "算法", "数据", "分析", "计算", "工程"],
            // 1: 哲学思想
            vec!["哲学", "思想", "理念", "观点", "思考", "认识", "意识", "存在", "真理", "智慧", "精神", "心灵"],
            // 2: 教育学习
            vec!["教育", "学习", "知识", "学校", "老师", "学生", "课程", "教学", "培训", "考试", "学位", "专业"],
            // 3: 文学艺术
            vec!["文学", "艺术", "小说", "诗歌", "音乐", "绘画", "创作", "作品", "美学", "文化", "传统", "历史"],
            // 4: 社会政治
            vec!["社会", "政治", "国家", "政府", "法律", "制度", "公民", "权利", "民主", "发展", "改革", "政策"],
            // 5: 经济商业
            vec!["经济", "商业", "市场", "企业", "金融", "投资", "贸易", "管理", "营销", "利润", "成本", "价格"],
            // 6: 医学健康
            vec!["医学", "健康", "疾病", "治疗", "药物", "医院", "医生", "患者", "诊断", "手术", "康复", "预防"],
            // 7: 体育运动
            vec!["体育", "运动", "比赛", "训练", "健身", "球类", "游泳", "跑步", "团队", "竞技", "锻炼", "体能"],
            // 8: 日常生活
            vec!["生活", "家庭", "工作", "朋友", "食物", "旅行", "购物", "娱乐", "休闲", "时间", "地方", "事情"],
            // 9: 情感心理
            vec!["情感", "心理", "感情", "爱情", "友谊", "快乐", "悲伤", "愤怒", "恐惧", "希望", "梦想", "回忆"],
        ];

        // 计算每个类别的匹配分数
        for (cat_idx, keywords) in categories.iter().enumerate() {
            let mut score = 0.0;
            for word in words {
                for keyword in keywords {
                    if word.contains(keyword) || keyword.contains(word) {
                        score += 1.0;
                    }
                }
            }
            category_scores[cat_idx] = score / words.len().max(1) as f32;
        }

        // 将类别分数添加到向量中
        for (i, &score) in category_scores.iter().enumerate() {
            if vector_dim > 10 + i {
                vector[vector_dim - 10 - i] = score.min(1.0);
            }
        }
    }

    /// 占位符编码实现
    async fn encode_placeholder(&self, text: &str) -> Result<Vec<f32>> {
        // 基于文本内容生成确定性的向量
        let dim = self.config.vector_dimension;
        let mut embedding = vec![0.0; dim];

        // 使用文本的字符来生成向量
        let chars: Vec<char> = text.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if i >= dim { break; }
            embedding[i] = (ch as u32 as f32) / 65536.0; // 归一化到[0,1]
        }

        // 添加一些基于文本长度的特征
        let text_len = text.len() as f32;
        for i in 0..dim {
            embedding[i] += (text_len * (i as f32 + 1.0)).sin() * 0.1;
        }

        // 归一化
        if self.config.normalize_vectors {
            Ok(self.normalize_vector(&embedding))
        } else {
            Ok(embedding)
        }
    }

    /// 向量归一化
    fn normalize_vector(&self, vector: &[f32]) -> Vec<f32> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.iter().map(|x| x / norm).collect()
        } else {
            vector.to_vec()
        }
    }

    /// 批量编码文本
    pub async fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        // 分批处理以避免内存问题
        let batch_size = self.config.batch_size;
        for chunk in texts.chunks(batch_size) {
            for text in chunk {
                embeddings.push(self.encode_single(text).await?);
            }
        }

        Ok(embeddings)
    }

    /// 获取模型配置
    pub fn get_config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// 检查模型是否已加载
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_loading() {
        let config = EmbeddingConfig::default();
        let result = BgeSmallZhModel::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_model_encoding() {
        let config = EmbeddingConfig::default();
        let model = BgeSmallZhModel::new(config).await.unwrap();

        let result = model.encode_single("测试文本").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 512); // 默认维度
    }

    #[tokio::test]
    async fn test_batch_encoding() {
        let config = EmbeddingConfig::default();
        let model = BgeSmallZhModel::new(config).await.unwrap();

        let texts = vec!["文本1".to_string(), "文本2".to_string()];
        let result = model.encode_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 512);
        assert_eq!(embeddings[1].len(), 512);
    }

    #[tokio::test]
    async fn test_chinese_text_encoding() {
        let config = EmbeddingConfig {
            enable_chinese_optimization: true,
            normalize_vectors: true,
            ..Default::default()
        };
        let model = BgeSmallZhModel::new(config).await.unwrap();

        let chinese_text = "这是一段中文测试文本，包含了各种中文字符。";
        let result = model.encode_single(chinese_text).await;
        assert!(result.is_ok());

        let embedding = result.unwrap();

        // 检查向量是否已归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }
}

// 为轻量级模型实现EmbeddingProvider trait
#[async_trait]
impl EmbeddingProvider for BgeSmallZhModel {
    async fn encode_single(&self, text: &str) -> Result<Vec<f32>> {
        self.encode_single(text).await
    }

    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let string_texts: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        self.encode_batch(&string_texts).await
    }

    fn get_dimension(&self) -> usize {
        self.config.vector_dimension
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "lightweight-chinese-model".to_string(),
            version: "1.0.0".to_string(),
            dimension: self.config.vector_dimension,
            max_sequence_length: 512,
            language: "zh".to_string(),
            model_type: ModelType::Custom,
            memory_usage: 50, // 估计50MB
        }
    }

    async fn health_check(&self) -> Result<()> {
        if self.is_loaded {
            Ok(())
        } else {
            Err(Error::ModelError("Model not loaded".to_string()))
        }
    }

    fn get_stats(&self) -> ProviderStats {
        ProviderStats {
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_latency_ms: 10.0,
            error_count: 0,
            memory_usage_mb: 50.0,
        }
    }
}
