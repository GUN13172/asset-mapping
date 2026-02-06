// 测试用户信息接口
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    println!("=== 测试用户信息接口 ===\n");
    
    let user_info_url = "https://www.daydaymap.com/api/v1/user/info";
    
    let client = Client::new();
    let response = client.get(user_info_url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    
    println!("HTTP状态码: {}", response.status());
    
    let response_text = response.text().await?;
    println!("\n原始响应:");
    println!("{}", response_text);
    
    println!("\n格式化响应:");
    if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    }
    
    Ok(())
}
