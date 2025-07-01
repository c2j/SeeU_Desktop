use inote::tag::Tag;
use inote::truncate_note_title;
use chrono::Utc;

#[test]
fn test_tag_creation() {
    let tag = Tag::new("测试标签".to_string(), "#FF0000".to_string());
    
    assert!(!tag.id.is_empty());
    assert_eq!(tag.name, "测试标签");
    assert_eq!(tag.color, "#FF0000");
    assert!(tag.created_at <= Utc::now());
}

#[test]
fn test_tag_color_validation() {
    // 测试各种颜色格式
    let tag1 = Tag::new("红色标签".to_string(), "#FF0000".to_string());
    let tag2 = Tag::new("蓝色标签".to_string(), "#0000FF".to_string());
    let tag3 = Tag::new("绿色标签".to_string(), "#00FF00".to_string());
    
    assert_eq!(tag1.color, "#FF0000");
    assert_eq!(tag2.color, "#0000FF");
    assert_eq!(tag3.color, "#00FF00");
    
    // 测试小写颜色代码
    let tag4 = Tag::new("紫色标签".to_string(), "#ff00ff".to_string());
    assert_eq!(tag4.color, "#ff00ff");
}

#[test]
fn test_tag_name_variations() {
    // 测试不同类型的标签名称
    let tag1 = Tag::new("工作".to_string(), "#FF0000".to_string());
    let tag2 = Tag::new("Personal Project".to_string(), "#00FF00".to_string());
    let tag3 = Tag::new("重要-紧急".to_string(), "#0000FF".to_string());
    let tag4 = Tag::new("TODO_2024".to_string(), "#FFFF00".to_string());
    
    assert_eq!(tag1.name, "工作");
    assert_eq!(tag2.name, "Personal Project");
    assert_eq!(tag3.name, "重要-紧急");
    assert_eq!(tag4.name, "TODO_2024");
}

#[test]
fn test_tag_id_uniqueness() {
    let tag1 = Tag::new("标签1".to_string(), "#FF0000".to_string());
    let tag2 = Tag::new("标签2".to_string(), "#00FF00".to_string());
    
    // 验证ID是唯一的
    assert_ne!(tag1.id, tag2.id);
    assert!(!tag1.id.is_empty());
    assert!(!tag2.id.is_empty());
}

#[test]
fn test_truncate_note_title() {
    // 测试短标题
    assert_eq!(truncate_note_title("短标题"), "短标题");
    
    // 测试长标题
    let long_title = "这是一个非常长的笔记标题，超过了十六个汉字的限制，应该被截断";
    let truncated = truncate_note_title(long_title);
    assert_eq!(truncated, "这是一个非常长的笔记标题，超过了...");
    
    // 测试恰好16个字符的标题
    let exact_title = "恰好十六个汉字的标题测试用例";
    assert_eq!(truncate_note_title(exact_title), exact_title);
    
    // 测试空标题
    assert_eq!(truncate_note_title(""), "");
    
    // 测试英文标题
    let english_title = "This is a very long English title that should be truncated";
    let truncated_english = truncate_note_title(english_title);
    assert_eq!(truncated_english, "This is a very l...");
}

#[test]
fn test_truncate_note_title_edge_cases() {
    // 测试只有空格的标题
    assert_eq!(truncate_note_title("   "), "   ");
    
    // 测试包含特殊字符的标题
    let special_title = "标题包含特殊字符!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let truncated_special = truncate_note_title(special_title);
    assert!(truncated_special.len() <= 19); // 16 + "..."
    
    // 测试混合中英文的标题
    let mixed_title = "Mixed中英文Title测试用例很长的标题需要截断";
    let truncated_mixed = truncate_note_title(mixed_title);
    assert!(truncated_mixed.ends_with("..."));
    assert!(truncated_mixed.len() <= 19);
}

#[test]
fn test_truncate_note_title_unicode() {
    // 测试包含emoji的标题
    let emoji_title = "📝 这是一个包含emoji的标题 🎉 需要测试截断功能 ✨";
    let truncated_emoji = truncate_note_title(emoji_title);
    assert!(truncated_emoji.len() <= 19);
    
    // 测试其他Unicode字符
    let unicode_title = "Ñoël 测试 Ümlauts 和其他特殊字符的处理能力";
    let truncated_unicode = truncate_note_title(unicode_title);
    assert!(truncated_unicode.len() <= 19);
}

#[test]
fn test_tag_empty_name() {
    let tag = Tag::new("".to_string(), "#FF0000".to_string());
    
    assert!(tag.name.is_empty());
    assert_eq!(tag.color, "#FF0000");
    assert!(!tag.id.is_empty()); // ID应该仍然生成
}

#[test]
fn test_tag_creation_timestamp() {
    let before = Utc::now();
    let tag = Tag::new("时间戳测试".to_string(), "#FF0000".to_string());
    let after = Utc::now();
    
    assert!(tag.created_at >= before);
    assert!(tag.created_at <= after);
}

#[test]
fn test_tag_color_formats() {
    // 测试不同的颜色格式（虽然可能不都被支持）
    let formats = vec![
        "#FF0000",    // 标准十六进制
        "#ff0000",    // 小写十六进制
        "red",        // 颜色名称
        "rgb(255,0,0)", // RGB格式
        "#F00",       // 短十六进制
    ];
    
    for color in formats {
        let tag = Tag::new(format!("测试{}", color), color.to_string());
        assert_eq!(tag.color, color);
    }
}

#[test]
fn test_multiple_tags_creation() {
    let tag_data = vec![
        ("工作", "#FF0000"),
        ("个人", "#00FF00"),
        ("学习", "#0000FF"),
        ("娱乐", "#FFFF00"),
        ("健康", "#FF00FF"),
    ];
    
    let mut tags = Vec::new();
    for (name, color) in tag_data {
        tags.push(Tag::new(name.to_string(), color.to_string()));
    }
    
    assert_eq!(tags.len(), 5);
    
    // 验证所有标签都有唯一的ID
    let mut ids = std::collections::HashSet::new();
    for tag in &tags {
        assert!(ids.insert(tag.id.clone()));
    }
    
    // 验证标签内容
    assert_eq!(tags[0].name, "工作");
    assert_eq!(tags[1].name, "个人");
    assert_eq!(tags[2].name, "学习");
    assert_eq!(tags[3].name, "娱乐");
    assert_eq!(tags[4].name, "健康");
}
