# Hunter API 测试和验证指南

## 最新更新（2026-02-05）

### ✅ 修复了404错误问题

之前的实现尝试访问不存在的 `/openApi/user` 端点，导致404错误。现已修复：

- **问题**：Hunter API没有单独的用户信息接口
- **解决方案**：改用搜索接口进行API密钥验证
- **影响**：每次验证会消耗1积分（执行一次简单查询）
- **详细信息**：参见 `HUNTER_API_VALIDATION_FIX.md`

## 概述
本文档说明如何测试和验证Hunter API的功能，包括API密钥验证、查询接口测试等。

## 测试程序

### 1. 测试文件位置
- 测试程序：`src-tauri/examples/test_hunter_api.rs`
- 测试脚本：`test_hunter_api.sh`

### 2. 测试内容
测试程序包含以下测试：

1. **API密钥验证** - 验证API密钥是否有效
2. **基础查询** - 测试基本的搜索功能
3. **状态码过滤** - 测试带状态码过滤的查询
4. **时间范围查询** - 测试带时间范围的查询

## 使用方法

### 方法1: 使用测试脚本（推荐）

```bash
# 设置API密钥环境变量
export HUNTER_API_KEY=your_api_key_here

# 运行测试脚本
cd asset-mapping
./test_hunter_api.sh
```

或者一行命令：
```bash
HUNTER_API_KEY=your_api_key_here ./test_hunter_api.sh
```

### 方法2: 直接运行测试程序

```bash
cd asset-mapping/src-tauri

# 编译测试程序
cargo build --example test_hunter_api

# 运行测试
HUNTER_API_KEY=your_api_key_here cargo run --example test_hunter_api
```

## 测试输出示例

### 成功的输出
```
=== Hunter API 测试 ===

API Key: b681bc6e...

【测试1】验证API密钥
测试查询: domain="test.com"
响应状态码: 200 OK
业务状态码: 200
✓ API密钥有效
剩余积分: "今日剩余积分：498"
消耗积分: "消耗积分：1"
账户类型: "个人账号"

【测试2】基础查询
查询语法: domain="baidu.com"
Base64编码: ZG9tYWluPSJiYWlkdS5jb20i
响应状态码: 200 OK
业务状态码: 200
✓ 查询成功
总结果数: 59788
返回结果数: 10
消耗积分: 消耗积分：9
剩余积分: 今日剩余积分：489

【测试3】带状态码过滤的查询
响应状态码: 200 OK
业务状态码: 429
✗ 查询失败
错误信息: 请求太多啦，稍后再试试
（这是正常的速率限制）

【测试4】带时间范围的查询
响应状态码: 200 OK
业务状态码: 400
✗ 查询失败
错误信息: 当前时间范围超出近30天，查看或导出资产都将扣除权益积分
（个人账号时间范围限制在30天内）
```

### 失败的输出（API密钥无效）
```
=== Hunter API 测试 ===

API Key: test_key

【测试1】验证API密钥
=== Hunter API密钥验证 ===
API Key: test_key
请求URL: https://hunter.qianxin.com/openApi/user
HTTP状态码: 401 Unauthorized
✗ API返回错误状态码: 401 Unauthorized
错误响应: {"code":401,"msg":"API密钥无效"}
```

## API密钥验证改进

### 改进内容

**重要提示**：验证API密钥会消耗1积分，因为实际上是执行了一次搜索查询。

1. **使用搜索接口验证**
   - Hunter API没有单独的用户信息接口
   - 使用简单的测试查询：`domain="test.com"`
   - 从搜索响应中提取配额信息
   - 输出请求URL
   - 输出HTTP状态码
   - 输出响应内容
   - 输出业务状态码

2. **详细日志输出**
   - 支持 `rest_quota` 和 `restQuota` 字段
   - 支持字符串和数字类型的配额值
   - 提取用户名信息（如果有）

3. **配额信息提取**
   - 提取 `rest_quota`（剩余积分）
   - 提取 `consume_quota`（消耗积分）
   - 提取 `account_type`（账户类型）

4. **多字段支持**
   - 网络错误详细提示
   - HTTP错误状态码处理
   - JSON解析错误处理
   - 业务错误码处理

5. **错误处理增强**
   - 使用 `eprintln!` 输出调试信息
   - 不影响正常的返回值
   - 便于排查问题

6. **调试信息**

### 验证函数代码示例

**注意**：新的验证方法使用搜索接口，而不是不存在的用户信息接口。

```rust
pub async fn validate_api_key(api_key: &str) -> Result<ApiKeyValidationResult, String> {
    eprintln!("=== Hunter API密钥验证 ===");
    eprintln!("API Key: {}...", &api_key[..8.min(api_key.len())]);
    
    // 使用搜索接口验证（Hunter API没有单独的用户信息接口）
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
    
    eprintln!("请求URL: {}", base_url);
    eprintln!("测试查询: {}", test_query);
    
    let client = Client::new();
    let response = client.get(base_url)
        .query(&params)
        .send()
        .await
        .map_err(|e| {
            eprintln!("✗ 网络请求失败: {}", e);
            format!("请求失败: {}", e)
        })?;
    
    let status = response.status();
    eprintln!("HTTP状态码: {}", status);
    
    // ... 更多处理逻辑
}
```

