use isearch::utils::{highlight_search_terms, extract_search_terms};

#[test]
fn test_highlight_search_terms() {
    let text = "This is a test document with some content.";
    let terms = vec!["test".to_string(), "content".to_string()];
    
    let highlighted = highlight_search_terms(text, &terms);
    
    // 验证高亮标记存在（使用实际的高亮标记）
    assert!(highlighted.contains("【test】"));
    assert!(highlighted.contains("【content】"));
}

#[test]
fn test_extract_search_terms() {
    // 测试基本词汇提取
    let terms = extract_search_terms("hello world test");
    assert!(terms.contains(&"hello".to_string()));
    assert!(terms.contains(&"world".to_string()));
    assert!(terms.contains(&"test".to_string()));
    
    // 测试过滤短词
    let terms = extract_search_terms("hello a world");
    assert!(terms.contains(&"hello".to_string()));
    assert!(terms.contains(&"world".to_string()));
    assert!(!terms.contains(&"a".to_string()));
    
    // 测试引号处理
    let terms = extract_search_terms("\"hello world\" test");
    assert!(terms.contains(&"hello world".to_string()));
    assert!(terms.contains(&"test".to_string()));
}

#[test]
fn test_highlight_case_insensitive() {
    let text = "This is a TEST document with some Content.";
    let terms = vec!["test".to_string(), "content".to_string()];
    
    let highlighted = highlight_search_terms(text, &terms);
    
    // 验证大小写不敏感的高亮
    assert!(highlighted.contains("【TEST】") || highlighted.contains("【test】"));
    assert!(highlighted.contains("【Content】") || highlighted.contains("【content】"));
}

#[test]
fn test_extract_search_terms_with_punctuation() {
    // 测试包含标点符号的搜索词提取
    let terms = extract_search_terms("hello, world! test?");
    assert!(terms.contains(&"hello".to_string()));
    assert!(terms.contains(&"world".to_string()));
    assert!(terms.contains(&"test".to_string()));
    
    // 确保标点符号被正确处理
    assert!(!terms.contains(&"hello,".to_string()));
    assert!(!terms.contains(&"world!".to_string()));
    assert!(!terms.contains(&"test?".to_string()));
}

#[test]
fn test_highlight_multiple_occurrences() {
    let text = "test this test and test again";
    let terms = vec!["test".to_string()];
    
    let highlighted = highlight_search_terms(text, &terms);
    
    // 计算高亮标记的数量
    let highlight_count = highlighted.matches("【test】").count();
    assert_eq!(highlight_count, 3); // 应该有3个"test"被高亮
}

#[test]
fn test_extract_search_terms_empty_input() {
    // 测试空输入
    let terms = extract_search_terms("");
    assert!(terms.is_empty());
    
    // 测试只有空格的输入
    let terms = extract_search_terms("   ");
    assert!(terms.is_empty());
    
    // 测试只有标点符号的输入
    let terms = extract_search_terms("!@#$%^&*()");
    assert!(terms.is_empty());
}

#[test]
fn test_highlight_no_matches() {
    let text = "This is a document without the search term.";
    let terms = vec!["nonexistent".to_string()];
    
    let highlighted = highlight_search_terms(text, &terms);
    
    // 如果没有匹配，文本应该保持不变
    assert_eq!(highlighted, text);
}

#[test]
fn test_extract_search_terms_with_numbers() {
    // 测试包含数字的搜索词
    let terms = extract_search_terms("version 1.0 and build 123");
    assert!(terms.contains(&"version".to_string()));
    assert!(terms.contains(&"1.0".to_string()) || terms.contains(&"1".to_string()));
    assert!(terms.contains(&"and".to_string()));
    assert!(terms.contains(&"build".to_string()));
    assert!(terms.contains(&"123".to_string()));
}

#[test]
fn test_highlight_overlapping_terms() {
    let text = "JavaScript and Java are different languages";
    let terms = vec!["Java".to_string(), "JavaScript".to_string()];
    
    let highlighted = highlight_search_terms(text, &terms);
    
    // 验证重叠的搜索词被正确处理
    assert!(highlighted.contains("【JavaScript】"));
    // Java可能被JavaScript包含，或者单独高亮
    assert!(highlighted.contains("【Java】") || highlighted.contains("【JavaScript】"));
}

#[test]
fn test_extract_search_terms_unicode() {
    // 测试Unicode字符的处理
    let terms = extract_search_terms("你好 world 测试");
    assert!(terms.contains(&"你好".to_string()));
    assert!(terms.contains(&"world".to_string()));
    assert!(terms.contains(&"测试".to_string()));
}

#[test]
fn test_highlight_unicode_text() {
    let text = "这是一个测试文档，包含一些内容。";
    let terms = vec!["测试".to_string(), "内容".to_string()];
    
    let highlighted = highlight_search_terms(text, &terms);
    
    // 验证Unicode文本的高亮
    assert!(highlighted.contains("【测试】"));
    assert!(highlighted.contains("【内容】"));
}
