/// 思源笔记导入功能示例
/// 
/// 这个示例展示了如何使用优化后的思源笔记导入功能
/// 
/// 使用方法:
/// 1. 确保你有一个思源笔记的工作空间目录
/// 2. 运行这个示例，传入工作空间路径
/// 3. 查看导入结果

use std::path::PathBuf;
use std::env;
use regex::Regex;
use serde_json;
use walkdir::WalkDir;

// 注意：这个示例需要实际的数据库连接，所以这里只是展示API的使用方式
fn main() {
    // 初始化日志
    env_logger::init();

    // 从命令行参数获取思源笔记工作空间路径
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("使用方法: {} <思源笔记工作空间路径>", args[0]);
        eprintln!("示例: {} ~/SiYuan", args[0]);
        std::process::exit(1);
    }

    let siyuan_path = PathBuf::from(&args[1]);
    
    println!("🚀 开始导入思源笔记");
    println!("📁 工作空间路径: {}", siyuan_path.display());

    // 验证路径
    if !siyuan_path.exists() {
        eprintln!("❌ 错误: 路径不存在: {}", siyuan_path.display());
        std::process::exit(1);
    }

    if !siyuan_path.is_dir() {
        eprintln!("❌ 错误: 路径不是目录: {}", siyuan_path.display());
        std::process::exit(1);
    }

    // 这里应该创建实际的数据库连接和导入器
    // 由于这只是一个示例，我们模拟导入过程
    
    println!("✅ 路径验证通过");
    println!("🔍 扫描思源笔记本...");
    
    // 模拟扫描笔记本
    match scan_notebooks(&siyuan_path) {
        Ok(notebooks) => {
            println!("📚 发现 {} 个笔记本:", notebooks.len());
            for (i, notebook) in notebooks.iter().enumerate() {
                println!("  {}. {} ({})", i + 1, notebook.name, notebook.id);
            }
        },
        Err(e) => {
            eprintln!("❌ 扫描笔记本失败: {}", e);
            std::process::exit(1);
        }
    }

    println!("🎉 导入完成！");
    println!("💡 提示: 在实际应用中，请使用 SiyuanImporter 类进行完整的导入操作");
}

/// 模拟的笔记本结构
#[derive(Debug)]
struct MockNotebook {
    id: String,
    name: String,
    documents_count: usize,
    assets_count: usize,
}

/// 扫描思源笔记本（模拟实现）
fn scan_notebooks(workspace_path: &PathBuf) -> Result<Vec<MockNotebook>, Box<dyn std::error::Error>> {
    use std::fs;
    use regex::Regex;

    let mut notebooks = Vec::new();
    let id_pattern = Regex::new(r"^[0-9]{14}-[0-9a-z]{7}$")?;

    for entry in fs::read_dir(workspace_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
            
            if id_pattern.is_match(&dir_name) {
                // 读取笔记本配置
                let config_path = path.join(".siyuan").join("conf.json");
                let name = if config_path.exists() {
                    match fs::read_to_string(&config_path) {
                        Ok(content) => {
                            match serde_json::from_str::<serde_json::Value>(&content) {
                                Ok(config) => {
                                    config.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(&dir_name)
                                        .to_string()
                                },
                                Err(_) => dir_name.to_string()
                            }
                        },
                        Err(_) => dir_name.to_string()
                    }
                } else {
                    dir_name.to_string()
                };

                // 统计文档数量
                let documents_count = count_sy_files(&path)?;
                
                // 统计资源文件数量
                let assets_count = count_assets(&path)?;

                notebooks.push(MockNotebook {
                    id: dir_name.to_string(),
                    name,
                    documents_count,
                    assets_count,
                });

                println!("  📖 发现笔记本: {} ({} 个文档, {} 个资源文件)", 
                    name, documents_count, assets_count);
            }
        }
    }

    Ok(notebooks)
}

/// 统计.sy文件数量
fn count_sy_files(notebook_path: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    use walkdir::WalkDir;
    
    let mut count = 0;
    for entry in WalkDir::new(notebook_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "sy") {
            count += 1;
        }
    }
    Ok(count)
}

/// 统计资源文件数量
fn count_assets(notebook_path: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    use walkdir::WalkDir;
    
    let assets_dir = notebook_path.join("assets");
    if !assets_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in WalkDir::new(&assets_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            count += 1;
        }
    }
    Ok(count)
}

/// 展示思源笔记导入功能的主要特性
fn show_features() {
    println!("🌟 思源笔记导入功能特性:");
    println!("  ✅ 自动识别笔记本 (按ID格式: YYYYMMDDHHMMSS-xxxxxxx)");
    println!("  ✅ 解析笔记本配置 (.siyuan/conf.json)");
    println!("  ✅ 解析.sy文档文件 (JSON格式)");
    println!("  ✅ 支持多种节点类型:");
    println!("     • 文本段落 (NodeParagraph)");
    println!("     • 标题 (NodeHeading)");
    println!("     • 列表 (NodeList/NodeListItem)");
    println!("     • 代码块 (NodeCodeBlock)");
    println!("     • 引用块 (NodeBlockquote)");
    println!("     • 文本标记 (NodeTextMark - 加粗、斜体、链接等)");
    println!("     • 块引用 (block-ref)");
    println!("  ✅ 资源文件处理 (图片、附件等)");
    println!("  ✅ 标签提取 (从属性和文本中)");
    println!("  ✅ 转换为Markdown格式");
    println!("  ✅ 完整的错误处理和日志记录");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_scan_empty_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let result = scan_notebooks(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_scan_workspace_with_invalid_dirs() {
        let temp_dir = TempDir::new().unwrap();
        
        // 创建一些无效的目录
        fs::create_dir_all(temp_dir.path().join("invalid-dir")).unwrap();
        fs::create_dir_all(temp_dir.path().join("20210808180117")).unwrap(); // 缺少后缀
        
        let result = scan_notebooks(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_scan_workspace_with_valid_notebook() {
        let temp_dir = TempDir::new().unwrap();
        
        // 创建有效的笔记本目录
        let notebook_dir = temp_dir.path().join("20210808180117-czj9bvb");
        fs::create_dir_all(&notebook_dir).unwrap();
        
        // 创建配置文件
        let siyuan_dir = notebook_dir.join(".siyuan");
        fs::create_dir_all(&siyuan_dir).unwrap();
        
        let config = serde_json::json!({
            "name": "测试笔记本",
            "icon": "1f4d4"
        });
        
        fs::write(
            siyuan_dir.join("conf.json"),
            serde_json::to_string_pretty(&config).unwrap()
        ).unwrap();
        
        // 创建一个.sy文件
        fs::write(
            notebook_dir.join("20200812220555-lj3enxa.sy"),
            r#"{"ID":"20200812220555-lj3enxa","Type":"NodeDocument"}"#
        ).unwrap();
        
        let result = scan_notebooks(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "测试笔记本");
        assert_eq!(result[0].documents_count, 1);
    }
}
