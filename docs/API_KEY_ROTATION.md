# DayDayMap API Key 轮询机制

## 功能概述

当配置了多个 DayDayMap API Key 时，系统会自动实现 key 轮询机制。当一个 key 的额度耗尽或失效时，自动切换到下一个可用的 key，确保查询和导出操作能够持续进行。

## 使用场景

### 场景 1：多 Key 配置

```json
{
  "api_keys": [
    "c5661493dbcf42d8aa4cf5289d92c772",  // Key 1
    "a1234567890abcdef1234567890abcde",  // Key 2
    "b9876543210fedcba9876543210fedcb",  // Key 3
    "d1111111111111111111111111111111"   // Key 4
  ]
}
```

### 场景 2：导出大量数据

```
用户请求：导出 50 页数据，每页 100 条（共 5000 条）

执行过程：
- 第 1-10 页：使用 Key 1 成功
- 第 11 页：Key 1 额度耗尽（错误码 2004）
- 自动切换到 Key 2
- 第 11-25 页：使用 Key 2 成功
- 第 26 页：Key 2 额度耗尽
- 自动切换到 Key 3
- 第 26-40 页：使用 Key 3 成功
- 第 41 页：Key 3 额度耗尽
- 自动切换到 Key 4
- 第 41-50 页：使用 Key 4 成功

结果：成功导出全部 5000 条数据
```

## 工作原理

### 1. Key 轮询逻辑

```rust
async fn search_with_key_rotation(
    query: &str, 
    page: u32, 
    page_size: u32,
    excluded_keys: Option<Vec<String>>
) -> Result<Value, String> {
    // 获取所有可用的 API keys
    let all_keys = config::get_all_daydaymap_api_keys()?;
    
    // 过滤掉已经失败的 keys
    let available_keys: Vec<String> = all_keys.into_iter()
        .filter(|k| !excluded.contains(k))
        .collect();
    
    // 尝试每个可用的 key
    for (index, api_key) in available_keys.iter().enumerate() {
        match try_search_with_key(query, page, page_size, api_key).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // 检查错误类型
                if is_quota_exhausted(&e) || is_key_invalid(&e) {
                    continue; // 尝试下一个 key
                } else {
                    return Err(e); // 其他错误，停止尝试
                }
            }
        }
    }
    
    Err("所有 API Key 都无法使用".to_string())
}
```

### 2. 错误分类

**会触发 Key 切换的错误：**
- `积分不足` / `额度不足` (错误码 2004)
- `API密钥无效` (错误码 401)
- `未授权`

**不会触发 Key 切换的错误：**
- 网络错误
- 查询语法错误
- 服务器错误（5xx）

### 3. 日志输出

```
=== DayDayMap search 函数 ===
查询字符串: ip:"183.201.199.0/24"
页码: 11
每页数量: 100
可用 API Key 数量: 4

尝试使用第 1 个 API Key: c5661493...
✗ API Key 1 查询失败: API返回错误: 积分不足,请联系管理员
  检测到额度耗尽，尝试下一个 key...

尝试使用第 2 个 API Key: a1234567...
✓ API Key 2 查询成功
```

## 配置方法

### 1. 通过前端界面添加

1. 打开应用
2. 进入"API密钥管理"页面
3. 选择 DayDayMap 平台
4. 依次添加多个 API Key

### 2. 手动编辑配置文件

配置文件位置：`~/Library/Application Support/asset-mapping/daydaymap_api.json`

```json
{
  "api_keys": [
    "your_first_key_here",
    "your_second_key_here",
    "your_third_key_here",
    "your_fourth_key_here"
  ]
}
```

### 3. 验证配置

```bash
# 查看当前配置的 keys
cat ~/Library/Application\ Support/asset-mapping/daydaymap_api.json

# 应该看到类似输出：
# {
#   "api_keys": [
#     "c5661493dbcf42d8aa4cf5289d92c772",
#     "a1234567890abcdef1234567890abcde",
#     "b9876543210fedcba9876543210fedcb",
#     "d1111111111111111111111111111111"
#   ]
# }
```

## 使用示例

### 示例 1：普通查询

```
用户操作：
1. 输入查询：ip:"183.201.199.0/24"
2. 点击查询

系统行为：
- 尝试使用 Key 1
- 如果 Key 1 额度耗尽，自动切换到 Key 2
- 返回查询结果

用户体验：
- 无感知切换
- 查询成功
```

### 示例 2：大批量导出

