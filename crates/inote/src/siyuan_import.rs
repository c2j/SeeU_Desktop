use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::Utc;
use log;
use serde_json::Value;
use walkdir::WalkDir;

use crate::notebook::Notebook;
use crate::note::Note;
use crate::tag::Tag;
use crate::db_storage::DbStorageManager;

/// 思源笔记导入器
pub struct SiyuanImporter {
    storage: DbStorageManager,
    siyuan_path: PathBuf,
    notebooks: Vec<Notebook>,
    notes: Vec<(Note, String)>, // (Note, notebook_id)
    tags: Vec<Tag>,
    tag_map: HashMap<String, String>, // 思源标签名称 -> 我们的标签ID
}

impl SiyuanImporter {
    /// 创建新的思源笔记导入器
    pub fn new(storage: DbStorageManager, siyuan_path: PathBuf) -> Self {
        Self {
            storage,
            siyuan_path,
            notebooks: Vec::new(),
            notes: Vec::new(),
            tags: Vec::new(),
            tag_map: HashMap::new(),
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
        // 检查思源笔记的data目录
        let data_dir = self.siyuan_path.join("data");
        if !data_dir.exists() || !data_dir.is_dir() {
            return Err(format!("无效的思源笔记目录: 找不到data目录 {}", data_dir.display()).into());
        }

        // 检查思源笔记的工作空间目录
        let workspace_dirs: Vec<_> = fs::read_dir(&data_dir)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .collect();

        if workspace_dirs.is_empty() {
            return Err("无效的思源笔记目录: data目录中没有工作空间".into());
        }

        Ok(())
    }

    /// 导入笔记本和笔记
    fn import_notebooks_and_notes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let data_dir = self.siyuan_path.join("data");

        // 遍历工作空间目录
        for workspace_entry in fs::read_dir(&data_dir)? {
            let workspace_entry = workspace_entry?;
            let workspace_path = workspace_entry.path();

            if workspace_path.is_dir() {
                // 处理工作空间中的笔记本
                self.process_workspace(&workspace_path)?;
            }
        }

        Ok(())
    }

    /// 处理工作空间
    fn process_workspace(&mut self, workspace_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("处理工作空间: {}", workspace_path.display());

        // 遍历工作空间中的笔记本目录
        for entry in fs::read_dir(workspace_path)? {
            let entry = entry?;
            let path = entry.path();

            // 笔记本通常是目录，并且不以.开头
            if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                self.process_notebook(&path)?;
            }
        }

        Ok(())
    }

    /// 处理笔记本
    fn process_notebook(&mut self, notebook_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let notebook_name = notebook_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        log::info!("处理笔记本: {}", notebook_name);

        // 创建笔记本
        let notebook = Notebook::new(
            notebook_name.clone(),
            format!("从思源笔记导入的笔记本: {}", notebook_name)
        );
        let notebook_id = notebook.id.clone();

        // 添加到笔记本列表
        self.notebooks.push(notebook);

        // 遍历笔记本中的所有Markdown文件
        for entry in WalkDir::new(notebook_path).into_iter().filter_map(Result::ok) {
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                self.process_note(path, &notebook_id)?;
            }
        }

        Ok(())
    }

    /// 处理笔记
    fn process_note(&mut self, note_path: &Path, notebook_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = note_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        log::info!("处理笔记: {}", file_name);

        // 读取笔记内容
        let content = fs::read_to_string(note_path)?;

        // 解析笔记标题和内容
        let (title, content) = self.parse_note_content(&content, &file_name);

        // 解析标签
        let tags = self.extract_tags(&content);

        // 创建笔记
        let mut note = Note::new(title, content);

        // 添加标签
        for tag_name in tags {
            let tag_id = self.get_or_create_tag(&tag_name);
            note.add_tag(tag_id);
        }

        // 添加到笔记列表
        self.notes.push((note, notebook_id.to_string()));

        Ok(())
    }

    /// 解析笔记内容，提取标题和正文
    fn parse_note_content(&self, content: &str, default_title: &str) -> (String, String) {
        // 尝试从内容的第一行提取标题
        let lines: Vec<&str> = content.lines().collect();

        if !lines.is_empty() {
            let first_line = lines[0].trim();

            // 如果第一行是Markdown标题
            if first_line.starts_with("# ") {
                let title = first_line[2..].trim().to_string();
                let content = lines[1..].join("\n");
                return (title, content);
            }
        }

        // 如果没有找到标题，使用文件名作为标题
        let title = if default_title.ends_with(".md") {
            default_title[..default_title.len() - 3].to_string()
        } else {
            default_title.to_string()
        };

        (title, content.to_string())
    }

    /// 从内容中提取标签
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();

        // 在思源笔记中，标签通常使用 #标签名 格式
        for line in content.lines() {
            let mut in_tag = false;
            let mut tag_start = 0;

            for (i, c) in line.char_indices() {
                if c == '#' && (i == 0 || line.chars().nth(i - 1) == Some(' ')) {
                    in_tag = true;
                    tag_start = i + 1;
                } else if in_tag && (c == ' ' || c == '#') {
                    if i > tag_start {
                        let tag = line[tag_start..i].trim().to_string();
                        if !tag.is_empty() {
                            tags.push(tag);
                        }
                    }
                    in_tag = false;
                }
            }

            // 处理行尾的标签
            if in_tag && tag_start < line.len() {
                let tag = line[tag_start..].trim().to_string();
                if !tag.is_empty() {
                    tags.push(tag);
                }
            }
        }

        tags
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
            format!("#{:06x}", rand::random::<u32>() % 0xFFFFFF)
        );

        let tag_id = tag.id.clone();
        self.tag_map.insert(tag_name.to_string(), tag_id.clone());
        self.tags.push(tag);

        tag_id
    }

    /// 保存数据到数据库
    fn save_to_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 保存笔记本
        for notebook in &self.notebooks {
            self.storage.save_notebook(notebook)?;
        }

        // 保存标签
        for tag in &self.tags {
            self.storage.save_tag(tag)?;
        }

        // 保存笔记
        for (note, notebook_id) in &self.notes {
            self.storage.save_note(note, notebook_id)?;
        }

        Ok(())
    }
}

/// 导入统计信息
#[derive(Clone)]
pub struct ImportStats {
    pub notebooks_count: usize,
    pub notes_count: usize,
    pub tags_count: usize,
}
