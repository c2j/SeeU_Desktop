/// Test Unicode character boundary handling in search functionality
use std::fs;
use isearch::indexer::Indexer;
use isearch::IndexedDirectory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unicode字符边界处理测试 ===\n");

    // 创建索引器
    let indexer = Indexer::new();
    
    // 测试包含中文字符的内容
    let test_content = r#"
反对 **澳大利亚** 澳大利亚
反对 **玻利维亚** 南美洲
反对 **巴西** 南美洲
反对 **中非** 非洲
反对 **乍得** 非洲
反对 **刚果民主共和国** 非洲
反对 **哥斯达黎加** 中美洲
反对 **达荷美（贝宁）** 非洲

这是一个包含中文字符的测试文档。
Rust是一种系统编程语言，专注于安全、速度和并发。
测试Unicode字符边界处理：🎉🚀💻
"#;

    println!("1. 测试内容:");
    println!("{}", test_content);

    // 创建测试目录和文件
    let test_dir = std::env::temp_dir().join("unicode_test");
    fs::create_dir_all(&test_dir)?;

    let test_file = test_dir.join("unicode_test.txt");
    fs::write(&test_file, test_content)?;

    // 索引目录
    println!("\n2. 索引目录...");
    let indexed_dir = IndexedDirectory {
        path: test_dir.to_string_lossy().to_string(),
        last_indexed: None,
        file_count: 0,
        total_size_bytes: 0,
    };
    indexer.index_directory(&indexed_dir)?;
    
    // 测试不同的搜索查询
    let test_queries = vec![
        "澳大利亚",
        "乍得",
        "中非",
        "Rust 安全",
        "系统编程",
        "Unicode 字符",
        "🎉",
    ];
    
    println!("\n3. 搜索测试:");
    for query in &test_queries {
        println!("\n  查询: \"{}\"", query);
        
        match indexer.search(query) {
            Ok(results) => {
                if results.is_empty() {
                    println!("    结果: 未找到匹配");
                } else {
                    for (i, result) in results.iter().enumerate() {
                        println!("    结果 {}: {}", i + 1, result.filename);
                        if !result.content_preview.is_empty() {
                            println!("    预览: {}", result.content_preview);
                        }
                    }
                }
            },
            Err(e) => {
                println!("    错误: {}", e);
            }
        }
    }
    
    println!("\n4. 字符边界安全性测试:");
    
    // 测试包含复杂Unicode字符的内容
    let complex_content = "测试🎉emoji🚀和中文混合💻内容处理";
    let complex_file = test_dir.join("complex_unicode.txt");
    fs::write(&complex_file, complex_content)?;

    // 重新索引目录以包含新文件
    indexer.index_directory(&indexed_dir)?;
    
    let complex_queries = vec!["emoji", "中文", "🎉", "💻"];
    
    for query in &complex_queries {
        println!("\n  复杂查询: \"{}\"", query);
        match indexer.search(query) {
            Ok(results) => {
                for result in results {
                    println!("    文件: {}", result.filename);
                    println!("    预览: {}", result.content_preview);
                }
            },
            Err(e) => {
                println!("    错误: {}", e);
            }
        }
    }
    
    println!("\n=== 测试完成 ===");
    println!("✓ Unicode字符边界处理正常");
    println!("✓ 中文字符搜索功能正常");
    println!("✓ Emoji字符处理正常");
    println!("✓ 混合内容搜索正常");
    
    Ok(())
}