```
用户操作：
1. 输入查询：ip:"183.201.199.0/24"
2. 点击导出
3. 设置导出 50 页，每页 100 条

系统行为：
- 第 1-10 页：使用 Key 1
- 第 11 页：Key 1 额度耗尽，切换到 Key 2
- 第 11-25 页：使用 Key 2
- 第 26 页：Key 2 额度耗尽，切换到 Key 3
- 第 26-40 页：使用 Key 3
- 第 41 页：Key 3 额度耗尽，切换到 Key 4
- 第 41-50 页：使用 Key 4
- 导出完成

日志输出：
正在导出第 11/50 页...
尝试使用第 1 个 API Key: c5661493...
✗ API Key 1 查询失败: 积分不足
  检测到额度耗尽，尝试下一个 key...
尝试使用第 2 个 API Key: a1234567...
✓ API Key 2 查询成功
第 11 页成功: 获取 100 条数据
```

### 示例 3：所有 Key 都耗尽

```
用户操作：
1. 尝试查询或导出

系统行为：
- 尝试 Key 1：额度耗尽
- 尝试 Key 2：额度耗尽
- 尝试 Key 3：额度耗尽
- 尝试 Key 4：额度耗尽
- 返回错误

错误信息：
"所有 API Key 都无法使用。最后错误: 积分不足,请联系管理员"

建议：
- 充值现有 Key
- 添加新的 Key
- 等待额度重置（如果是按时间重置的额度）
```

## 优势

1. **无缝切换**：用户无需手动切换 key，系统自动处理
2. **提高可用性**：单个 key 额度耗尽不影响整体使用
3. **大批量导出**：支持导出超过单个 key 额度限制的数据
4. **智能重试**：只对额度/权限错误进行 key 切换，其他错误直接返回
5. **详细日志**：清楚记录每个 key 的使用情况

## 注意事项

### 1. Key 顺序

- Keys 按照配置文件中的顺序使用
- 建议将额度最多的 key 放在前面

### 2. 并发请求

- 当前实现是顺序尝试 keys
- 如果有并发请求，可能会同时使用多个 keys

### 3. Key 失效

- 如果某个 key 永久失效（如被封禁），建议从配置中删除
- 系统会自动跳过失效的 key，但会增加请求延迟

### 4. 额度管理

- 建议定期检查各个 key 的剩余额度
- 可以通过"API密钥管理"页面验证 key 状态

## 与重试机制的配合

Key 轮询机制与重试机制是互补的：

1. **重试机制**：处理临时性错误（网络故障、限流）
   - 同一个 key 重试 3 次
   - 每次重试间隔 5 秒

2. **Key 轮询**：处理额度耗尽
   - 检测到额度耗尽立即切换 key
   - 不进行重试

**组合效果：**
```
第 11 页查询：
1. 使用 Key 1 尝试
2. 返回"积分不足"
3. 不重试，直接切换到 Key 2
4. 使用 Key 2 尝试
5. 如果网络超时，重试 3 次
6. 成功返回结果
```

## 测试验证

### 测试用例 1：单 Key 额度耗尽

```bash
# 配置 2 个 keys，第一个额度耗尽
# 预期：自动切换到第二个 key，查询成功
```

### 测试用例 2：多 Key 轮询

```bash
# 配置 4 个 keys
# 导出 50 页数据
# 预期：依次使用 4 个 keys，全部导出成功
```

### 测试用例 3：所有 Key 耗尽

```bash
# 配置 2 个 keys，都已额度耗尽
# 预期：返回错误"所有 API Key 都无法使用"
```

## 代码结构

```
src/api/daydaymap.rs
├── search()                      // 公开接口
├── search_with_key_rotation()    // Key 轮询逻辑
└── try_search_with_key()         // 使用指定 key 查询

src/config/mod.rs
├── get_daydaymap_api_key()       // 获取第一个 key（兼容旧代码）
└── get_all_daydaymap_api_keys()  // 获取所有 keys（新增）
```

## 未来优化

1. **智能 Key 选择**
   - 记录每个 key 的剩余额度
   - 优先使用额度最多的 key

2. **Key 状态缓存**
   - 缓存已知失效的 keys
   - 避免重复尝试失效的 keys

3. **并发优化**
   - 支持多个请求并发使用不同的 keys
   - 实现 key 池管理

4. **额度监控**
   - 实时显示各个 key 的剩余额度
   - 额度不足时提前预警

## 总结

API Key 轮询机制大大提高了 DayDayMap 平台的可用性和数据导出能力。通过配置多个 API Keys，用户可以：

- 导出超过单个 key 额度限制的大量数据
- 避免因单个 key 失效而中断操作
- 实现无缝的 key 切换，提升用户体验

建议用户配置至少 2-3 个 API Keys 以获得最佳使用体验。
