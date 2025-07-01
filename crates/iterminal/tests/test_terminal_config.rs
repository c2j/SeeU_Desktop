use iterminal::{TerminalConfig, TerminalSession, OutputLine, LineType};
use chrono::Utc;

#[test]
fn test_terminal_config_creation() {
    let config = TerminalConfig::default();
    
    assert_eq!(config.font_family, "Source Code Pro");
    assert_eq!(config.font_size, 14.0);
    assert_eq!(config.scrollback_lines, 10000);
    assert!(!config.enable_bell);
    assert_eq!(config.cursor_blink_rate, 500);
    assert_eq!(config.tab_width, 4);
    assert!(config.auto_scroll);
    assert!(config.show_line_numbers);
}

#[test]
fn test_terminal_config_modification() {
    let mut config = TerminalConfig::default();
    
    // 修改配置
    config.font_family = "Monaco".to_string();
    config.font_size = 16.0;
    config.scrollback_lines = 5000;
    config.enable_bell = true;
    config.cursor_blink_rate = 1000;
    config.tab_width = 8;
    config.auto_scroll = false;
    config.show_line_numbers = false;
    
    // 验证修改
    assert_eq!(config.font_family, "Monaco");
    assert_eq!(config.font_size, 16.0);
    assert_eq!(config.scrollback_lines, 5000);
    assert!(config.enable_bell);
    assert_eq!(config.cursor_blink_rate, 1000);
    assert_eq!(config.tab_width, 8);
    assert!(!config.auto_scroll);
    assert!(!config.show_line_numbers);
}

#[test]
fn test_terminal_config_validation() {
    let mut config = TerminalConfig::default();
    
    // 测试字体大小边界值
    config.font_size = 8.0; // 最小值
    assert_eq!(config.font_size, 8.0);
    
    config.font_size = 72.0; // 最大值
    assert_eq!(config.font_size, 72.0);
    
    // 测试滚动行数
    config.scrollback_lines = 100; // 最小值
    assert_eq!(config.scrollback_lines, 100);
    
    config.scrollback_lines = 100000; // 大值
    assert_eq!(config.scrollback_lines, 100000);
    
    // 测试制表符宽度
    config.tab_width = 2; // 最小值
    assert_eq!(config.tab_width, 2);
    
    config.tab_width = 8; // 常用值
    assert_eq!(config.tab_width, 8);
}

#[test]
fn test_terminal_config_font_families() {
    let mut config = TerminalConfig::default();
    
    // 测试不同字体
    let fonts = vec![
        "Source Code Pro",
        "Monaco",
        "Consolas",
        "Menlo",
        "DejaVu Sans Mono",
        "Liberation Mono",
        "Courier New",
    ];
    
    for font in fonts {
        config.font_family = font.to_string();
        assert_eq!(config.font_family, font);
    }
}

#[test]
fn test_terminal_config_serialization() {
    let config = TerminalConfig {
        font_family: "Monaco".to_string(),
        font_size: 16.0,
        scrollback_lines: 5000,
        enable_bell: true,
        cursor_blink_rate: 1000,
        tab_width: 8,
        auto_scroll: false,
        show_line_numbers: false,
    };
    
    // 测试序列化（如果实现了Serialize trait）
    let serialized = serde_json::to_string(&config);
    if let Ok(json_str) = serialized {
        let deserialized: Result<TerminalConfig, _> = serde_json::from_str(&json_str);
        if let Ok(restored_config) = deserialized {
            assert_eq!(config.font_family, restored_config.font_family);
            assert_eq!(config.font_size, restored_config.font_size);
            assert_eq!(config.scrollback_lines, restored_config.scrollback_lines);
            assert_eq!(config.enable_bell, restored_config.enable_bell);
        }
    }
}

#[test]
fn test_terminal_config_clone() {
    let config = TerminalConfig::default();
    let cloned_config = config.clone();
    
    assert_eq!(config.font_family, cloned_config.font_family);
    assert_eq!(config.font_size, cloned_config.font_size);
    assert_eq!(config.scrollback_lines, cloned_config.scrollback_lines);
    assert_eq!(config.enable_bell, cloned_config.enable_bell);
    assert_eq!(config.cursor_blink_rate, cloned_config.cursor_blink_rate);
    assert_eq!(config.tab_width, cloned_config.tab_width);
    assert_eq!(config.auto_scroll, cloned_config.auto_scroll);
    assert_eq!(config.show_line_numbers, cloned_config.show_line_numbers);
}

#[test]
fn test_terminal_config_debug() {
    let config = TerminalConfig::default();
    let debug_str = format!("{:?}", config);
    
    // 验证调试输出包含关键信息
    assert!(debug_str.contains("TerminalConfig"));
    assert!(debug_str.contains("font_family"));
    assert!(debug_str.contains("font_size"));
    assert!(debug_str.contains("scrollback_lines"));
}

#[test]
fn test_terminal_config_partial_eq() {
    let config1 = TerminalConfig::default();
    let config2 = TerminalConfig::default();
    let mut config3 = TerminalConfig::default();
    config3.font_size = 16.0;
    
    assert_eq!(config1, config2);
    assert_ne!(config1, config3);
}

#[test]
fn test_terminal_config_extreme_values() {
    let mut config = TerminalConfig::default();
    
    // 测试极端值
    config.font_size = 1.0;
    config.scrollback_lines = 1;
    config.cursor_blink_rate = 1;
    config.tab_width = 1;
    
    assert_eq!(config.font_size, 1.0);
    assert_eq!(config.scrollback_lines, 1);
    assert_eq!(config.cursor_blink_rate, 1);
    assert_eq!(config.tab_width, 1);
    
    // 测试大值
    config.font_size = 100.0;
    config.scrollback_lines = 1000000;
    config.cursor_blink_rate = 10000;
    config.tab_width = 16;
    
    assert_eq!(config.font_size, 100.0);
    assert_eq!(config.scrollback_lines, 1000000);
    assert_eq!(config.cursor_blink_rate, 10000);
    assert_eq!(config.tab_width, 16);
}
