# Hunter API 验证功能修复报告

## 问题描述

用户在UI中验证Hunter API密钥时遇到"API密钥无效: API返回错误状态码: 404 Not Found"错误。

## 问题原因

经过调查发现，Hunter API（hunter.qianxin.com）**没有单独的用户信息接口** (`/openApi/user`)。之前的实现尝试访问这个不存在的端点，导致404错误。

### 网络问题

测试过程中还发现了网络连接问题：
- DNS解析返回正确的IP地址：`123.6.81.42`
- 但实际连接时被重定向到：`198.18.0.56`（测试网络地址段）
- 这表明存在代理、VPN或网络过滤

使用 `--resolve` 参数强制使用正确的IP后，连接成功但返回404。

## 解决方案

### 修改验证方法

Hunter API的配额信息直接包含在搜索接口的响应中，因此我们改用搜索接口来验证API密钥：

1. **使用搜索接口验证**：发送一个简单的测试查询（`domain="test.com"`）
2. **提取配额信息**：从响应中获取 `rest_quota` 和 `consume_quota`
3. **验证成功标准**：HTTP状态码200 + 业务状态码200

### 代码修改

#### hunter.rs - validate_api_key 函数

```rust
pub async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    // 使用搜索接口验证（而不是不存在的 /openApi/user）
    let base_url = "https://hunter.qianxin.com/openApi/search";
    
    // 使用简单的测试查询
    let test_query = "domain=\"test.com\"";
    let encoded_query = general_purpose::URL_SAFE.encode(test_query.as_bytes());
    
    let params = [
        ("api-key", api_key),
        ("search", &encoded_query),
        ("page", "1"),
        ("page_size", "1"),
        ("is_web", "3"),
    ];
    
    // 发送请求并解析响应...
}
```

## 测试结果

### 成功的验证测试

```
【测试1】验证API密钥
测试查询: domain="test.com"
响应状态码: 200 OK
业务状态码: 200
✓ API密钥有效
剩余积分: "今日剩余积分：498"
消耗积分: "消耗积分：1"
账户类型: "个人账号"
```

### 成功的查询测试

```
【测试2】基础查询
查询语法: domain="baidu.com"
响应状态码: 200 OK
业务状态码: 200
✓ 查询成功
总结果数: 59788
返回结果数: 10
消耗积分: 消耗积分：9
剩余积分: 今日剩余积分：489
```

### 速率限制测试

```
【测试3】带状态码过滤的查询
业务状态码: 429
✗ 查询失败
错误信息: 请求太多啦，稍后再试试
```

这是正常的速率限制，说明API正常工作。

### 时间范围限制

```
【测试4】带时间范围的查询
业务状态码: 400
✗ 查询失败
错误信息: 当前时间范围超出近30天，查看或导出资产都将扣除权益积分
```

这说明时间范围查询有30天的限制（个人账号）。

## API响应格式

### 成功响应

```json
{
  "code": 200,
  "message": "success",
  "data": {
    "account_type": "个人账号",
    "total": 59788,
    "time": 7,
    "arr": [...],
    "consume_quota": "消耗积分：9",
    "rest_quota": "今日剩余积分：489",
    "syntax_prompt": ""
  }
}
```

### 关键字段

- `code`: 业务状态码（200=成功）
- `data.account_type`: 账户类型（个人账号/企业账号）
- `data.rest_quota`: 今日剩余积分
- `data.consume_quota`: 本次消耗积分
- `data.total`: 查询结果总数
- `data.arr`: 结果数组

## 配额消耗

- **验证API密钥**：消耗1积分
- **基础查询**：消耗9积分（取决于查询复杂度）
- **速率限制**：请求过快会触发429错误

## 注意事项

### 1. 没有独立的用户信息接口

Hunter API不提供 `/openApi/user` 端点，所有信息都通过搜索接口返回。

### 2. 验证会消耗积分

每次验证API密钥都会消耗1积分，因为实际上是执行了一次搜索。

### 3. 速率限制

频繁请求会触发速率限制（429错误），建议：
- 添加请求间隔（2秒）
- 使用智能密钥轮询
- 缓存验证结果

### 4. 时间范围限制

个人账号查询时间范围限制在近30天内，超出范围需要额外权益积分。

### 5. 网络问题

如果遇到连接问题，可能是：
- 代理或VPN干扰
- DNS解析问题
- 防火墙限制

## 改进建议

### 1. 缓存验证结果

```rust
// 缓存验证结果，避免频繁消耗积分
struct ValidationCache {
    api_key: String,
    valid: bool,
    quota: String,
    timestamp: SystemTime,
}
```

### 2. 前端优化

- 添加"验证中"状态提示
- 显示剩余积分
- 显示账户类型
- 添加验证间隔限制（避免频繁点击）

### 3. 错误处理

- 区分网络错误和API错误
- 提供更友好的错误提示
- 添加重试机制（针对网络错误）

### 4. 配额监控

- 实时显示剩余积分
- 积分不足时提醒用户
- 记录积分消耗历史

## 相关文件

- `asset-mapping/src-tauri/src/api/hunter.rs` - Hunter API实现
- `asset-mapping/src-tauri/examples/test_hunter_api.rs` - 测试程序
- `asset-mapping/test_hunter_api.sh` - 测试脚本
- `asset-mapping/HUNTER_API_TESTING.md` - 测试文档
- `asset-mapping/HUNTER_API_ENHANCEMENT.md` - API增强文档

## 总结

通过改用搜索接口进行API密钥验证，成功解决了404错误问题。新的验证方法：

✅ 使用实际存在的API端点  
✅ 能够获取配额信息  
✅ 能够获取账户类型  
✅ 验证结果准确可靠  
⚠️ 每次验证消耗1积分  
⚠️ 需要注意速率限制  

建议在前端添加验证结果缓存，避免频繁验证消耗积分。
