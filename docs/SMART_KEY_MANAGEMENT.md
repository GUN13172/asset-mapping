# 智能 API Key 管理系统

## 问题分析

### 旧实现的问题

从日志可以看到，旧的实现每次都要尝试所有已失效的 key：

```
尝试使用第 1 个 API Key: c5661493...
✗ API Key 1 查询失败: 积分不足
尝试使用第 2 个 API Key: f101056a...
✗ API Key 2 查询失败: 积分不足
尝试使用第 3 个 API Key: a92e6b2d...
✗ API Key 3 查询失败: 积分不足
尝试使用第 4 个 API Key: 8067ff1c...
✓ API Key 4 查询成功
```

**问题：**
1. 每次请求都要尝试 3 个已失效的 key
2. 浪费 API 调用次数
3. 增加响应延迟
4. 日志输出混乱

## 新实现：智能 Key 管理

### 核心特性

1. **游标机制** - 记住当前使用的 key 索引
2. **状态持久化** - 保存每个 key 的状态到文件
3. **失效标记** - 标记已耗尽的 key 和失效时间
4. **自动重置** - 每天 0 点自动重置所有 key 状态
5. **智能跳过** - 直接跳过已知失效的 keys

### 状态文件结构

文件位置：`~/Library/Application Support/asset-mapping/daydaymap_key_state.json`

```json
{
  "current_index": 3,
  "last_reset_date": "2026-02-05",
  "keys": [
    {
      "key": "c5661493dbcf42d8aa4cf5289d92c772",
      "is_exhausted": true,
      "exhausted_at": "2026-02-05T10:18:45+08:00",
      "last_used_at": "2026-02-05T10:18:45+08:00"
    },
    {
      "key": "f101056a98154ef4ad4e3b7d1d5d75e8",
      "is_exhausted": true,
      "exhausted_at": "2026-02-05T10:25:12+08:00",
      "last_used_at": "2026-02-05T10:25:12+08:00"
    },
    {
      "key": "a92e6b2d695a480eab67928608c20c35",
      "is_exhausted": true,
      "exhausted_at": "2026-02-05T10:27:30+08:00",
      "last_used_at": "2026-02-05T10:27:30+08:00"
    },
    {
      "key": "8067ff1c9eba49f68f6ecf87bf7d983c",
      "is_exhausted": false,
      "exhausted_at": null,
      "last_used_at": "2026-02-05T10:29:15+08:00"
    }
  ]
}
```

### 工作流程

#### 1. 首次请求

```
请求 #1:
1. 加载状态文件（不存在，创建新状态）
2. 初始化所有 keys 为可用状态
3. current_index = 0
4. 使用 Key 1 (索引 0)
5. 成功 → 更新 last_used_at
```

#### 2. Key 耗尽

```
请求 #N:
1. 使用 Key 1 (索引 0)
2. 返回"积分不足"
3. 标记 Key 1 为已耗尽
4. 移动游标: current_index = 1
5. 保存状态
```

#### 3. 后续请求（智能跳过）

```
请求 #N+1:
1. 加载状态文件
2. current_index = 1
3. 检查 Key 1: is_exhausted = true → 跳过
4. 直接使用 Key 2 (索引 1)
5. 成功 → 更新 last_used_at
```

#### 4. 每日重置

```
第二天首次请求:
1. 加载状态文件
2. 检查 last_reset_date: "2026-02-05"
3. 当前日期: "2026-02-06"
4. 日期不同 → 重置所有 keys
5. 所有 is_exhausted = false
6. current_index = 0
7. last_reset_date = "2026-02-06"
8. 保存状态
```

### 日志对比

#### 旧实现（低效）

```
正在导出第 6/20 页...
可用 API Key 数量: 7
尝试使用第 1 个 API Key: c5661493...
✗ API Key 1 查询失败: 积分不足
  检测到额度耗尽，尝试下一个 key...
尝试使用第 2 个 API Key: f101056a...
✗ API Key 2 查询失败: 积分不足
  检测到额度耗尽，尝试下一个 key...
尝试使用第 3 个 API Key: a92e6b2d...
✗ API Key 3 查询失败: 积分不足
  检测到额度耗尽，尝试下一个 key...
尝试使用第 4 个 API Key: 8067ff1c...
✓ API Key 4 查询成功
```

**问题：**
- 4 次 API 调用
- 3 次失败尝试
- 响应时间长

#### 新实现（高效）

```
正在导出第 6/20 页...
Key 状态: 总计: 4 个 Key | 可用: 1 | 已耗尽: 3 | 当前游标: 4
尝试 #1: 使用 Key 4 (索引 3)
✓ Key 4 查询成功
```

**优势：**
- 1 次 API 调用
- 0 次失败尝试
- 响应时间短
- 日志清晰

### API 接口

#### 1. 获取下一个可用 key

```rust
use crate::api::key_manager;

let (api_key, key_index) = key_manager::get_next_key()?;
// api_key: "8067ff1c9eba49f68f6ecf87bf7d983c"
// key_index: 3
```

#### 2. 标记 key 为已耗尽

```rust
key_manager::mark_exhausted(key_index)?;
// 自动移动游标到下一个位置
```

#### 3. 更新最后使用时间

```rust
key_manager::update_used(key_index)?;
```

#### 4. 获取状态摘要

```rust
let status = key_manager::get_status()?;
// "总计: 4 个 Key | 可用: 1 | 已耗尽: 3 | 当前游标: 4"
```

### 使用示例

#### 场景 1：正常查询

