# DayDayMap 导出功能增强 - 重试机制和部分导出

## 问题描述

在导出 DayDayMap 数据时，如果遇到以下情况，之前的实现会直接失败并丢失已导出的数据：
1. API 额度耗尽（错误码 2004："积分不足"）
2. 网络临时故障
3. API 临时限流

**示例场景：**
- 用户请求导出 10 页数据
- 前 4 页成功（400 条数据）
- 第 5 页失败：积分不足
- **旧行为**：所有数据丢失，导出失败
- **新行为**：保存前 4 页数据，提示部分成功

## 解决方案

### 1. 重试机制

对于临时性错误（网络故障、限流等），自动重试最多 3 次：

```rust
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_SECS: u64 = 5;

while retry_count < MAX_RETRIES && !page_success {
    match search(query, page, page_size).await {
        Ok(data) => {
            // 成功处理
            page_success = true;
        }
        Err(e) => {
            // 检查是否可重试
            if is_retryable_error(&e) {
                retry_count += 1;
                tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
            } else {
                break; // 不可重试错误，立即停止
            }
        }
    }
}
```

### 2. 错误分类

**不可重试错误**（立即停止并保存已有数据）：
- `积分不足` / `额度不足`
- `API密钥无效`
- `未授权`

**可重试错误**（自动重试）：
- 网络超时
- 临时服务器错误
- 限流错误

### 3. 部分导出

即使导出过程中断，也会保存已成功获取的数据：

```rust
// 生成带有状态标识的文件名
let file_name = if successful_pages < pages {
    format!("daydaymap_export_{}_partial_{}of{}_pages.csv", 
            timestamp, successful_pages, pages)
} else {
    format!("daydaymap_export_{}.csv", timestamp)
};
```

**文件命名规则：**
- 完整导出：`daydaymap_export_20260205_103045.csv`
- 部分导出：`daydaymap_export_20260205_103045_partial_4of10_pages.csv`

### 4. 详细日志

导出过程中提供详细的进度和错误信息：

```
正在导出第 1/10 页...
第 1 页成功: 获取 100 条数据
等待 2 秒后继续...

正在导出第 5/10 页...
第 5 页失败（不可重试）: API返回错误: 积分不足,请联系管理员
检测到额度耗尽或权限问题，停止导出

开始写入文件: /path/to/daydaymap_export_20260205_103045_partial_4of10_pages.csv
成功导出 4 页，共 400 条数据
文件写入完成
```

## 使用示例

### 场景 1：完整导出成功

```
用户请求：导出 10 页，每页 100 条
结果：
- 成功导出 10 页，共 1000 条数据
- 文件名：daydaymap_export_20260205_103045.csv
- 返回：成功
```

### 场景 2：部分导出（额度耗尽）

```
用户请求：导出 10 页，每页 100 条
结果：
- 成功导出 4 页，共 400 条数据
- 第 5 页失败：积分不足
- 文件名：daydaymap_export_20260205_103045_partial_4of10_pages.csv
- 返回：错误信息包含文件路径和已保存数据量
```

### 场景 3：临时错误自动重试

```
用户请求：导出 10 页，每页 100 条
过程：
- 第 3 页首次失败：网络超时
- 等待 5 秒后重试
- 第 3 页重试成功
结果：
- 成功导出 10 页，共 1000 条数据
- 文件名：daydaymap_export_20260205_103045.csv
- 返回：成功
```

## 代码变更

### 主要修改

1. **添加重试逻辑**
   ```rust
   let mut retry_count = 0;
   while retry_count < MAX_RETRIES && !page_success {
       // 尝试获取数据
       // 失败时检查是否可重试
   }
   ```

2. **错误分类处理**
   ```rust
   if e.contains("积分不足") || e.contains("额度不足") || 
      e.contains("API密钥无效") || e.contains("未授权") {
       // 不可重试，停止导出
       break;
   }
   ```

3. **保存部分数据**
   ```rust
   if all_results.is_empty() {
       return Err("没有成功获取任何数据".to_string());
   }
   // 即使部分失败，也写入已获取的数据
   ```

4. **返回详细信息**
   ```rust
   if successful_pages < pages {
       return Err(format!(
           "部分导出成功: 已保存 {} 页（共 {} 条数据）到文件 {}。\n最后错误: {}", 
           successful_pages, total_results, file_path, error
       ));
   }
   ```

## 配置参数

可以通过修改常量来调整重试行为：

```rust
const MAX_RETRIES: u32 = 3;          // 最大重试次数
const RETRY_DELAY_SECS: u64 = 5;     // 重试间隔（秒）
```

## 测试验证

### 测试用例 1：正常导出

```bash
# 导出 2 页数据
cargo run --example test_export_normal

# 预期结果：
# - 成功导出 2 页
# - 文件名不包含 "partial"
```

### 测试用例 2：模拟额度耗尽

```bash
# 导出 10 页数据（假设第 5 页额度耗尽）
cargo run --example test_export_quota_exceeded

# 预期结果：
# - 成功导出 4 页
# - 文件名包含 "partial_4of10_pages"
# - 返回错误信息包含文件路径
```

### 测试用例 3：模拟网络故障

```bash
# 导出数据，模拟临时网络故障
cargo run --example test_export_network_retry

# 预期结果：
# - 自动重试失败的请求
# - 最终成功导出所有数据
```

## 前端集成

前端需要处理部分成功的情况：

```typescript
try {
  await invoke('export_results', {
    platform: 'daydaymap',
    query: 'ip:"183.201.199.0/24"',
    pages: 10,
    pageSize: 100,
    timeRange: 'all'
  });
  
  // 完全成功
  message.success('导出成功！');
  
} catch (error) {
  // 检查是否是部分成功
  if (error.includes('部分导出成功')) {
    // 提取文件路径和数据量
    message.warning(error);
  } else {
    // 完全失败
    message.error('导出失败: ' + error);
  }
}
```

## 优势

1. **数据不丢失**：即使导出中断，已获取的数据也会被保存
2. **自动恢复**：临时错误自动重试，提高成功率
3. **清晰反馈**：详细的日志和错误信息，便于问题排查
4. **文件标识**：文件名清楚标识是完整导出还是部分导出
5. **用户友好**：用户可以根据部分导出的数据决定是否继续

## 后续优化建议

1. **断点续传**：记录导出进度，支持从中断处继续
2. **并发导出**：支持多线程并发获取数据（需注意 API 限流）
3. **智能重试**：根据错误类型动态调整重试策略
4. **导出队列**：支持后台导出，不阻塞用户操作

## 总结

通过添加重试机制和部分导出功能，DayDayMap 导出功能现在更加健壮和用户友好。即使遇到 API 额度耗尽或临时故障，用户也不会丢失已经获取的数据。
