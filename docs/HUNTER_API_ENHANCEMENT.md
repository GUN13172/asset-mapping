# Hunter API 功能完善报告

## 概述
根据Hunter（鹰图）平台的官方API文档，完善了Hunter API的实现，增加了更多参数支持和返回字段。

## API文档参考
- 官方文档：https://hunter.qianxin.com/home/helpCenter?r=5-1-2
- API端点：https://hunter.qianxin.com/openApi/search

## 新增功能

### 1. 新增查询参数

#### status_code（状态码过滤）
- **类型**: 可选参数
- **格式**: 字符串，多个状态码用逗号分隔
- **示例**: `"200,401,403"`
- **用途**: 过滤特定HTTP状态码的资产

#### start_time（开始时间）
- **类型**: 可选参数
- **格式**: `YYYY-MM-DD HH:MM:SS`
- **示例**: `"2022-06-19 00:00:00"`
- **用途**: 查询指定时间范围内的资产

#### end_time（结束时间）
- **类型**: 可选参数
- **格式**: `YYYY-MM-DD HH:MM:SS`
- **示例**: `"2022-07-18 23:59:59"`
- **用途**: 查询指定时间范围内的资产

### 2. 新增返回字段

#### 基础信息
- `domain` - 域名
- `status_code` - HTTP状态码
- `protocol` - 协议（http/https）
- `base_protocol` - 基础协议（tcp/udp）

#### 系统信息
- `os` - 操作系统
- `component` - 组件信息（格式：`nginx:1.6, php:7.4`）

#### 组织信息
- `company` - 公司名称
- `number` - 备案号
- `isp` - 运营商信息
- `as_org` - AS组织

#### 时间信息
- `updated_at` - 更新时间

#### 配额信息
- `consume_quota` - 本次消耗积分
- `rest_quota` - 今日剩余积分

## API使用示例

### 1. 基础查询
```rust
// 基础查询（不带可选参数）
let result = hunter::search("domain:example.com", 1, 100).await?;
```

### 2. 带状态码过滤
```rust
// 只查询状态码为200的资产
let result = hunter::search_with_options(
    "domain:example.com",
    1,
    100,
    Some("200"),
    None,
    None
).await?;
```

### 3. 带时间范围查询
```rust
// 查询指定时间范围内的资产
let result = hunter::search_with_options(
    "domain:example.com",
    1,
    100,
    None,
    Some("2022-06-19 00:00:00"),
    Some("2022-07-18 23:59:59")
).await?;
```

### 4. 完整参数查询
```rust
// 使用所有可选参数
let result = hunter::search_with_options(
    "domain:example.com",
    1,
    100,
    Some("200,401"),
    Some("2022-06-19 00:00:00"),
    Some("2022-07-18 23:59:59")
).await?;
```

## 返回数据示例

### JSON格式
```json
{
  "total": 1,
  "consume_quota": "消耗积分：20",
  "rest_quota": "今日剩余积分：77",
  "results": [
    {
      "url": "http://example.com",
      "ip": "127.0.0.1",
      "port": "80",
      "domain": "example.com",
      "web_title": "Example Domain",
      "status_code": 200,
      "country": "中国",
      "province": "北京",
      "city": "北京",
      "server": "nginx/1.6",
      "protocol": "http",
      "base_protocol": "tcp",
      "os": "linux",
      "company": "北京xxx公司",
      "number": "京ICP备12345678号",
      "isp": "中国电信",
      "as_org": "CHINANET",
      "component": "nginx:1.6, php:7.4",
      "updated_at": "2021-01-01 00:00:00"
    }
  ]
}
```

## CSV导出字段

导出的CSV文件现在包含以下字段（按顺序）：

1. IP
2. 端口
3. 域名
4. 标题
5. 状态码
6. 服务器
7. 协议
8. 基础协议
9. 操作系统
10. 国家
11. 省份
12. 城市
13. 公司
14. 备案号
15. ISP
16. AS组织
17. 组件
18. 更新时间
19. URL

### CSV示例
```csv
IP,端口,域名,标题,状态码,服务器,协议,基础协议,操作系统,国家,省份,城市,公司,备案号,ISP,AS组织,组件,更新时间,URL
127.0.0.1,80,example.com,Example Domain,200,nginx/1.6,http,tcp,linux,中国,北京,北京,北京xxx公司,京ICP备12345678号,中国电信,CHINANET,nginx:1.6 php:7.4,2021-01-01 00:00:00,http://example.com
```

## 组件信息处理

组件信息（component）是一个数组，包含多个组件及其版本：

### API返回格式
```json
"component": [
  {
    "name": "nginx",
    "version": "1.6"
  },
  {
    "name": "php",
    "version": "7.4"
  }
]
```

### 处理后格式
```
nginx:1.6, php:7.4
```

## 配额信息

每次查询都会返回配额信息：

- `consume_quota`: 本次查询消耗的积分
- `rest_quota`: 今日剩余积分

