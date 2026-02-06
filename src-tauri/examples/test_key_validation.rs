// 测试修复后的API密钥验证功能
// 使用搜索接口来验证密钥是否有效

use asset_mapping::api::daydaymap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "c5661493dbcf42d8aa4cf5289d92c772";
    
    println!("=== 测试修复后的API密钥验证 ===\n");
    println!("使用搜索接口验证密钥 (查询: ip=\"1.1.1.1\")");
    println!("如果能成功查询到数据，则证明密钥有效\n");
    
    match daydaymap::validate_api_key(api_key).await {
        Ok(result) => {
            println!("验证结果:");
            println!("  - 有效: {}", result.valid);
            if let Some(msg) = result.message {
                println!("  - 消息: {}", msg);
            }
            if let Some(quota) = result.quota {
                println!("  - 信息: {}", quota);
            }
            
            if result.valid {
                println!("\n✓ API密钥验证成功！");
            } else {
                println!("\n✗ API密钥验证失败");
            }
        }
        Err(e) => {
            println!("✗ 验证过程出错: {}", e);
        }
    }
    
    println!("\n=== 测试完成 ===");
    Ok(())
}