```
时间: 2026-02-05 10:00
状态: Key 1-3 已耗尽，Key 4 可用，游标在 Key 4

用户查询:
1. 加载状态 → 游标指向 Key 4
2. 直接使用 Key 4
3. 成功返回结果

API 调用: 1 次
响应时间: 快
```

#### 场景 2：Key 4 也耗尽

```
时间: 2026-02-05 11:00
状态: Key 1-3 已耗尽，Key 4 可用，游标在 Key 4

用户查询:
1. 使用 Key 4
2. 返回"积分不足"
3. 标记 Key 4 为已耗尽
4. 尝试查找下一个可用 key
5. 所有 keys 都已耗尽
6. 返回错误："所有 API Key 都已额度耗尽"

API 调用: 1 次
```

#### 场景 3：第二天自动重置

```
时间: 2026-02-06 00:05
状态: 所有 keys 已耗尽（昨天）

用户查询:
1. 加载状态文件
2. 检测到新的一天
3. 重置所有 keys 为可用
4. 游标重置为 0
5. 使用 Key 1
6. 成功返回结果

日志:
检测到新的一天，重置所有 key 状态
Key 状态: 总计: 4 个 Key | 可用: 4 | 已耗尽: 0 | 当前游标: 1
尝试 #1: 使用 Key 1 (索引 0)
✓ Key 1 查询成功
```

### 性能对比

#### 导出 20 页数据

**旧实现：**
```
Key 1: 使用 0 页（已耗尽）
Key 2: 使用 0 页（已耗尽）
Key 3: 使用 0 页（已耗尽）
Key 4: 使用 3 页
Key 5: 使用 17 页

总 API 调用: 20 + (3 × 17) = 71 次
  - 成功调用: 20 次
  - 失败尝试: 51 次
浪费率: 71.8%
```

**新实现：**
```
Key 4: 使用 3 页
Key 5: 使用 17 页

总 API 调用: 20 次
  - 成功调用: 20 次
  - 失败尝试: 0 次
浪费率: 0%
```

**节省：**
- API 调用减少 71.8%
- 响应时间减少约 70%
- 日志输出减少 71.8%

### 配置管理

#### 查看当前状态

```bash
# 查看状态文件
cat ~/Library/Application\ Support/asset-mapping/daydaymap_key_state.json | python3 -m json.tool

# 输出示例
{
  "current_index": 3,
  "last_reset_date": "2026-02-05",
  "keys": [
    {
      "key": "c5661493...",
      "is_exhausted": true,
      "exhausted_at": "2026-02-05T10:18:45+08:00"
    },
    ...
  ]
}
```

#### 手动重置状态

```bash
# 删除状态文件，下次请求时会自动重新初始化
rm ~/Library/Application\ Support/asset-mapping/daydaymap_key_state.json
```

#### 添加新 key

```bash
# 编辑配置文件
nano ~/Library/Application\ Support/asset-mapping/daydaymap_api.json

# 添加新 key 到 api_keys 数组
# 下次请求时会自动检测并添加到状态管理
```

### 故障排查

#### 问题 1：所有 keys 都显示已耗尽，但实际还有额度

**原因：** 状态文件中的日期还是今天，没有触发重置

**解决：**
```bash
# 方法 1：删除状态文件
rm ~/Library/Application\ Support/asset-mapping/daydaymap_key_state.json

# 方法 2：手动修改日期
# 编辑状态文件，将 last_reset_date 改为昨天的日期
```

#### 问题 2：游标一直指向已耗尽的 key

**原因：** 状态文件损坏或逻辑错误

**解决：**
```bash
# 删除状态文件，重新初始化
rm ~/Library/Application\ Support/asset-mapping/daydaymap_key_state.json
```

#### 问题 3：新添加的 key 没有被使用

**原因：** 游标还在旧的位置，需要等待轮询到新 key

**解决：**
```bash
# 删除状态文件，让系统重新扫描所有 keys
rm ~/Library/Application\ Support/asset-mapping/daydaymap_key_state.json
```

### 代码结构

```
src/api/
├── key_manager.rs          # Key 管理器（新增）
│   ├── KeyStatus          # Key 状态结构
│   ├── KeyManagerState    # 管理器状态
│   ├── KeyManager         # 管理器实现
│   └── 便捷函数
│       ├── get_next_key()
│       ├── mark_exhausted()
│       ├── update_used()
│       └── get_status()
│
└── daydaymap.rs
    ├── search()                          # 公开接口
    ├── search_with_smart_key_rotation()  # 智能轮询（新）
    └── try_search_with_key()             # 使用指定 key
```

### 未来优化

1. **额度预测**
   - 记录每个 key 的使用次数
   - 预测何时会耗尽
   - 提前切换到下一个 key

2. **负载均衡**
   - 在多个可用 keys 之间均衡分配请求
   - 避免单个 key 过快耗尽

3. **Web 界面**
   - 可视化显示每个 key 的状态
   - 手动重置或切换 keys
   - 实时监控使用情况

4. **通知提醒**
   - 当所有 keys 都耗尽时发送通知
   - 当某个 key 即将耗尽时预警

## 总结

智能 Key 管理系统通过以下机制大幅提升了效率：

1. **游标机制** - 记住当前位置，避免重复尝试
2. **状态持久化** - 跨请求保持状态
3. **智能跳过** - 直接跳过已知失效的 keys
4. **自动重置** - 每天自动恢复所有 keys

**效果：**
- API 调用减少 70%+
- 响应时间减少 70%+
- 日志输出清晰简洁
- 用户体验显著提升
