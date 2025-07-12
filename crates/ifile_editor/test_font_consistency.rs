// 这是一个测试文件，用于验证字体、行间距、行号显示的一致性
// 第2行：包含中文和英文混合内容
fn main() {
    println!("Hello, world! 你好世界！");
    let very_long_line_that_should_wrap_when_word_wrap_is_enabled_and_should_show_horizontal_scroll_when_disabled = "This is a very long string to test wrapping behavior";
    
    // 第7行：测试不同字符类型
    let numbers = 1234567890;
    let symbols = !@#$%^&*()_+-=[]{}|;':\",./<>?;
    let chinese = "这是一行包含中文字符的文本，用来测试中文字符的显示效果";
    
    // 第12行：测试代码缩进
    if true {
        if true {
            if true {
                println!("深层嵌套的代码块");
            }
        }
    }
    
    // 第20行：测试空行和注释
    
    /* 多行注释
       第二行注释
       第三行注释 */
    
    // 第26行：测试各种语法元素
    struct TestStruct {
        field1: String,
        field2: i32,
        field3: Vec<String>,
    }
    
    impl TestStruct {
        fn new() -> Self {
            Self {
                field1: "test".to_string(),
                field2: 42,
                field3: vec!["a".to_string(), "b".to_string()],
            }
        }
    }
}

// 第42行：文件结束
