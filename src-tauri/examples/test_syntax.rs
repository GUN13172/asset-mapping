use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() {
    println!("=== 测试 DayDayMap 查询语法 ===\n");
    
    // 测试不同的语法格式
    let test_queries = vec![
        ("错误语法1", "183.201.199.0/24"),
        ("错误语法2", "ip=183.201.199.0/24"),
        ("正确语法1", "ip:\"183.201.199.0/24\""),
        ("正确语法2", "ip=\"183.201.199.0/24\""),
        ("正确语法3", "ip:\"183.201.199.1\""),
    ];
    
    for (name, query) in test_queries {
        let encoded = general_purpose::STANDARD.encode(query.as_bytes());
        println!("{}: {}", name, query);
        pr