use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::Utc;
use log;
use serde_json::Value;
use walkdir::WalkDir;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::notebook::Notebook;
use crate::note::{Note, Attachment};
use crate::tag::Tag;
use crate::db_storage::DbStorageManager;

/// 思源笔记笔记本配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiyuanNotebookConfig {
    pub name: String,
    pub sort: Option<i32>,
    pub icon: Option<String>,
    pub closed: Option<bool>,
    #[serde(rename = "refCreateSaveBox")]
    pub ref_create_save_box: Option<String>,
    #[serde(rename = "refCreateSavePath")]
    pub ref_create_save_path: Option<String>,
    #[serde(rename = "docCreateSaveBox")]
    pub doc_create_save_box: Option<String>,
    #[serde(rename = "docCreateSavePath")]
    pub doc_create_save_path: Option<String>,
    #[serde(rename = "dailyNoteSavePath")]
    pub daily_note_save_path: Option<String>,
    #[serde(rename = "dailyNoteTemplatePath")]
    pub daily_note_template_path: Option<String>,
    #[serde(rename = "sortMode")]
    pub sort_mode: Option<i32>,
}

/// 思源笔记文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiyuanDocument {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Spec")]
    pub spec: Option<String>,
    #[serde(rename = "Type")]
    pub doc_type: String,
    #[serde(rename = "Properties")]
    pub properties: Option<HashMap<String, Value>>,
    #[serde(rename = "Children")]
    pub children: Option<Vec<SiyuanNode>>,
}

/// 思源笔记节点结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiyuanNode {
    #[serde(rename = "ID")]
    pub id: Option<String>,
    #[serde(rename = "Type")]
    pub node_type: String,
    #[serde(rename = "Data")]
    pub data: Option<String>,
    #[serde(rename = "Properties")]
    pub properties: Option<HashMap<String, Value>>,
    #[serde(rename = "Children")]
    pub children: Option<Vec<SiyuanNode>>,
    #[serde(rename = "HeadingLevel")]
    pub heading_level: Option<i32>,
    #[serde(rename = "TextMarkType")]
    pub text_mark_type: Option<String>,
    #[serde(rename = "TextMarkBlockRefID")]
    pub text_mark_block_ref_id: Option<String>,
    #[serde(rename = "TextMarkBlockRefSubtype")]
    pub text_mark_block_ref_subtype: Option<String>,
    #[serde(rename = "TextMarkTextContent")]
    pub text_mark_text_content: Option<String>,
}

/// 思源笔记导入器
pub struct SiyuanImporter {
    storage: DbStorageManager,
    siyuan_path: PathBuf,
    notebooks: Vec<Notebook>,
    notes: Vec<(Note, String)>, // (Note, notebook_id)
    tags: Vec<Tag>,
    tag_map: HashMap<String, String>, // 思源标签名称 -> 我们的标签ID
    id_pattern: Regex, // 笔记本ID格式验证
    assets_map: HashMap<String, String>, // 资源文件映射
}

impl SiyuanImporter {
    /// 创建新的思源笔记导入器
    pub fn new(storage: DbStorageManager, siyuan_path: PathBuf) -> Self {
        // 思源笔记ID格式：YYYYMMDDHHMMSS-随机字符
        let id_pattern = Regex::new(r"^[0-9]{14}-[0-9a-z]{7}$").unwrap();

        Self {
            storage,
            siyuan_path,
            notebooks: Vec::new(),
            notes: Vec::new(),
            tags: Vec::new(),
            tag_map: HashMap::new(),
            id_pattern,
            assets_map: HashMap::new(),
        }
    }

    /// 执行导入过程
    pub fn import(&mut self) -> Result<ImportStats, Box<dyn std::error::Error>> {
        log::info!("开始从思源笔记导入数据: {}", self.siyuan_path.display());

        // 1. 验证思源笔记目录
        self.validate_siyuan_directory()?;

        // 2. 导入笔记本和笔记
        self.import_notebooks_and_notes()?;

        // 3. 保存数据到数据库
        self.save_to_database()?;

        // 4. 返回导入统计信息
        let stats = ImportStats {
            notebooks_count: self.notebooks.len(),
            notes_count: self.notes.len(),
            tags_count: self.tags.len(),
        };

        log::info!("思源笔记导入完成: {} 个笔记本, {} 个笔记, {} 个标签",
            stats.notebooks_count, stats.notes_count, stats.tags_count);

        Ok(stats)
    }

