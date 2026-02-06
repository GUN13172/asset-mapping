# 批量API密钥管理功能完成报告

## 概述
成功为所有平台（Hunter、FOFA、Quake、DayDayMap）实现了智能API密钥管理系统，包括自动轮询、配额耗尽检测、重试机制和部分导出功能。

## 实现的功能

### 1. 智能密钥管理系统 (key_manager.rs)

#### 核心功能
- **自动轮询**: 当一个API密钥配额耗尽时，自动切换到下一个可用密钥
- **状态持久化**: 将密钥状态保存到文件，重启应用后状态保持
- **游标机制**: 记住当前使用的密钥索引，避免重复尝试已耗尽的密钥
- **午夜重置**: 每天0点自动重置所有密钥状态
- **平台独立**: 支持多个平台（Hunter、FOFA、Quake、DayDayMap）

#### 状态文件位置
```
~/Library/Application Support/asset-mapping/
├── hunter_key_state.json
├── fofa_key_state.json
├── quake_key_state.json
└── daydaymap_key_state.json
```

#### 状态文件格式
```json
{
  "current_index": 0,
  "keys": [
    {
      "key": "your-api-key-1",
      "is_exhausted": false,
      "exhausted_at": null,
      "last_used_at": "2026-02-05T10:30:45+08:00"
    },
    {
      "key": "your-api-key-2",
      "is_exhausted": true,
      "exhausted_at": "2026-02-05T09:15:30+08:00",
      "last_used_at": "2026-02-05T09:15:30+08:00"
    }
  ],
  "last_reset_date": "2026-02-05"
}
```

### 2. 配额耗尽检测

#### 各平台错误码识别
- **DayDayMap**: 错误码 2004，消息包含"积分不足"
- **Hunter**: 消息包含"积分用完"、"次牛"、"quota"
- **FOFA**: 消息包含"F币"、"quota"
- **Quake**: 消息包含"积分"、"quota"

#### 智能错误处理
- **配额耗尽**: 标记密钥为已耗尽，自动尝试下一个密钥
- **网络错误**: 重试当前请求（最多3次）
- **其他错误**: 直接返回错误，不尝试其他密钥

### 3. 重试机制

#### 重试策略
- **最大重试次数**: 3次
- **重试延迟**: 5秒
- **重试条件**: 仅对网络错误重试，配额耗尽不重试

#### 实现位置
- `hunter.rs`: export() 函数
- `fofa.rs`: export() 函数
- `quake.rs`: export() 函数
- `daydaymap.rs`: export() 函数（已有）

### 4. 部分导出功能

#### 功能说明
当导出过程中遇到错误（配额耗尽或网络错误），自动保存已成功导出的数据。

#### 文件命名规则
- **完整导出**: `{platform}_export_{timestamp}.csv`
- **部分导出**: `{platform}_export_{timestamp}_partial_{成功页数}of{总页数}_pages.csv`

#### 示例
```
hunter_export_20260205_103045.csv                    # 完整导出
hunter_export_20260205_103045_partial_4of10_pages.csv  # 部分导出（成功4页，共10页）
```

### 5. 配置文件更新 (config/mod.rs)

#### 新增函数
```rust
// 获取所有Hunter API密钥（用于轮询）
pub fn get_all_hunter_api_keys() -> Result<Vec<String>, String>

// 获取所有FOFA API密钥（用于轮询）
pub fn get_all_fofa_api_keys() -> Result<Vec<(String, String)>, String>

// 获取所有Quake API密钥（用于轮询）
pub fn get_all_quake_api_keys() -> Result<Vec<String>, String>

// 获取所有DayDayMap API密钥（用于轮询）
pub fn get_all_daydaymap_api_keys() -> Result<Vec<String>, String>
```

## 技术实现细节

### 1. 异步闭包处理
使用 `execute_with_key_rotation` 函数统一处理密钥轮询逻辑：

```rust
let result = key_manager::execute_with_key_rotation(
    "hunter",
    &api_keys,
    |api_key| {
        let query = query.clone();
        let api_key = api_key.to_string();
        async move {
            search_with_key(&api_key, &query, page, page_size).await
        }
    }
).await;
```

### 2. FOFA特殊处理
FOFA需要同时提供API密钥和邮箱，使用 `key:email` 格式存储：

```rust
let api_keys: Vec<String> = api_key_pairs.iter()
    .map(|(key, email)| format!("{}:{}", key, email))
    .collect();
```

### 3. 生命周期管理
通过克隆数据解决闭包生命周期问题：

