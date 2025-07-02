use inote::db_state::DbINoteState;
use inote::slide::SlideParser;

#[cfg(test)]
mod slideshow_detection_tests {
    use super::*;

    #[test]
    fn test_slideshow_detection_with_slide_separator() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();
        
        // 测试包含 --slide 分隔符的内容
        let content_with_slide = r#"
# 第一张幻灯片

这是第一张幻灯片的内容。

--slide

# 第二张幻灯片

这是第二张幻灯片的内容。
"#;
        
        // 两个检测方法应该都返回 true
        assert!(state.check_slideshow_format(content_with_slide), "DbINoteState 应该检测到幻灯片格式");
        assert!(parser.is_slideshow(content_with_slide), "SlideParser 应该检测到幻灯片格式");
    }

    #[test]
    fn test_slideshow_detection_with_triple_dash() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();
        
        // 测试包含 --- 分隔符的内容
        let content_with_dash = r#"
# 第一张幻灯片

这是第一张幻灯片的内容。

---

# 第二张幻灯片

这是第二张幻灯片的内容。
"#;
        
        // 两个检测方法应该都返回 true
        assert!(state.check_slideshow_format(content_with_dash), "DbINoteState 应该检测到幻灯片格式");
        assert!(parser.is_slideshow(content_with_dash), "SlideParser 应该检测到幻灯片格式");
    }

    #[test]
    fn test_slideshow_detection_with_css() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();
        
        // 测试包含 CSS 样式的内容
        let content_with_css = r#"
# 幻灯片标题

```css
.slide {
    background: #f0f0f0;
    color: #333;
}
```

这是幻灯片内容。
"#;
        
        // 两个检测方法应该都返回 true
        assert!(state.check_slideshow_format(content_with_css), "DbINoteState 应该检测到CSS样式");
        assert!(parser.is_slideshow(content_with_css), "SlideParser 应该检测到CSS样式");
    }

    #[test]
    fn test_slideshow_detection_with_config() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();
        
        // 测试包含配置标记的内容
        let content_with_config = r#"
# 幻灯片标题

slide-config: theme=dark

这是幻灯片内容。
"#;
        
        // 两个检测方法应该都返回 true
        assert!(state.check_slideshow_format(content_with_config), "DbINoteState 应该检测到配置标记");
        assert!(parser.is_slideshow(content_with_config), "SlideParser 应该检测到配置标记");
    }

    #[test]
    fn test_slideshow_detection_negative_cases() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();
        
        // 测试普通 Markdown 内容
        let normal_content = r#"
# 普通标题

这是普通的 Markdown 内容。

## 子标题

- 列表项 1
- 列表项 2

**粗体文本** 和 *斜体文本*。
"#;
        
        // 两个检测方法应该都返回 false
        assert!(!state.check_slideshow_format(normal_content), "DbINoteState 不应该检测到幻灯片格式");
        assert!(!parser.is_slideshow(normal_content), "SlideParser 不应该检测到幻灯片格式");
    }

    #[test]
    fn test_slideshow_detection_edge_cases() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();
        
        // 测试边界情况：只有一个分隔符
        let single_separator = r#"
# 标题

内容

