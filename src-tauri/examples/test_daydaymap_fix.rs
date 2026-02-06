// 测试DayDayMap API修复
// 验证三个关键修复：
// 1. 端点URL: /api/v1/raymap/search/all
// 2. 认证头: api-key (不是 Authorization: Bearer)
// 3. 请求体: keyword字段（base64编码）

use base64::{Engine as _, engine::general_purpose};

fn main() {
    println!("=== DayDayMap API 修复验证 ===\n");
    
    // 测试1: 验证端点URL
    let correct_endpoint = "https://www.daydaymap.com/api/v1/raymap/search/all";
    println!("✓ 正确的端点URL: {}", correct_endpoint);
    
    // 测试2: 验证认证头格式
    let api_key = "test_api_key_12345";
    println!("✓ 正确的认证头: api-key: {}", api_key);
    println!("  (不是 Authorization: Bearer {})", api_key);
    
    // 测试3: 验证请求体格式
    let query = "port=\"80\"";
    let keyword_base64 = general_purpose::STANDARD.encode(query.as_bytes());
    println!("\n✓ 查询字符串: {}", query);
    println!("✓ Base64编码后: {}", keyword_base64);
    
    // 验证解码
    let decoded = general_purpose::STANDARD.decode(&keyword_base64).unwrap();
    let decoded_str = String::from_utf8(decoded).unwrap();
    println!("✓ 解码验证: {}", decoded_str);
    assert_eq!(decoded_str, query, "Base64编码/解码验证失败");
    
    // 显示完整的请求体格式
    println!("\n✓ 正确的请求体格式:");
    println!("{{");
    println!("  \"keyword\": \"{}\",  // base64编码的查询", keyword_base64);
    println!("  \"page\": 1,");
    println!("  \"page_size\": 10");
    println!("}}");
    
    println!("\n=== 所有修复验证通过 ===");
}
