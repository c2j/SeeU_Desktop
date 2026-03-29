/// 字符串处理工具函数

/// 安全地从字符串尾部截取指定字节数的子串
/// 确保返回的子串总是有效的 UTF-8 字符串
///
/// # 参数
/// - `content`: 原始字符串
/// - `max_bytes`: 最大保留的字节数（从尾部开始）
///
/// # 返回
/// 如果字符串长度超过 `max_bytes`，返回尾部的 `max_bytes` 字节（对齐到字符边界）
/// 否则返回原始字符串
///
/// # 示例
/// ```rust
/// let s = "Hello世界";
/// assert_eq!(safe_truncate_tail(s, 2), "界");
/// assert_eq!(safe_truncate_tail(s, 10), "Hello世界");
/// ```
pub fn safe_truncate_tail(content: &str, max_bytes: usize) -> &str {
    if content.len() <= max_bytes {
        return content;
    }

    let start_byte = content.len() - max_bytes;

    // 找到最近的字符边界（从 start_byte 开始查找第一个合法的字符边界）
    match content
        .char_indices()
        .find(|&(byte_pos, _)| byte_pos >= start_byte)
    {
        Some((valid_pos, _)) => &content[valid_pos..],
        None => content, // 兜底：返回原字符串
    }
}

/// 安全地从字符串头部截取指定字节数的子串
/// 确保返回的子串总是有效的 UTF-8 字符串
///
/// # 参数
/// - `content`: 原始字符串
/// - `max_bytes`: 最大保留的字节数（从头部开始）
///
/// # 返回
/// 如果字符串长度超过 `max_bytes`，返回头部的 `max_bytes` 字节（对齐到字符边界）
/// 否则返回原始字符串
///
/// # 示例
/// ```rust
/// let s = "Hello世界";
/// assert_eq!(safe_truncate_head(s, 5), "Hello");
/// assert_eq!(safe_truncate_head(s, 10), "Hello世界");
/// ```
pub fn safe_truncate_head(content: &str, max_bytes: usize) -> &str {
    if content.len() <= max_bytes {
        return content;
    }

    // 找到不超过 max_bytes 的最后一个字符边界
    match content
        .char_indices()
        .take_while(|&(byte_pos, _)| byte_pos <= max_bytes)
        .last()
    {
        Some((valid_pos, _)) => &content[..valid_pos],
        None => "", // 兜底：返回空字符串
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_truncate_tail_ascii() {
        let s = "Hello, World!";
        assert_eq!(safe_truncate_tail(s, 5), "orld!");
        assert_eq!(safe_truncate_tail(s, 13), "Hello, World!");
        assert_eq!(safe_truncate_tail(s, 20), "Hello, World!");
    }

    #[test]
    fn test_safe_truncate_tail_multibyte() {
        let s = "Hello世界"; // "世界" 每个 3 字节
        assert_eq!(safe_truncate_tail(s, 1), "界");
        assert_eq!(safe_truncate_tail(s, 3), "界");
        assert_eq!(safe_truncate_tail(s, 4), "界");
        assert_eq!(safe_truncate_tail(s, 5), "世界");
        assert_eq!(safe_truncate_tail(s, 11), "Hello世界");
    }

    #[test]
    fn test_safe_truncate_tail_box_drawing_chars() {
        let s = "┃  make sure"; // "┃" 是 3 字节的 box-drawing 字符
        assert_eq!(safe_truncate_tail(s, 5), "sure");
        assert_eq!(safe_truncate_tail(s, 10), " make sure");
    }

    #[test]
    fn test_safe_truncate_head_ascii() {
        let s = "Hello, World!";
        assert_eq!(safe_truncate_head(s, 5), "Hello");
        assert_eq!(safe_truncate_head(s, 13), "Hello, World!");
        assert_eq!(safe_truncate_head(s, 20), "Hello, World!");
    }

    #[test]
    fn test_safe_truncate_head_multibyte() {
        let s = "Hello世界";
        assert_eq!(safe_truncate_head(s, 5), "Hello");
        assert_eq!(safe_truncate_head(s, 8), "Hello");
        assert_eq!(safe_truncate_head(s, 9), "Hello世");
        assert_eq!(safe_truncate_head(s, 11), "Hello世界");
    }

    #[test]
    fn test_safe_truncate_head_box_drawing_chars() {
        let s = "┃  make sure";
        assert_eq!(safe_truncate_head(s, 3), "┃");
        assert_eq!(safe_truncate_head(s, 4), "┃");
        assert_eq!(safe_truncate_head(s, 5), "┃ ");
    }
}
