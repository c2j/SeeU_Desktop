# Unicode字符边界错误修复

## 问题描述

在搜索功能中遇到了严重的Unicode字符边界错误：

```
thread 'main' panicked at crates/isearch/src/indexer.rs:781:48:
byte index 134 is not a char boundary; it is inside '乍' (bytes 133..136) of `反对 **澳大利亚** 澳大利亚 反对 **玻利维亚** 南美洲 反对 **巴西** 南美洲 反对 **中非** 非洲 反对 **乍得** 非洲 反对 **刚果民主共和国** 非洲 反对 **哥斯达黎加** 中美洲 反对 **达荷美（贝宁）`[...]
```

这个错误发生在处理包含中文字符的文本时，尝试在字节边界134处访问字符，但该位置在中文字符"乍"的中间（字节133-136）。

## 根本原因

Rust中的字符串是UTF-8编码的，中文字符通常占用3个字节。原代码在以下几个地方使用了不安全的字节索引：

1. **`find_all_term_matches`方法**: 使用`find()`返回的字节位置直接进行字符串切片
2. **`calculate_term_score`方法**: 使用字节位置计算字符位置时没有考虑UTF-8边界
3. **`extract_snippet_for_group`方法**: 字节位置到字符位置的转换不安全

## 修复方案

### 1. 字符级别的搜索匹配

**原代码问题**:
```rust
while let Some(pos) = content_lower[start..].find(term) {
    let absolute_pos = start + pos;
    // 直接使用字节位置，在Unicode字符中会出错
}
```

**修复后**:
```rust
fn find_all_term_matches(&self, content: &str, terms: &[String]) -> Vec<TermMatch> {
    let content_lower = content.to_lowercase();
    let content_chars: Vec<char> = content.chars().collect();
    let content_lower_chars: Vec<char> = content_lower.chars().collect();
    
    // 使用字符级别的匹配，避免字节边界问题
    let mut char_start = 0;
    while char_start + term_chars.len() <= content_lower_chars.len() {
        // 逐字符比较，确保Unicode安全
        let mut matches_here = true;
        for (i, &term_char) in term_chars.iter().enumerate() {
            if content_lower_chars[char_start + i] != term_char {
                matches_here = false;
                break;
            }
        }
        // ...
    }
}
```

### 2. 安全的字符位置转换

**新增辅助方法**:
```rust
/// Safely convert byte position to character position
fn byte_to_char_position(&self, content: &str, byte_pos: usize) -> usize {
    if byte_pos == 0 {
        return 0;
    }
    if byte_pos >= content.len() {
        return content.chars().count();
    }

    let mut char_count = 0;
    let mut byte_count = 0;
    
    for ch in content.chars() {
        if byte_count >= byte_pos {
            break;
        }
        byte_count += ch.len_utf8();
        char_count += 1;
    }
    
    char_count
}
```

### 3. 改进的评分算法

**修复字符边界计算**:
```rust
fn calculate_term_score(&self, term: &str, content: &str, byte_position: usize) -> f32 {
    // 使用字符数而非字节数计算分数
    score += (term.chars().count() as f32) * 0.1;
    
    // 安全的字符位置计算
    let char_pos = if byte_position == 0 {
        0
    } else if byte_position >= content.len() {
        content.chars().count()
    } else {
        // 安全地找到字符位置
        let mut char_count = 0;
        let mut byte_count = 0;
        for ch in content.chars() {
            if byte_count >= byte_position {
                break;
            }
            byte_count += ch.len_utf8();
            char_count += 1;
        }
        char_count
    };
    // ...
}
```

## 测试验证

### 1. Unicode测试用例

创建了专门的测试来验证Unicode字符处理：

```rust
#[test]
fn test_unicode_character_handling() {
    // 测试中文字符
    let terms = vec!["澳大利亚".to_string(), "乍得".to_string()];
    let text = "反对 **澳大利亚** 澳大利亚 反对 **玻利维亚** 南美洲 反对 **巴西** 南美洲 反对 **中非** 非洲 反对 **乍得** 非洲";
    
    let result = highlight_search_terms(text, &terms);
    assert!(result.contains("【澳大利亚】"));
    assert!(result.contains("【乍得】"));
    
    // 测试混合中英文
    let terms = vec!["rust".to_string(), "安全".to_string()];
    let text = "Rust是一种系统编程语言，专注于安全、速度和并发";
    
    let result = highlight_search_terms(text, &terms);
    assert!(result.contains("【Rust】"));
    assert!(result.contains("【安全】"));
    
    // 测试emoji和特殊字符
    let terms = vec!["测试".to_string()];
    let text = "这是一个🎉测试🚀文本";
    
    let result = highlight_search_terms(text, &terms);
    assert!(result.contains("【测试】"));
}
```

### 2. 实际运行测试

创建了完整的演示程序验证修复效果：

```bash
cargo run --example unicode_boundary_test -p isearch
```

**测试结果**:
- ✅ 没有出现字符边界panic错误
- ✅ 中文字符搜索正常工作
- ✅ 搜索结果预览显示正确的上下文
- ✅ 混合Unicode内容处理正常

## 修复效果

### 修复前
```
thread 'main' panicked at crates/isearch/src/indexer.rs:781:48:
byte index 134 is not a char boundary; it is inside '乍' (bytes 133..136)
```

### 修复后
```
查询: "乍得"
  结果 1: unicode_test.txt
  预览: ...**玻利维亚** 南美洲 反对 **巴西** 南美洲 反对 **中非** 非洲 反对 **乍得** 非洲 反对 **刚果民主共和国** 非洲...
```

## 技术要点

### 1. UTF-8字符编码理解
- ASCII字符: 1字节
- 中文字符: 通常3字节
- Emoji字符: 通常4字节
- 字节索引和字符索引不同

### 2. Rust字符串处理最佳实践
- 使用`chars()`迭代器进行字符级操作
- 避免直接使用字节索引进行字符串切片
- 使用`char.len_utf8()`获取字符的字节长度

### 3. 安全的字符边界处理
- 始终在字符边界处进行字符串操作
- 使用字符计数而非字节计数进行长度计算
- 提供安全的字节位置到字符位置转换函数

## 总结

通过这次修复，我们解决了一个严重的Unicode字符边界错误，使搜索功能能够正确处理包含中文、emoji等多字节Unicode字符的内容。修复的核心是将所有字符串操作从字节级别改为字符级别，确保在Unicode字符边界上进行安全的操作。

这个修复不仅解决了crash问题，还提升了搜索功能对国际化内容的支持，使SeeU Desktop能够更好地服务全球用户。