```rust
let query = query.to_string();  // 克隆数据
async move {
    search_with_key(&api_key, &query, page, page_size).await
}
```

## 性能优化

### 1. 减少API调用
- **优化前**: 每次请求尝试所有已耗尽的密钥（4个密钥 = 4次API调用）
- **优化后**: 直接使用可用密钥（1次API调用）
- **性能提升**: ~75%

### 2. 状态持久化
- 避免每次启动应用时重新检测密钥状态
- 减少不必要的API调用

## 使用示例

### 1. 配置多个API密钥

#### DayDayMap
```json
{
  "api_keys": [
    "c5661493...",
    "d7772504...",
    "e8883615...",
    "f9994726..."
  ]
}
```

#### Hunter
```json
{
  "api_keys": [
    "hunter-key-1",
    "hunter-key-2",
    "hunter-key-3"
  ]
}
```

#### FOFA
```json
{
  "api_keys": ["fofa-key-1", "fofa-key-2"],
  "emails": ["email1@example.com", "email2@example.com"]
}
```

#### Quake
```json
{
  "api_keys": [
    "quake-key-1",
    "quake-key-2"
  ]
}
```

### 2. 查询示例

```rust
// 自动使用可用的API密钥
let result = hunter::search("domain:example.com", 1, 100).await?;
```

### 3. 导出示例

```rust
// 自动处理重试和部分导出
let result = hunter::export(
    "domain:example.com",
    10,  // 10页
    100, // 每页100条
    "all",
    None,
    None,
    "/path/to/export"
).await?;
```

## 日志输出示例

### 成功切换密钥
```
[hunter] 使用 Key 1 (索引 0): hunter-k...
Hunter: Key 1 配额耗尽，尝试下一个...
[hunter] 标记 Key 1 为已耗尽: hunter-k...
[hunter] 使用 Key 2 (索引 1): hunter-k...
第 1 页成功: 获取 100 条数据
```

### 部分导出
```
正在导出第 1/10 页...
第 1 页成功: 获取 100 条数据
等待 2 秒后继续...
正在导出第 2/10 页...
第 2 页成功: 获取 100 条数据
...
正在导出第 5/10 页...
Hunter: 配额耗尽，停止导出
已保存部分数据到: /path/to/export/hunter_export_20260205_103045_partial_4of10_pages.csv
```

## 测试建议

### 1. 单密钥测试
- 配置1个API密钥
- 测试正常查询和导出
- 验证配额耗尽后的错误提示

### 2. 多密钥测试
- 配置4个API密钥
- 测试密钥自动切换
- 验证状态文件正确保存

### 3. 重试测试
- 模拟网络错误
- 验证重试机制
- 验证部分导出功能

### 4. 午夜重置测试
- 修改状态文件的 `last_reset_date` 为昨天
- 重启应用
- 验证所有密钥状态被重置

## 文件修改清单

### 新增文件
- `src-tauri/src/api/key_manager.rs` - 密钥管理器（已存在，已更新）

### 修改文件
1. `src-tauri/src/config/mod.rs`
   - 新增 `get_all_hunter_api_keys()`
   - 新增 `get_all_fofa_api_keys()`
   - 新增 `get_all_quake_api_keys()`
   - 已有 `get_all_daydaymap_api_keys()`

2. `src-tauri/src/api/hunter.rs`
   - 更新 `search()` 使用 key_manager
   - 更新 `export()` 添加重试和部分导出

3. `src-tauri/src/api/fofa.rs`
   - 更新 `search()` 使用 key_manager
   - 更新 `export()` 添加重试和部分导出

4. `src-tauri/src/api/quake.rs`
   - 更新 `search()` 使用 key_manager
   - 更新 `export()` 添加重试和部分导出

5. `src-tauri/src/api/daydaymap.rs`
   - 更新 `search()` 使用新的 key_manager API
   - 已有重试和部分导出功能

6. `src-tauri/src/api/key_manager.rs`
   - 新增 `execute_with_key_rotation()` 函数
   - 简化便捷函数实现

## 总结

所有平台（Hunter、FOFA、Quake、DayDayMap）现在都支持：
- ✅ 智能API密钥轮询
- ✅ 配额耗尽自动检测
- ✅ 密钥状态持久化
- ✅ 午夜自动重置
- ✅ 重试机制（最多3次）
- ✅ 部分导出功能
- ✅ 性能优化（减少75%的API调用）

用户现在可以配置多个API密钥，系统会自动管理和轮询，大大提高了导出大量数据的成功率和效率。