### 示例
```
消耗积分：20
今日剩余积分：77
```

## API参数说明

### is_web（资产类型）
- `1` - 仅web资产
- `2` - 非web资产
- `3` - 全部资产（默认）

### search（查询语法）
查询字符串需要进行Base64 URL编码（符合RFC 4648标准）

#### 查询语法示例
- `domain="example.com"` - 查询域名
- `ip="1.1.1.1"` - 查询IP
- `web.body="keyword"` - 查询网页内容
- `port="80"` - 查询端口
- `status_code="200"` - 查询状态码

## 错误处理

### 配额耗尽
当API返回配额耗尽错误时，系统会：
1. 自动标记当前密钥为已耗尽
2. 切换到下一个可用密钥
3. 如果所有密钥都耗尽，保存已导出的数据

### 错误码识别
- 消息包含"积分用完"
- 消息包含"次牛"
- 消息包含"quota"

## 兼容性

### 向后兼容
原有的 `search()` 函数保持不变，不带可选参数：
```rust
pub async fn search(query: &str, page: u32, page_size: u32) -> Result<Value, String>
```

### 新增函数
新增带可选参数的函数：
```rust
pub async fn search_with_options(
    query: &str, 
    page: u32, 
    page_size: u32,
    status_code: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Value, String>
```

## 实现细节

### 1. 参数构建
```rust
let mut params = vec![
    ("api-key", api_key.to_string()),
    ("search", encoded_query),
    ("page", page.to_string()),
    ("page_size", page_size.to_string()),
    ("is_web", "3".to_string()),
];

// 添加可选参数
if let Some(code) = status_code {
    params.push(("status_code", code.to_string()));
}
if let Some(start) = start_time {
    params.push(("start_time", start.to_string()));
}
if let Some(end) = end_time {
    params.push(("end_time", end.to_string()));
}
```

### 2. 组件信息提取
```rust
let components = if let Some(comp_array) = item["component"].as_array() {
    comp_array.iter()
        .filter_map(|c| {
            let name = c["name"].as_str().unwrap_or("");
            let version = c["version"].as_str().unwrap_or("");
            if !name.is_empty() {
                if !version.is_empty() {
                    Some(format!("{}:{}", name, version))
                } else {
                    Some(name.to_string())
                }
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join(", ")
} else {
    String::new()
};
```

### 3. Banner字段处理
注意：API返回的 `banner` 字段是字符串，不是对象：
```rust
// 正确的方式
"server": item["banner"].as_str().unwrap_or("")

// 错误的方式（旧代码）
"server": item["banner"]["server"].as_str().unwrap_or("")
```

## 测试建议

### 1. 基础查询测试
```bash
# 测试基础查询
curl "https://hunter.qianxin.com/openApi/search?api-key=YOUR_KEY&search=ZG9tYWluPSJleGFtcGxlLmNvbSI=&page=1&page_size=10&is_web=3"
```

### 2. 状态码过滤测试
```bash
# 测试状态码过滤
curl "https://hunter.qianxin.com/openApi/search?api-key=YOUR_KEY&search=ZG9tYWluPSJleGFtcGxlLmNvbSI=&page=1&page_size=10&is_web=3&status_code=200"
```

### 3. 时间范围测试
```bash
# 测试时间范围
curl "https://hunter.qianxin.com/openApi/search?api-key=YOUR_KEY&search=ZG9tYWluPSJleGFtcGxlLmNvbSI=&page=1&page_size=10&is_web=3&start_time=2022-06-19+00:00:00&end_time=2022-07-18+23:59:59"
```

## 改进总结

### 新增功能
- ✅ 支持状态码过滤
- ✅ 支持时间范围查询
- ✅ 返回更多字段（组件、操作系统、公司、备案号等）
- ✅ 返回配额信息
- ✅ CSV导出包含所有新字段

### 优化改进
- ✅ 修复banner字段解析错误
- ✅ 添加组件信息格式化
- ✅ 保持向后兼容性
- ✅ 完善错误处理

### 文档完善
- ✅ API参数说明
- ✅ 返回字段说明
- ✅ 使用示例
- ✅ 测试建议

## 注意事项

1. **Base64编码**: 查询字符串必须使用Base64 URL编码（RFC 4648标准）
2. **时间格式**: 时间参数格式为 `YYYY-MM-DD HH:MM:SS`
3. **状态码格式**: 多个状态码用逗号分隔，如 `"200,401,403"`
4. **配额消耗**: 每次查询会消耗一定积分，注意查看返回的配额信息
5. **CSV逗号处理**: 包含逗号的字段会自动替换为中文逗号（，）

## 未来改进建议

1. **前端支持**: 在前端界面添加状态码和时间范围过滤选项
2. **配额显示**: 在前端显示剩余配额信息
3. **组件搜索**: 支持按组件名称和版本搜索
4. **高级过滤**: 支持更多高级过滤选项（如按ISP、AS组织等）
