use std::path::PathBuf;

fn main() {
    // 测试搜索功能
    let workspace_dir = PathBuf::from("/Volumes/Raiden_C2J/Projects/Desktop_Projects/CU/SeeU-Desktop_V5");
    
    // 初始化搜索索引器
    let mut indexer = isearch::indexer::Indexer::new(workspace_dir.clone());
    
    // 添加测试文件目录到索引
    let test_dir = workspace_dir.join("test_files");
    indexer.add_directory(test_dir);
    
    // 构建索引
    if let Err(e) = indexer.build_index() {
        eprintln!("构建索引失败: {}", e);
        return;
    }
    
    println!("索引构建完成！");
    
    // 测试搜索 "From"
    println!("\n=== 测试搜索 'From' ===");
    match indexer.search("From") {
        Ok(results) => {
            println!("找到 {} 个结果:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {} (分数: {:.2})", i + 1, result.filename, result.score);
                println!("   路径: {}", result.path);
                println!("   预览: {}", result.content_preview);
                println!();
            }
        }
        Err(e) => eprintln!("搜索失败: {}", e),
    }
    
    // 测试搜索 "From202206"
    println!("\n=== 测试搜索 'From202206' ===");
    match indexer.search("From202206") {
        Ok(results) => {
            println!("找到 {} 个结果:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {} (分数: {:.2})", i + 1, result.filename, result.score);
                println!("   路径: {}", result.path);
                println!("   预览: {}", result.content_preview);
                println!();
            }
        }
        Err(e) => eprintln!("搜索失败: {}", e),
    }
}