## 常见问题

### 1. API密钥无效（401错误）

**问题**: 返回401 Unauthorized错误

**原因**:
- API密钥错误或已过期
- API密钥格式不正确

**解决方法**:
1. 检查API密钥是否正确
2. 登录Hunter平台重新获取API密钥
3. 确认API密钥没有过期

### 2. 404 Not Found错误（已修复）

**问题**: 返回404 Not Found错误

**原因**:
- ~~之前的代码尝试访问不存在的 `/openApi/user` 端点~~
- Hunter API没有单独的用户信息接口

**解决方法**:
- ✅ 已修复：现在使用搜索接口进行API密钥验证
- 配额信息直接从搜索响应中获取
- 详见：`HUNTER_API_VALIDATION_FIX.md`

### 3. 配额耗尽

**问题**: 返回"积分用完"或"次牛"错误

**原因**:
- 当前API密钥的积分已用完

**解决方法**:
1. 等待第二天自动重置（如果是每日配额）
2. 购买更多积分
3. 使用其他API密钥（系统会自动轮询）

### 4. 查询语法错误

**问题**: 返回查询语法错误

**原因**:
- 查询语法不符合Hunter规范
- Base64编码错误

**解决方法**:
1. 检查查询语法是否正确
2. 确认使用正确的Base64编码（URL Safe）
3. 参考Hunter官方文档的查询语法

## 查询语法参考

### 基本语法
```
field="value"
```

### 常用字段
- `domain` - 域名
- `ip` - IP地址
- `port` - 端口
- `web.title` - 网页标题
- `web.body` - 网页内容
- `status_code` - 状态码
- `protocol` - 协议
- `os` - 操作系统
- `company` - 公司

### 示例
```
domain="baidu.com"
ip="1.1.1.1"
port="80"
web.body="login"
status_code="200"
```

### 组合查询
```
domain="baidu.com" && port="80"
ip="1.1.1.1" || ip="2.2.2.2"
```

## API端点

### 重要说明
Hunter API **没有单独的用户信息接口**。配额信息直接包含在搜索接口的响应中。

### 搜索接口（也用于验证API密钥）
- **URL**: `https://hunter.qianxin.com/openApi/search`
- **方法**: GET
- **参数**:
  - `api-key` (必需) - API密钥
  - `search` (必需) - Base64编码的查询语法
  - `page` (必需) - 页码
  - `page_size` (必需) - 每页数量
  - `is_web` (必需) - 资产类型（1=web, 2=非web, 3=全部）
  - `status_code` (可选) - 状态码过滤
  - `start_time` (可选) - 开始时间
  - `end_time` (可选) - 结束时间

## 配置API密钥

### 方法1: 环境变量
```bash
export HUNTER_API_KEY=your_api_key_here
```

### 方法2: 配置文件
配置文件位置：`~/Library/Application Support/asset-mapping/hunter_api.json`

```json
{
  "api_keys": [
    "your_first_api_key",
    "your_second_api_key",
    "your_third_api_key"
  ]
}
```

## 调试技巧

### 1. 查看详细日志
测试程序会输出详细的调试信息到stderr，包括：
- 请求URL
- 请求参数
- HTTP状态码
- 响应内容
- 业务状态码

### 2. 使用curl测试
```bash
# 测试API密钥验证（使用搜索接口）
curl "https://hunter.qianxin.com/openApi/search?api-key=YOUR_KEY&search=ZG9tYWluPSJ0ZXN0LmNvbSI=&page=1&page_size=1&is_web=3"

# 测试基础查询
curl "https://hunter.qianxin.com/openApi/search?api-key=YOUR_KEY&search=ZG9tYWluPSJiYWlkdS5jb20i&page=1&page_size=10&is_web=3"
```

注意：
- `ZG9tYWluPSJ0ZXN0LmNvbSI=` 是 `domain="test.com"` 的Base64编码
- `ZG9tYWluPSJiYWlkdS5jb20i` 是 `domain="baidu.com"` 的Base64编码

### 3. Base64编码工具
```bash
# 编码查询语法
echo -n 'domain="baidu.com"' | base64

# 解码Base64
echo 'ZG9tYWluPSJiYWlkdS5jb20i' | base64 -d
```

## 下一步

1. **前端集成**: 在前端界面添加API密钥验证功能
2. **错误提示**: 改进前端的错误提示信息
3. **配额显示**: 在前端显示剩余配额
4. **自动测试**: 添加自动化测试脚本

## 相关文档

- [Hunter API功能完善报告](./HUNTER_API_ENHANCEMENT.md)
- [批量API密钥管理功能](./BATCH_API_KEY_FEATURE.md)
- [智能密钥管理系统](./SMART_KEY_MANAGEMENT.md)
