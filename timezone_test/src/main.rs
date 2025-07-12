use chrono::{DateTime, Utc, Local, TimeZone};

fn main() {
    println!("=== 时区测试程序 ===");

    // 1. 获取当前UTC时间
    let utc_now = Utc::now();
    println!("当前UTC时间: {}", utc_now.format("%Y-%m-%d %H:%M:%S UTC"));

    // 2. 获取当前本地时间
    let local_now = Local::now();
    println!("当前本地时间: {}", local_now.format("%Y-%m-%d %H:%M:%S %Z"));

    // 3. 测试时区转换
    let local_from_utc = utc_now.with_timezone(&Local);
    println!("UTC转本地时间: {}", local_from_utc.format("%Y-%m-%d %H:%M:%S %Z"));

    // 4. 测试当前代码中使用的方法
    let local_time_current_method = utc_now.with_timezone(&Local::now().timezone());
    println!("当前代码方法: {}", local_time_current_method.format("%Y-%m-%d %H:%M:%S %Z"));

    // 5. 测试推荐的方法
    let local_time_recommended = utc_now.with_timezone(&Local);
    println!("推荐方法: {}", local_time_recommended.format("%Y-%m-%d %H:%M:%S %Z"));

    // 6. 比较两种方法是否相同
    println!("两种方法是否相同: {}", local_time_current_method == local_time_recommended);

    // 7. 测试时区偏移
    let local_binding = Local::now();
    let offset = local_binding.offset();
    println!("时区偏移: {}", offset);

    // 8. 测试一个具体的UTC时间戳
    let test_utc = Utc.with_ymd_and_hms(2025, 7, 12, 5, 9, 0).unwrap();
    println!("测试UTC时间: {}", test_utc.format("%Y-%m-%d %H:%M:%S UTC"));

    let test_local_current = test_utc.with_timezone(&Local::now().timezone());
    println!("测试本地时间(当前方法): {}", test_local_current.format("%Y-%m-%d %H:%M:%S %Z"));

    let test_local_recommended = test_utc.with_timezone(&Local);
    println!("测试本地时间(推荐方法): {}", test_local_recommended.format("%Y-%m-%d %H:%M:%S %Z"));

    // 9. 测试RFC3339格式解析
    let rfc3339_str = "2025-07-12T05:09:00Z";
    if let Ok(parsed_utc) = DateTime::parse_from_rfc3339(rfc3339_str) {
        let parsed_utc = parsed_utc.with_timezone(&Utc);
        println!("解析RFC3339: {}", parsed_utc.format("%Y-%m-%d %H:%M:%S UTC"));

        let parsed_local_current = parsed_utc.with_timezone(&Local::now().timezone());
        println!("解析后本地时间(当前方法): {}", parsed_local_current.format("%Y-%m-%d %H:%M:%S %Z"));

        let parsed_local_recommended = parsed_utc.with_timezone(&Local);
        println!("解析后本地时间(推荐方法): {}", parsed_local_recommended.format("%Y-%m-%d %H:%M:%S %Z"));
    }

    println!("=== 测试完成 ===");
}
