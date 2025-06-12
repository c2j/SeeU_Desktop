/// Demo showing the improved search preview functionality with keyword highlighting
use isearch::utils::{extract_search_terms, highlight_search_terms, truncate_with_ellipsis};

fn main() {
    println!("=== SeeU Desktop 搜索预览优化演示 ===\n");

    // 模拟文档内容
    let sample_content = r#"
Rust是一种系统编程语言，专注于安全、速度和并发。Rust的设计目标是成为一个"安全、并发、实用"的语言，支持函数式和命令式以及泛型等编程范式。

Rust语言的主要特点包括：
1. 内存安全：Rust通过所有权系统在编译时防止内存错误
2. 零成本抽象：抽象不会带来运行时开销
3. 并发安全：防止数据竞争和其他并发问题
4. 跨平台支持：可以在多种操作系统和架构上运行

Rust的包管理器Cargo使得依赖管理和项目构建变得简单。Cargo不仅可以管理依赖，还可以运行测试、生成文档等。

在Web开发领域，Rust也有很多优秀的框架，如Actix-web、Rocket等。这些框架提供了高性能的Web服务能力。

Rust的学习曲线相对较陡，但一旦掌握，开发者就能编写出既安全又高效的代码。
"#;

    // 测试不同的搜索查询
    let test_queries = vec![
        "Rust 安全",
        "内存 并发",
        "Cargo 管理",
        "Web 框架",
        "学习曲线",
        "filetype:rs Rust", // 包含操作符的查询
    ];

    println!("1. 搜索词提取演示:");
    for query in &test_queries {
        let terms = extract_search_terms(query);
        println!("  查询: \"{}\" -> 搜索词: {:?}", query, terms);
    }

    println!("\n2. 智能预览片段提取演示:");
    for query in &test_queries {
        let terms = extract_search_terms(query);
        if !terms.is_empty() {
            println!("\n  查询: \"{}\"", query);
            println!("  搜索词: {:?}", terms);
            
            // 模拟智能片段提取
            let preview = extract_relevant_preview(sample_content, &terms, 150);
            println!("  预览: {}", preview);
        }
    }

    println!("\n3. 关键词高亮演示:");
    let highlight_query = "Rust 安全 并发";
    let terms = extract_search_terms(highlight_query);
    let preview = extract_relevant_preview(sample_content, &terms, 200);
    let highlighted = highlight_search_terms(&preview, &terms);
    
    println!("  原始预览: {}", preview);
    println!("  高亮预览: {}", highlighted);

    println!("\n4. 多片段组合演示:");
    let multi_query = "Cargo Web";
    let terms = extract_search_terms(multi_query);
    let snippets = extract_multiple_snippets(sample_content, &terms);
    
    println!("  查询: \"{}\"", multi_query);
    println!("  找到 {} 个相关片段:", snippets.len());
    for (i, snippet) in snippets.iter().enumerate() {
        let highlighted = highlight_search_terms(snippet, &terms);
        println!("    片段 {}: {}", i + 1, highlighted);
    }

    println!("\n=== 优化效果总结 ===");
    println!("✓ 智能提取包含搜索关键词的文本片段");
    println!("✓ 支持多个不连续的匹配片段组合");
    println!("✓ 关键词高亮显示，提升可读性");
    println!("✓ 上下文感知的片段边界检测");
    println!("✓ 相关性评分，优先显示最相关的内容");
}

/// 模拟智能预览片段提取
fn extract_relevant_preview(content: &str, terms: &[String], max_length: usize) -> String {
    if terms.is_empty() {
        return truncate_with_ellipsis(content.trim(), max_length);
    }

    // 查找包含搜索词的句子
    let sentences: Vec<&str> = content.split(&['.', '。', '\n'][..])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 10)
        .collect();

    let mut best_sentences = Vec::new();
    
    for sentence in sentences {
        let sentence_lower = sentence.to_lowercase();
        let mut score = 0;
        
        for term in terms {
            if sentence_lower.contains(term) {
                score += 1;
            }
        }
        
        if score > 0 {
            best_sentences.push((sentence, score));
        }
    }

    if best_sentences.is_empty() {
        return truncate_with_ellipsis(content.trim(), max_length);
    }

    // 按评分排序
    best_sentences.sort_by(|a, b| b.1.cmp(&a.1));

    // 组合最佳句子
    let mut result = String::new();
    for (sentence, _score) in best_sentences.iter().take(2) {
        if !result.is_empty() {
            result.push_str(" ... ");
        }
        result.push_str(sentence);
        
        if result.len() >= max_length {
            break;
        }
    }

    truncate_with_ellipsis(&result, max_length)
}

/// 提取多个相关片段
fn extract_multiple_snippets(content: &str, terms: &[String]) -> Vec<String> {
    let mut snippets = Vec::new();
    
    let paragraphs: Vec<&str> = content.split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    for paragraph in paragraphs {
        let paragraph_lower = paragraph.to_lowercase();
        let mut has_term = false;
        
        for term in terms {
            if paragraph_lower.contains(term) {
                has_term = true;
                break;
            }
        }
        
        if has_term {
            // 提取包含关键词的句子
            let sentences: Vec<&str> = paragraph.split(&['.', '。'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
                
            for sentence in sentences {
                let sentence_lower = sentence.to_lowercase();
                for term in terms {
                    if sentence_lower.contains(term) {
                        let snippet = truncate_with_ellipsis(sentence, 100);
                        if !snippets.contains(&snippet) {
                            snippets.push(snippet);
                        }
                        break;
                    }
                }
            }
        }
    }

    snippets
}
