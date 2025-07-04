// 测试Rust文件
use std::collections::HashMap;

fn main() {
    println!("Hello, iFile Editor!");
    
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_main() {
        // 测试主函数
        main();
    }
}