    /// 验证思源笔记目录结构
    fn validate_siyuan_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 检查工作空间根目录
        if !self.siyuan_path.exists() || !self.siyuan_path.is_dir() {
            return Err(format!("无效的思源笔记目录: {} 不存在或不是目录", self.siyuan_path.display()).into());
        }

        // 检查是否有符合ID格式的笔记本目录
        let mut found_notebook = false;
        for entry in fs::read_dir(&self.siyuan_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if self.id_pattern.is_match(&dir_name) {
                    found_notebook = true;
                    break;
                }
            }
        }

        if !found_notebook {
            return Err("无效的思源笔记目录: 没有找到符合格式的笔记本目录".into());
        }

        Ok(())
    }

    /// 导入笔记本和笔记
    fn import_notebooks_and_notes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 直接在工作空间根目录中查找笔记本
        for entry in fs::read_dir(&self.siyuan_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();

                // 检查是否符合思源笔记本ID格式
                if self.id_pattern.is_match(&dir_name) {
                    self.process_notebook(&path)?;
                }
            }
        }

        Ok(())
    }

    /// 处理笔记本
    fn process_notebook(&mut self, notebook_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let notebook_id = notebook_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        log::info!("处理笔记本: {}", notebook_id);

        // 读取笔记本配置
        let config_path = notebook_path.join(".siyuan").join("conf.json");
        let notebook_config = if config_path.exists() {
            let config_content = fs::read_to_string(&config_path)?;
            serde_json::from_str::<SiyuanNotebookConfig>(&config_content)
                .unwrap_or_else(|_| SiyuanNotebookConfig {
                    name: notebook_id.clone(),
                    sort: None,
                    icon: None,
                    closed: None,
                    ref_create_save_box: None,
                    ref_create_save_path: None,
                    doc_create_save_box: None,
                    doc_create_save_path: None,
                    daily_note_save_path: None,
                    daily_note_template_path: None,
                    sort_mode: None,
                })
        } else {
            SiyuanNotebookConfig {
                name: notebook_id.clone(),
                sort: None,
                icon: None,
                closed: None,
                ref_create_save_box: None,
                ref_create_save_path: None,
                doc_create_save_box: None,
                doc_create_save_path: None,
                daily_note_save_path: None,
                daily_note_template_path: None,
                sort_mode: None,
            }
        };

        // 创建笔记本
        let notebook = Notebook::new(
            notebook_config.name.clone(),
            format!("从思源笔记导入的笔记本: {} ({})", notebook_config.name, notebook_id)
        );
        let internal_notebook_id = notebook.id.clone();

        // 添加到笔记本列表
        self.notebooks.push(notebook);

        // 处理笔记本中的资源文件
        self.process_notebook_assets(notebook_path)?;

        // 遍历笔记本中的所有.sy文件
        let mut document_count = 0;
        const MAX_DOCUMENTS_PER_NOTEBOOK: usize = 10000; // 限制每个笔记本最多处理10000个文档

        for entry in WalkDir::new(notebook_path).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "sy") {
                if document_count >= MAX_DOCUMENTS_PER_NOTEBOOK {
                    log::warn!("笔记本 {} 的文档数量超过限制 ({}), 停止处理", notebook_id, MAX_DOCUMENTS_PER_NOTEBOOK);
                    break;
                }

                self.process_siyuan_document(path, &internal_notebook_id)?;
                document_count += 1;

                // 每处理100个文档输出一次进度
                if document_count % 100 == 0 {
                    log::info!("已处理 {} 个文档...", document_count);
                }
            }
        }

        log::info!("笔记本 {} 处理完成，共处理 {} 个文档", notebook_id, document_count);

        Ok(())
    }

    /// 处理笔记本资源文件
    fn process_notebook_assets(&mut self, notebook_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let assets_dir = notebook_path.join("assets");
        if !assets_dir.exists() {
            return Ok(());
        }

        log::info!("处理笔记本资源文件: {}", assets_dir.display());

        // 遍历assets目录中的所有文件
        for entry in WalkDir::new(&assets_dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            if path.is_file() {
                let absolute_path = path.to_string_lossy().to_string();

                // 记录多种路径格式的映射
                if let Some(relative_path) = path.strip_prefix(notebook_path).ok() {
                    let relative_path_str = relative_path.to_string_lossy().to_string();

                    // 1. 完整相对路径 (assets/xxx.png)
                    self.assets_map.insert(relative_path_str.clone(), absolute_path.clone());

                    // 2. 去掉assets前缀的路径 (xxx.png)
                    if let Some(asset_relative) = relative_path_str.strip_prefix("assets/") {
                        self.assets_map.insert(asset_relative.to_string(), absolute_path.clone());
                    }

                    // 3. 文件名 (xxx.png)
                    if let Some(file_name) = path.file_name() {
                        let file_name_str = file_name.to_string_lossy().to_string();
                        self.assets_map.insert(file_name_str, absolute_path.clone());
                    }

                    log::debug!("记录资源文件: {} -> {}", relative_path_str, path.display());
                }
            }
        }

        Ok(())
    }

    /// 处理思源笔记文档(.sy文件)
    fn process_siyuan_document(&mut self, doc_path: &Path, notebook_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = doc_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        log::info!("处理思源文档: {}", file_name);

        // 检查文件大小，避免处理过大的文件
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
        if let Ok(metadata) = fs::metadata(doc_path) {
            if metadata.len() > MAX_FILE_SIZE {
                log::warn!("跳过过大的文件: {} ({}MB)", file_name, metadata.len() / 1024 / 1024);
                return Ok(());
            }
        }

        // 读取.sy文件内容
        let content = fs::read_to_string(doc_path)?;

        // 解析JSON文档
        let document: SiyuanDocument = serde_json::from_str(&content)
            .map_err(|e| format!("解析文档 {} 失败: {}", file_name, e))?;

        // 提取文档标题
        let title = self.extract_document_title(&document);

        // 转换文档内容为Markdown
        let markdown_content = self.convert_document_to_markdown(&document)?;

        // 提取标签
        let tags = self.extract_tags_from_document(&document);

        // 创建笔记
        let mut note = Note::new(title, markdown_content);

        // 添加标签
        for tag_name in tags {
            let tag_id = self.get_or_create_tag(&tag_name);
            note.add_tag(tag_id);
        }

        // 处理文档中的附件
        self.process_document_attachments(&document, &mut note)?;

        // 添加到笔记列表
        let note_id = note.id.clone();
        self.notes.push((note, notebook_id.to_string()));

        // 同时更新笔记本的note_ids字段
        if let Some(notebook) = self.notebooks.iter_mut().find(|nb| nb.id == notebook_id) {
            notebook.append_note(note_id); // 使用 append_note 保持导入顺序
            log::debug!("Added note to notebook '{}', total notes: {}", notebook.name, notebook.note_ids.len());
        } else {
            log::warn!("Could not find notebook '{}' to add note", notebook_id);
        }

        Ok(())
    }

    /// 提取文档标题
    fn extract_document_title(&self, document: &SiyuanDocument) -> String {
        // 从Properties中提取title
        if let Some(properties) = &document.properties {
            if let Some(title_value) = properties.get("title") {
                if let Some(title) = title_value.as_str() {
                    return title.to_string();
                }
            }
        }

        // 如果没有title属性，使用文档ID作为标题
        document.id.clone()
    }

    /// 将思源文档转换为Markdown格式
    fn convert_document_to_markdown(&self, document: &SiyuanDocument) -> Result<String, Box<dyn std::error::Error>> {
        let mut markdown = String::new();

        if let Some(children) = &document.children {
            for child in children {
                markdown.push_str(&self.convert_node_to_markdown(child, 0)?);
            }
        }

        Ok(markdown)
    }

    /// 将思源节点转换为Markdown格式
    fn convert_node_to_markdown(&self, node: &SiyuanNode, depth: usize) -> Result<String, Box<dyn std::error::Error>> {
        // 防止无限递归，限制最大深度
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            log::warn!("节点转换深度超过限制 ({}), 停止递归", MAX_DEPTH);
            return Ok(String::new());
        }

        let mut result = String::new();

        match node.node_type.as_str() {
            "NodeText" => {
                if let Some(data) = &node.data {
                    result.push_str(data);
                }
            },
            "NodeParagraph" => {
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
                result.push_str("\n\n");
            },
            "NodeHeading" => {
                let level = node.heading_level.unwrap_or(1);
                result.push_str(&"#".repeat(level as usize));
                result.push(' ');

                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
                result.push_str("\n\n");
            },
            "NodeList" => {
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
            },
            "NodeListItem" => {
                result.push_str(&"  ".repeat(depth));
                result.push_str("- ");

                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth + 1)?);
                    }
                }
                result.push('\n');
            },
            "NodeCodeBlock" => {
                result.push_str("```");

                // 尝试从属性中获取语言类型
                if let Some(properties) = &node.properties {
                    if let Some(lang_value) = properties.get("language") {
                        if let Some(lang) = lang_value.as_str() {
                            result.push_str(lang);
                        }
                    }
                }

                result.push('\n');

                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }

                result.push_str("\n```\n\n");
            },
            "NodeBlockquote" => {
                if let Some(children) = &node.children {
                    for child in children {
                        let child_content = self.convert_node_to_markdown(child, depth)?;
                        for line in child_content.lines() {
                            if !line.trim().is_empty() {
                                result.push_str("> ");
                                result.push_str(line);
                                result.push('\n');
                            }
                        }
                    }
                }
                result.push('\n');
            },
            "NodeTextMark" => {
                // 处理文本标记（加粗、斜体、链接等）
                if let Some(mark_type) = &node.text_mark_type {
                    match mark_type.as_str() {
                        "strong" => {
                            result.push_str("**");
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                            result.push_str("**");
                        },
                        "em" => {
                            result.push('*');
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                            result.push('*');
                        },
                        "code" => {
                            result.push('`');
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                            result.push('`');
                        },
                        "del" | "s" => {
                            result.push_str("~~");
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                            result.push_str("~~");
                        },
                        "u" => {
                            result.push_str("<u>");
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                            result.push_str("</u>");
                        },
                        "mark" => {
                            result.push_str("==");
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                            result.push_str("==");
                        },
                        "a" => {
                            // 处理链接
                            if let Some(properties) = &node.properties {
                                if let Some(href_value) = properties.get("href") {
                                    if let Some(href) = href_value.as_str() {
                                        result.push('[');
                                        if let Some(children) = &node.children {
                                            for child in children {
                                                result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                            }
                                        }
                                        result.push_str(&format!("]({})", href));
                                    }
                                } else {
                                    // 没有href属性，直接输出内容
                                    if let Some(children) = &node.children {
                                        for child in children {
                                            result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                        }
                                    }
                                }
                            }
                        },
                        "block-ref" => {
                            // 处理块引用
                            if let Some(ref_content) = &node.text_mark_text_content {
                                result.push_str(&format!("[[{}]]", ref_content));
                            } else if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                        },
                        _ => {
                            // 其他类型的文本标记，直接输出内容
                            if let Some(children) = &node.children {
                                for child in children {
                                    result.push_str(&self.convert_node_to_markdown(child, depth)?);
                                }
                            }
                        }
                    }
                } else if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
            },
            "NodeTable" => {
                // 处理表格
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
                result.push('\n');
            },
            "NodeTableHead" => {
                // 处理表格头部
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
                // 添加表格分隔线
                if let Some(children) = &node.children {
                    result.push('|');
                    for _ in children {
                        result.push_str(" --- |");
                    }
                    result.push('\n');
                }
            },
            "NodeTableRow" => {
                // 处理表格行
                result.push('|');
                if let Some(children) = &node.children {
                    for child in children {
                        result.push(' ');
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                        result.push_str(" |");
                    }
                }
                result.push('\n');
            },
            "NodeTableCell" => {
                // 处理表格单元格
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
            },
            "NodeImage" => {
                // 处理图片
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            let alt_text = properties.get("alt")
                                .and_then(|v| v.as_str())
                                .unwrap_or("image");
                            let title = properties.get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            if title.is_empty() {
                                result.push_str(&format!("![{}]({})", alt_text, src));
                            } else {
                                result.push_str(&format!("![{}]({} \"{}\")", alt_text, src, title));
                            }
                            result.push_str("\n\n");
                        }
                    }
                }
            },
            "NodeAsset" => {
                // 处理附件
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            let name = properties.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("attachment");
                            result.push_str(&format!("[{}]({})", name, src));
                            result.push_str("\n\n");
                        }
                    }
                }
            },
            "NodeMathBlock" => {
                // 处理数学公式块
                result.push_str("$$\n");
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
                result.push_str("\n$$\n\n");
            },
            "NodeInlineMath" => {
                // 处理行内数学公式
                result.push('$');
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
                result.push('$');
            },
            "NodeHorizontalRule" | "NodeThematicBreak" => {
                // 处理分隔线
                result.push_str("---\n\n");
            },
            "NodeSuperBlock" => {
                // 处理超级块
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
            },
            "NodeVideo" => {
                // 处理视频
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            result.push_str(&format!("![Video]({})", src));
                            result.push_str("\n\n");
                        }
                    }
                }
            },
            "NodeAudio" => {
                // 处理音频
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            result.push_str(&format!("![Audio]({})", src));
                            result.push_str("\n\n");
                        }
                    }
                }
            },
            "NodeIFrame" => {
                // 处理嵌入框架
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            result.push_str(&format!("[Embedded Content]({})", src));
                            result.push_str("\n\n");
                        }
                    }
                }
            },
            _ => {
                // 对于未知的节点类型，递归处理子节点
                if let Some(children) = &node.children {
                    for child in children {
                        result.push_str(&self.convert_node_to_markdown(child, depth)?);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 从思源文档中提取标签
    fn extract_tags_from_document(&self, document: &SiyuanDocument) -> Vec<String> {
        let mut tags = Vec::new();

        // 从文档属性中提取标签
        if let Some(properties) = &document.properties {
            if let Some(tag_value) = properties.get("tag") {
                if let Some(tag_str) = tag_value.as_str() {
                    // 思源笔记的标签可能以逗号分隔
                    for tag in tag_str.split(',') {
                        let tag = tag.trim().to_string();
                        if !tag.is_empty() {
                            tags.push(tag);
                        }
                    }
                }
            }
        }

        // 从文档内容中提取标签（递归搜索节点）
        if let Some(children) = &document.children {
            for child in children {
                self.extract_tags_from_node(child, &mut tags);
            }
        }

        tags
    }

    /// 从节点中递归提取标签
    fn extract_tags_from_node(&self, node: &SiyuanNode, tags: &mut Vec<String>) {
        self.extract_tags_from_node_with_depth(node, tags, 0);
    }

    /// 从节点中递归提取标签（带深度限制）
    fn extract_tags_from_node_with_depth(&self, node: &SiyuanNode, tags: &mut Vec<String>, depth: usize) {
        // 防止无限递归
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            log::warn!("标签提取深度超过限制 ({}), 停止递归", MAX_DEPTH);
            return;
        }
        // 检查节点属性中的标签
        if let Some(properties) = &node.properties {
            if let Some(tag_value) = properties.get("tag") {
                if let Some(tag_str) = tag_value.as_str() {
                    for tag in tag_str.split(',') {
                        let tag = tag.trim().to_string();
                        if !tag.is_empty() && !tags.contains(&tag) {
                            tags.push(tag);
                        }
                    }
                }
            }
        }

        // 从文本内容中提取标签（#标签名 格式）
        if node.node_type == "NodeText" {
            if let Some(data) = &node.data {
                self.extract_hashtags_from_text(data, tags);
            }
        }

        // 递归处理子节点
        if let Some(children) = &node.children {
            for child in children {
                self.extract_tags_from_node_with_depth(child, tags, depth + 1);
            }
        }
    }

    /// 从文本中提取井号标签
    fn extract_hashtags_from_text(&self, text: &str, tags: &mut Vec<String>) {
        // 使用正则表达式匹配 #标签名 格式
        let tag_regex = Regex::new(r"#([^\s#]+)").unwrap();

        for cap in tag_regex.captures_iter(text) {
            if let Some(tag_match) = cap.get(1) {
                let tag = tag_match.as_str().to_string();
                if !tag.is_empty() && !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }
    }

    /// 处理文档中的附件
    fn process_document_attachments(&self, document: &SiyuanDocument, note: &mut Note) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(children) = &document.children {
            for child in children {
                self.process_node_attachments(child, note)?;
            }
        }
        Ok(())
    }

    /// 递归处理节点中的附件
    fn process_node_attachments(&self, node: &SiyuanNode, note: &mut Note) -> Result<(), Box<dyn std::error::Error>> {
        self.process_node_attachments_with_depth(node, note, 0)
    }

    /// 递归处理节点中的附件（带深度限制）
    fn process_node_attachments_with_depth(&self, node: &SiyuanNode, note: &mut Note, depth: usize) -> Result<(), Box<dyn std::error::Error>> {
        // 防止无限递归
        const MAX_DEPTH: usize = 100;
        if depth > MAX_DEPTH {
            log::warn!("附件处理深度超过限制 ({}), 停止递归", MAX_DEPTH);
            return Ok(());
        }

        // 根据节点类型处理不同的资源文件
        match node.node_type.as_str() {
            "NodeImage" | "NodeAsset" | "NodeVideo" | "NodeAudio" => {
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            self.add_attachment_from_src(src, note)?;
                        }
                    }
                }
            },
            "NodeTextMark" => {
                // 检查是否是链接到资源文件
                if let Some(properties) = &node.properties {
                    if let Some(href_value) = properties.get("href") {
                        if let Some(href) = href_value.as_str() {
                            // 检查是否是本地资源文件
                            if href.starts_with("assets/") || href.contains("/assets/") {
                                self.add_attachment_from_src(href, note)?;
                            }
                        }
                    }
                }
            },
            _ => {
                // 对于其他节点类型，检查是否有src属性
                if let Some(properties) = &node.properties {
                    if let Some(src_value) = properties.get("src") {
                        if let Some(src) = src_value.as_str() {
                            // 检查是否是资源文件路径
                            if src.starts_with("assets/") || src.contains("/assets/") {
                                self.add_attachment_from_src(src, note)?;
                            }
                        }
                    }
                }
            }
        }

        // 递归处理子节点
        if let Some(children) = &node.children {
            for child in children {
                self.process_node_attachments_with_depth(child, note, depth + 1)?;
            }
        }

        Ok(())
    }

    /// 从资源路径添加附件
    fn add_attachment_from_src(&self, src: &str, note: &mut Note) -> Result<(), Box<dyn std::error::Error>> {

        // 尝试多种路径格式匹配
        let possible_paths = vec![
            src.to_string(),
            format!("assets/{}", src),
            src.strip_prefix("assets/").unwrap_or(src).to_string(),
        ];

        log::debug!("尝试的路径变体: {:?}", possible_paths);
        log::debug!("当前资源映射数量: {}", self.assets_map.len());

        for path_variant in possible_paths {
            if let Some(asset_path) = self.assets_map.get(&path_variant) {
                // 检查是否已经添加过这个附件
                let file_name = Path::new(src).file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // 避免重复添加相同的附件
                if note.attachments.iter().any(|att| att.file_path == *asset_path) {
                    log::debug!("附件已存在，跳过: {}", asset_path);
                    return Ok(());
                }

                let attachment = Attachment {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: file_name,
                    file_path: asset_path.clone(),
                    file_type: self.get_file_type(src),
                    created_at: Utc::now(),
                };

                note.attachments.push(attachment);
                log::info!("成功添加附件: {} -> {}", src, asset_path);
                return Ok(());
            } else {
                log::debug!("路径变体未找到: {}", path_variant);
            }
        }

        // 如果没有找到对应的资源文件，记录警告
        log::warn!("未找到资源文件: {} (在 {} 个资源中搜索)", src, self.assets_map.len());

        // 资源文件未找到，但不输出调试信息以减少日志噪音

        Ok(())
    }

    /// 根据文件扩展名确定文件类型
    fn get_file_type(&self, file_path: &str) -> String {
        if let Some(extension) = Path::new(file_path).extension() {
            match extension.to_string_lossy().to_lowercase().as_str() {
                // 图片文件
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif" => "image".to_string(),

                // 文档文件
                "pdf" => "pdf".to_string(),
                "doc" | "docx" | "odt" | "rtf" => "document".to_string(),
                "xls" | "xlsx" | "ods" | "csv" => "spreadsheet".to_string(),
                "ppt" | "pptx" | "odp" => "presentation".to_string(),
                "txt" | "md" | "markdown" | "rst" => "text".to_string(),

                // 音频文件
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => "audio".to_string(),

                // 视频文件
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => "video".to_string(),

                // 压缩文件
                "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "archive".to_string(),

                // 代码文件
                "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "rs" | "go" | "php" | "rb" | "swift" => "code".to_string(),
                "html" | "htm" | "css" | "xml" | "json" | "yaml" | "yml" => "markup".to_string(),

                // 其他常见文件
                "exe" | "msi" | "dmg" | "pkg" | "deb" | "rpm" => "executable".to_string(),
                "font" | "ttf" | "otf" | "woff" | "woff2" => "font".to_string(),

                _ => "file".to_string(),
            }
        } else {
            "file".to_string()
        }
    }

    /// 获取或创建标签
    fn get_or_create_tag(&mut self, tag_name: &str) -> String {
        if let Some(tag_id) = self.tag_map.get(tag_name) {
            return tag_id.clone();
        }

        // 创建新标签
        let tag = Tag::new(
            tag_name.to_string(),
            // 生成随机颜色
            self.generate_tag_color()
        );

        let tag_id = tag.id.clone();
        self.tag_map.insert(tag_name.to_string(), tag_id.clone());
        self.tags.push(tag);

        tag_id
    }

    /// 生成标签颜色
    fn generate_tag_color(&self) -> String {
        // 预定义的一些好看的颜色
        let colors = [
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7",
            "#DDA0DD", "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E9",
            "#F8C471", "#82E0AA", "#F1948A", "#85C1E9", "#D7BDE2"
        ];

        let index = self.tags.len() % colors.len();
        colors[index].to_string()
    }

    /// 保存数据到数据库
    fn save_to_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("开始保存导入数据到数据库...");

        // 保存笔记本 (必须先保存，因为笔记有外键约束)
        log::info!("保存 {} 个笔记本", self.notebooks.len());
        for notebook in &self.notebooks {
            log::debug!("保存笔记本: {} (包含 {} 个笔记)", notebook.name, notebook.note_ids.len());
            match self.storage.save_notebook(notebook) {
                Ok(_) => {
                    log::debug!("✅ 笔记本 '{}' 保存成功", notebook.name);
                }
                Err(e) => {
                    log::error!("❌ 笔记本 '{}' 保存失败: {}", notebook.name, e);
                    return Err(format!("保存笔记本失败: {}", e).into());
                }
            }
        }

        // 保存标签
        log::info!("保存 {} 个标签", self.tags.len());
        for tag in &self.tags {
            match self.storage.save_tag(tag) {
                Ok(_) => {
                    log::debug!("✅ 标签 '{}' 保存成功", tag.name);
                }
                Err(e) => {
                    log::warn!("⚠️ 标签 '{}' 保存失败: {}", tag.name, e);
                    // 标签保存失败不是致命错误，继续处理
                }
            }
        }

        // 保存笔记 (在笔记本保存成功后)
        log::info!("保存 {} 个笔记", self.notes.len());
        let mut successful_notes = 0;
        let mut failed_notes = 0;
        let mut failed_note_ids = Vec::new();

        for (note, notebook_id) in &self.notes {
            log::debug!("保存笔记: {} 到笔记本: {}", note.title, notebook_id);
            match self.storage.save_note(note, notebook_id) {
                Ok(_) => {
                    log::debug!("✅ 笔记 '{}' 保存成功", note.title);
                    successful_notes += 1;
                }
                Err(e) => {
                    log::error!("❌ 笔记 '{}' 保存失败: {}", note.title, e);
                    failed_notes += 1;
                    failed_note_ids.push(note.id.clone());

                    // 对于个别笔记保存失败，记录错误但继续处理其他笔记
                    if failed_notes > self.notes.len() / 2 {
                        // 如果超过一半的笔记保存失败，则认为是系统性问题
                        return Err(format!("笔记保存失败率过高，已失败 {} 个笔记", failed_notes).into());
                    }
                }
            }
        }

        log::info!("导入数据保存完成: {} 个笔记成功, {} 个笔记失败", successful_notes, failed_notes);

        if failed_notes > 0 {
            log::warn!("部分笔记保存失败，但导入过程继续完成");
        }

        Ok(())
    }
}

/// 导入统计信息
#[derive(Debug, Clone)]
pub struct ImportStats {
    pub notebooks_count: usize,
    pub notes_count: usize,
    pub tags_count: usize,
}
