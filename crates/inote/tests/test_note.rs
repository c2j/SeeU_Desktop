use inote::note::Note;
use chrono::Utc;

#[test]
fn test_note_creation() {
    let note = Note::new("测试笔记".to_string(), "这是测试内容".to_string());
    
    assert!(!note.id.is_empty());
    assert_eq!(note.title, "测试笔记");
    assert_eq!(note.content, "这是测试内容");
    assert!(note.tag_ids.is_empty());
    assert!(note.attachments.is_empty());
    assert!(note.created_at <= Utc::now());
    assert!(note.updated_at <= Utc::now());
}

#[test]
fn test_note_tag_management() {
    let mut note = Note::new("测试笔记".to_string(), "内容".to_string());
    let initial_updated_at = note.updated_at;
    
    // 添加标签
    std::thread::sleep(std::time::Duration::from_millis(1));
    note.add_tag("tag1".to_string());
    assert_eq!(note.tag_ids.len(), 1);
    assert!(note.tag_ids.contains(&"tag1".to_string()));
    assert!(note.updated_at > initial_updated_at);
    
    // 添加重复标签（应该不会重复添加）
    let before_duplicate = note.tag_ids.len();
    note.add_tag("tag1".to_string());
    assert_eq!(note.tag_ids.len(), before_duplicate);
    
    // 添加更多标签
    note.add_tag("tag2".to_string());
    note.add_tag("tag3".to_string());
    assert_eq!(note.tag_ids.len(), 3);
    
    // 移除标签
    note.remove_tag("tag2");
    assert_eq!(note.tag_ids.len(), 2);
    assert!(!note.tag_ids.contains(&"tag2".to_string()));
    assert!(note.tag_ids.contains(&"tag1".to_string()));
    assert!(note.tag_ids.contains(&"tag3".to_string()));
}

#[test]
fn test_note_attachment_management() {
    let mut note = Note::new("测试笔记".to_string(), "内容".to_string());
    let initial_updated_at = note.updated_at;
    
    // 添加附件
    std::thread::sleep(std::time::Duration::from_millis(1));
    note.add_attachment("test.txt".to_string(), "/path/to/test.txt".to_string(), "text/plain".to_string());
    assert_eq!(note.attachments.len(), 1);
    assert!(note.updated_at > initial_updated_at);
    
    let attachment = &note.attachments[0];
    assert!(!attachment.id.is_empty());
    assert_eq!(attachment.name, "test.txt");
    assert_eq!(attachment.file_path, "/path/to/test.txt");
    assert_eq!(attachment.file_type, "text/plain");
    
    // 添加更多附件
    note.add_attachment("image.png".to_string(), "/path/to/image.png".to_string(), "image/png".to_string());
    assert_eq!(note.attachments.len(), 2);
    
    // 移除附件
    let attachment_id = note.attachments[0].id.clone();
    note.remove_attachment(&attachment_id);
    assert_eq!(note.attachments.len(), 1);
    assert_eq!(note.attachments[0].name, "image.png");
}

#[test]
fn test_note_content_updates() {
    let mut note = Note::new("原始标题".to_string(), "原始内容".to_string());
    let initial_created_at = note.created_at;
    let initial_updated_at = note.updated_at;
    
    // 等待一小段时间确保时间戳不同
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    // 更新笔记内容
    note.title = "更新后的标题".to_string();
    note.content = "更新后的内容".to_string();
    note.updated_at = Utc::now();
    
    // 验证更新
    assert_eq!(note.title, "更新后的标题");
    assert_eq!(note.content, "更新后的内容");
    assert_eq!(note.created_at, initial_created_at); // 创建时间不变
    assert!(note.updated_at > initial_updated_at); // 更新时间改变
}

#[test]
fn test_note_empty_content() {
    let note = Note::new("".to_string(), "".to_string());
    
    assert!(note.title.is_empty());
    assert!(note.content.is_empty());
    assert!(!note.id.is_empty()); // ID应该仍然生成
}

#[test]
fn test_note_large_content() {
    let large_title = "很长的标题".repeat(100);
    let large_content = "很长的内容。".repeat(10000);
    
    let note = Note::new(large_title.clone(), large_content.clone());
    
    assert_eq!(note.title, large_title);
    assert_eq!(note.content, large_content);
    assert!(!note.id.is_empty());
}

#[test]
fn test_note_special_characters() {
    let title_with_special = "标题包含特殊字符: !@#$%^&*()_+-=[]{}|;':\",./<>?";
    let content_with_special = "内容包含换行符\n和制表符\t以及其他特殊字符 🎉 📝 ✨";
    
    let note = Note::new(title_with_special.to_string(), content_with_special.to_string());
    
    assert_eq!(note.title, title_with_special);
    assert_eq!(note.content, content_with_special);
}

#[test]
fn test_note_attachment_file_types() {
    let mut note = Note::new("附件测试".to_string(), "测试各种附件类型".to_string());
    
    // 添加不同类型的附件
    note.add_attachment("document.pdf".to_string(), "/path/to/document.pdf".to_string(), "application/pdf".to_string());
    note.add_attachment("image.jpg".to_string(), "/path/to/image.jpg".to_string(), "image/jpeg".to_string());
    note.add_attachment("video.mp4".to_string(), "/path/to/video.mp4".to_string(), "video/mp4".to_string());
    note.add_attachment("archive.zip".to_string(), "/path/to/archive.zip".to_string(), "application/zip".to_string());
    
    assert_eq!(note.attachments.len(), 4);
    
    // 验证不同类型的附件
    let file_types: Vec<&str> = note.attachments.iter().map(|a| a.file_type.as_str()).collect();
    assert!(file_types.contains(&"application/pdf"));
    assert!(file_types.contains(&"image/jpeg"));
    assert!(file_types.contains(&"video/mp4"));
    assert!(file_types.contains(&"application/zip"));
}

#[test]
fn test_note_tag_operations() {
    let mut note = Note::new("标签测试".to_string(), "测试标签操作".to_string());
    
    // 测试添加多个标签
    let tags = vec!["工作", "重要", "待办", "项目A"];
    for tag in &tags {
        note.add_tag(tag.to_string());
    }
    
    assert_eq!(note.tag_ids.len(), tags.len());
    
    // 验证所有标签都被添加
    for tag in &tags {
        assert!(note.tag_ids.contains(&tag.to_string()));
    }
    
    // 测试移除标签
    note.remove_tag("待办");
    assert_eq!(note.tag_ids.len(), tags.len() - 1);
    assert!(!note.tag_ids.contains(&"待办".to_string()));
    
    // 测试移除不存在的标签
    note.remove_tag("不存在的标签");
    assert_eq!(note.tag_ids.len(), tags.len() - 1); // 长度不变
}
