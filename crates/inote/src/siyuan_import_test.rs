#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use serde_json::json;

    /// 创建测试用的思源笔记目录结构
    fn create_test_siyuan_workspace() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // 创建笔记本目录 (符合思源笔记ID格式)
        let notebook_id = "20210808180117-czj9bvb";
        let notebook_path = workspace_path.join(notebook_id);
        fs::create_dir_all(&notebook_path).unwrap();

        // 创建笔记本配置文件
        let siyuan_dir = notebook_path.join(".siyuan");
        fs::create_dir_all(&siyuan_dir).unwrap();
        
        let config = json!({
            "name": "测试笔记本",
            "sort": 1,
            "icon": "1f4d4",
            "closed": false,
            "sortMode": 15
        });
        
        fs::write(
            siyuan_dir.join("conf.json"),
            serde_json::to_string_pretty(&config).unwrap()
        ).unwrap();

        // 创建测试文档
        let doc_id = "20200812220555-lj3enxa";
        let document = json!({
            "ID": doc_id,
            "Spec": "1",
            "Type": "NodeDocument",
            "Properties": {
                "icon": "1f389",
                "id": doc_id,
                "title": "测试文档",
                "type": "doc",
                "updated": "20241125224159"
            },
            "Children": [
                {
                    "ID": "20241125224159-8zf3bos",
                    "Type": "NodeParagraph",
                    "Properties": {
                        "id": "20241125224159-8zf3bos",
                        "updated": "20241125224159"
                    },
                    "Children": [
                        {
                            "Type": "NodeText",
                            "Data": "这是一个测试段落。"
                        }
                    ]
                },
                {
                    "ID": "20241125224200-abc123",
                    "Type": "NodeHeading",
                    "HeadingLevel": 2,
                    "Properties": {
                        "id": "20241125224200-abc123",
                        "updated": "20241125224200"
                    },
                    "Children": [
                        {
                            "Type": "NodeText",
                            "Data": "测试标题"
                        }
                    ]
                },
                {
                    "ID": "20241125224300-def456",
                    "Type": "NodeList",
                    "Properties": {
                        "id": "20241125224300-def456",
                        "updated": "20241125224300"
                    },
                    "Children": [
                        {
                            "Type": "NodeListItem",
                            "Children": [
                                {
                                    "Type": "NodeText",
                                    "Data": "列表项1"
                                }
                            ]
                        },
                        {
                            "Type": "NodeListItem",
                            "Children": [
                                {
                                    "Type": "NodeText",
                                    "Data": "列表项2"
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        fs::write(
            notebook_path.join(format!("{}.sy", doc_id)),
            serde_json::to_string_pretty(&document).unwrap()
        ).unwrap();

        // 创建assets目录和测试图片
        let assets_dir = notebook_path.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        
        // 创建一个虚拟的图片文件
        fs::write(
            assets_dir.join("test-image.png"),
            b"fake image data"
        ).unwrap();

        temp_dir
    }

    #[test]
    fn test_notebook_id_pattern() {
        let temp_dir = create_test_siyuan_workspace();
        let storage = create_test_storage();
        let importer = SiyuanImporter::new(storage, temp_dir.path().to_path_buf());

        // 测试有效的笔记本ID
        assert!(importer.id_pattern.is_match("20210808180117-czj9bvb"));
        assert!(importer.id_pattern.is_match("20200101000000-abc1234"));

        // 测试无效的笔记本ID
        assert!(!importer.id_pattern.is_match("invalid-id"));
        assert!(!importer.id_pattern.is_match("20210808180117"));
        assert!(!importer.id_pattern.is_match("20210808180117-"));
        assert!(!importer.id_pattern.is_match("20210808180117-abc123"));
    }

    #[test]
    fn test_validate_siyuan_directory() {
        let temp_dir = create_test_siyuan_workspace();
        let storage = create_test_storage();
        let importer = SiyuanImporter::new(storage, temp_dir.path().to_path_buf());

        // 应该验证成功
        assert!(importer.validate_siyuan_directory().is_ok());

        // 测试无效目录
        let invalid_storage = create_test_storage();
        let invalid_importer = SiyuanImporter::new(
            invalid_storage, 
            PathBuf::from("/nonexistent/path")
        );
        assert!(invalid_importer.validate_siyuan_directory().is_err());
    }

    #[test]
    fn test_extract_document_title() {
        let storage = create_test_storage();
        let importer = SiyuanImporter::new(storage, PathBuf::new());

        // 测试有title属性的文档
        let document = SiyuanDocument {
            id: "test-id".to_string(),
            spec: Some("1".to_string()),
            doc_type: "NodeDocument".to_string(),
            properties: Some({
                let mut props = HashMap::new();
                props.insert("title".to_string(), json!("测试标题"));
                props
            }),
            children: None,
        };

        assert_eq!(importer.extract_document_title(&document), "测试标题");

        // 测试没有title属性的文档
        let document_no_title = SiyuanDocument {
            id: "test-id-2".to_string(),
            spec: Some("1".to_string()),
            doc_type: "NodeDocument".to_string(),
            properties: None,
            children: None,
        };

        assert_eq!(importer.extract_document_title(&document_no_title), "test-id-2");
    }

    #[test]
    fn test_convert_node_to_markdown() {
        let storage = create_test_storage();
        let importer = SiyuanImporter::new(storage, PathBuf::new());

        // 测试文本节点
        let text_node = SiyuanNode {
            id: None,
            node_type: "NodeText".to_string(),
            data: Some("Hello World".to_string()),
            properties: None,
            children: None,
            heading_level: None,
            text_mark_type: None,
            text_mark_block_ref_id: None,
            text_mark_block_ref_subtype: None,
            text_mark_text_content: None,
        };

        let result = importer.convert_node_to_markdown(&text_node, 0).unwrap();
        assert_eq!(result, "Hello World");

        // 测试标题节点
        let heading_node = SiyuanNode {
            id: Some("heading-id".to_string()),
            node_type: "NodeHeading".to_string(),
            data: None,
            properties: None,
            children: Some(vec![text_node.clone()]),
            heading_level: Some(2),
            text_mark_type: None,
            text_mark_block_ref_id: None,
            text_mark_block_ref_subtype: None,
            text_mark_text_content: None,
        };

        let result = importer.convert_node_to_markdown(&heading_node, 0).unwrap();
        assert_eq!(result, "## Hello World\n\n");
    }

    #[test]
    fn test_extract_hashtags_from_text() {
        let storage = create_test_storage();
        let importer = SiyuanImporter::new(storage, PathBuf::new());
        let mut tags = Vec::new();

        importer.extract_hashtags_from_text("这是一个包含 #标签1 和 #标签2 的文本", &mut tags);
        
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"标签1".to_string()));
        assert!(tags.contains(&"标签2".to_string()));
    }

    #[test]
    fn test_get_file_type() {
        let storage = create_test_storage();
        let importer = SiyuanImporter::new(storage, PathBuf::new());

        assert_eq!(importer.get_file_type("test.jpg"), "image");
        assert_eq!(importer.get_file_type("test.png"), "image");
        assert_eq!(importer.get_file_type("test.pdf"), "pdf");
        assert_eq!(importer.get_file_type("test.mp3"), "audio");
        assert_eq!(importer.get_file_type("test.mp4"), "video");
        assert_eq!(importer.get_file_type("test.txt"), "file");
    }

    // 辅助函数：创建测试用的存储管理器
    fn create_test_storage() -> DbStorageManager {
        // 这里应该创建一个测试用的存储管理器
        // 由于我们没有看到DbStorageManager的具体实现，这里先用占位符
        todo!("需要实现测试用的DbStorageManager")
    }
}