--slide
"#;
        
        // 应该被检测为幻灯片（至少有一个分隔符）
        assert!(state.check_slideshow_format(single_separator), "单个分隔符应该被检测为幻灯片");
        assert!(parser.is_slideshow(single_separator), "单个分隔符应该被检测为幻灯片");
        
        // 测试空内容
        let empty_content = "";
        assert!(!state.check_slideshow_format(empty_content), "空内容不应该被检测为幻灯片");
        assert!(!parser.is_slideshow(empty_content), "空内容不应该被检测为幻灯片");
    }

    #[test]
    fn test_slideshow_detection_consistency() {
        let state = DbINoteState::default();
        let parser = SlideParser::new();

        // 测试多种内容，确保两个检测方法的结果一致
        let test_cases = vec![
            ("--slide\n# 标题", true),
            ("---\n# 标题", true),
            ("```css\n.slide{}\n```", true),
            ("slide-config: test", true),
            ("slideshow: enabled", true),
            ("# 普通标题\n普通内容", false),
            ("", false),
            ("---\n普通内容", true), // 有分隔符就算幻灯片
        ];

        for (content, expected) in test_cases {
            let state_result = state.check_slideshow_format(content);
            let parser_result = parser.is_slideshow(content);

            assert_eq!(state_result, expected, "DbINoteState 检测结果不符合预期: {}", content);
            assert_eq!(parser_result, expected, "SlideParser 检测结果不符合预期: {}", content);
            assert_eq!(state_result, parser_result, "两个检测方法结果不一致: {}", content);
        }
    }

    #[test]
    fn test_user_specific_content() {
        // 测试用户提供的具体内容
        let state = DbINoteState::default();
        let parser = SlideParser::new();

        let user_content = r#"# 第一张幻灯片
内容...

---

# 第二张幻灯片
内容...

--slide

# 第三张幻灯片
内容..."#;

        println!("测试用户内容:");
        println!("{}", user_content);

        let state_result = state.check_slideshow_format(user_content);
        let parser_result = parser.is_slideshow(user_content);

        println!("DbINoteState 检测结果: {}", state_result);
        println!("SlideParser 检测结果: {}", parser_result);

        // 分析分隔符
        let lines: Vec<&str> = user_content.lines().collect();
        let mut separator_count = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "---" || trimmed == "--slide" || trimmed.starts_with("---slide") {
                separator_count += 1;
                println!("找到分隔符在第{}行: '{}'", i + 1, trimmed);
            }
        }

        println!("总分隔符数量: {}", separator_count);

        // 两个检测方法都应该返回 true
        assert!(state_result, "DbINoteState 应该检测到幻灯片格式");
        assert!(parser_result, "SlideParser 应该检测到幻灯片格式");
        assert_eq!(state_result, parser_result, "两个检测方法结果应该一致");
    }

    #[test]
    fn test_current_note_slideshow_detection() {
        // 测试 is_current_note_slideshow 方法
        let mut state = DbINoteState::default();

        let user_content = r#"# 第一张幻灯片
内容...

---

# 第二张幻灯片
内容...

--slide

# 第三张幻灯片
内容..."#;

        // 设置当前笔记
        state.current_note = Some("test_note_id".to_string());
        state.note_content = user_content.to_string();

        println!("测试 is_current_note_slideshow 方法:");
        println!("当前笔记内容:");
        println!("{}", state.note_content);

        let result = state.is_current_note_slideshow();
        println!("is_current_note_slideshow 结果: {}", result);

        // 应该返回 true
        assert!(result, "is_current_note_slideshow 应该返回 true");

        // 测试没有当前笔记的情况
        state.current_note = None;
        let result_no_note = state.is_current_note_slideshow();
        println!("没有当前笔记时的结果: {}", result_no_note);
        assert!(!result_no_note, "没有当前笔记时应该返回 false");

        // 测试普通内容
        state.current_note = Some("test_note_id".to_string());
        state.note_content = "# 普通标题\n这是普通内容".to_string();
        let result_normal = state.is_current_note_slideshow();
        println!("普通内容的结果: {}", result_normal);
        assert!(!result_normal, "普通内容应该返回 false");
    }

    #[test]
    fn test_slideshow_parsing() {
        // 测试幻灯片解析功能
        let parser = SlideParser::new();

        let user_content = r#"# 第一张幻灯片
内容...

---

# 第二张幻灯片
内容...

--slide

# 第三张幻灯片
内容..."#;

        println!("测试幻灯片解析:");
        println!("内容:");
        println!("{}", user_content);

        // 首先测试检测
        let is_slideshow = parser.is_slideshow(user_content);
        println!("is_slideshow 结果: {}", is_slideshow);
        assert!(is_slideshow, "应该检测为幻灯片格式");

        // 然后测试解析
        match parser.parse(user_content) {
            Ok(slideshow) => {
                println!("解析成功！幻灯片数量: {}", slideshow.slides.len());
                assert!(slideshow.slides.len() > 0, "应该至少有一张幻灯片");
            }
            Err(e) => {
                panic!("解析失败: {}", e);
            }
        }
    }

    #[test]
    fn test_start_slideshow_with_current_content() {
        // 测试 start_slideshow 方法使用当前编辑内容
        let mut state = DbINoteState::default();

        let user_content = r#"# 第一张幻灯片
内容...

---

# 第二张幻灯片
内容...

--slide

# 第三张幻灯片
内容..."#;

        // 设置当前笔记和内容
        state.current_note = Some("test_note_id".to_string());
        state.note_content = user_content.to_string();

        println!("测试 start_slideshow 方法:");
        println!("当前笔记内容:");
        println!("{}", state.note_content);

        // 首先验证检测
        let is_slideshow = state.is_current_note_slideshow();
        println!("is_current_note_slideshow 结果: {}", is_slideshow);
        assert!(is_slideshow, "应该检测为幻灯片格式");

        // 然后测试启动幻灯片
        match state.start_slideshow() {
            Ok(()) => {
                println!("幻灯片启动成功！");
                // 验证幻灯片状态
                assert!(state.slide_play_state.is_playing, "幻灯片应该正在播放");
            }
            Err(e) => {
                panic!("幻灯片启动失败: {}", e);
            }
        }

        // 测试没有当前笔记的情况
        state.current_note = None;
        match state.start_slideshow() {
            Ok(()) => {
                panic!("没有当前笔记时不应该成功启动幻灯片");
            }
            Err(e) => {
                println!("没有当前笔记时正确返回错误: {}", e);
                assert_eq!(e, "No note selected");
            }
        }
    }
}
