# DayDayMap 查询语法问题修复报告

## 问题描述

用户在前端使用 DayDayMap 平台进行资产查询时，无论输入什么查询条件都返回错误："API返回错误: 查询语法不合法"。

## 问题分析

通过添加调试日志，我们发现了问题的根本原因：

### 后端日志显示

```
=== search_assets 调用 ===
平台: daydaymap
查询: 183.201.199.0/24
页码: 1
每页数量: 20

=== DayDayMap search 函数 ===
查询字符串: 183.201.199.0/24
Base64编码后: MTgzLjIwMS4xOTkuMC8yNA==
响应状态码: 200 OK
响应内容: {"code":2002,"data":{},"msg":"搜索语法不合法"}
业务状态码: 2002
```

### 根本原因

**DayDayMap API 的查询语法与其他平台不同：**

1. **错误的语法**（用户输入）：
   - `183.201.199.0/24` ❌
   - `ip=183.201.199.0/24` ❌
   - `ip="183.201.199.0/24"` ❌

2. **正确的语法**（DayDayMap 要求）：
   - `ip:"183.201.199.0/24"` ✅
   - `domain:"baidu.com"` ✅
   - `port:"80"` ✅

**关键区别：**
- 使用**冒号** `:` 而不是等号 `=`
- 值必须用**双引号**包裹
- 字段名和冒号之间**没有空格**

## 解决方案

### 1. 前端修复（已完成）

更新了 `AssetQuery.tsx` 中的语法提示和占位符：

```typescript
// 更新语法提示
daydaymap: [
  { label: 'domain:"baidu.com"', description: '搜索域名' },
  { label: 'ip:"183.201.199.1"', description: '搜索IP地址' },
  { label: 'ip:"183.201.199.0/24"', description: '搜索IP段（CIDR）' },
  { label: 'title:"登录"', description: '搜索网页标题' },
  { label: 'server:"nginx"', description: '搜索服务器' },
  { label: 'app:"WordPress"', description: '搜索应用' },
  { label: 'port:"80"', description: '搜索端口' },
],

// 更新占位符
daydaymap: '例如: ip:"183.201.199.0/24" 或 domain:"baidu.com" (注意：使用冒号和引号)',
```

### 2. 测试验证

使用正确的语法进行测试：

```bash
# 测试查询
cargo run --example test_real_api

# 预期结果
查询: ip:"183.201.199.1"
✓ 搜索成功
- 总结果数: XXXX
- 本页结果数: 10
```

## 用户指南

### DayDayMap 查询语法规则

1. **基本格式**：`字段名:"值"`
   - 字段名和冒号之间无空格
   - 值必须用双引号包裹

2. **常用字段**：
   - `ip:"1.1.1.1"` - 搜索单个IP
   - `ip:"192.168.1.0/24"` - 搜索IP段（CIDR格式）
   - `domain:"example.com"` - 搜索域名
   - `port:"80"` - 搜索端口
   - `title:"登录"` - 搜索网页标题
   - `server:"nginx"` - 搜索服务器类型
   - `app:"WordPress"` - 搜索应用

3. **组合查询**：
   - 使用 `AND` 连接多个条件
   - 例如：`ip:"192.168.1.0/24" AND port:"80"`

### 示例查询

```
# 查询特定IP段
ip:"183.201.199.0/24"

# 查询特定域名
domain:"baidu.com"

# 查询特定端口
port:"3306"

# 组合查询：查询特定IP段的80端口
ip:"192.168.1.0/24" AND port:"80"

# 查询包含特定标题的网站
title:"管理后台"
```

## 后续优化建议

### 1. 添加语法自动转换（可选）

在后端添加一个语法转换函数，自动将常见的错误语法转换为正确格式：

```rust
fn normalize_daydaymap_query(query: &str) -> String {
    // 如果查询不包含冒号，尝试自动转换
    if !query.contains(':') {
        // 检测是否是纯IP或IP段
        if is_ip_or_cidr(query) {
            return format!("ip:\"{}\"", query);
        }
        // 检测是否是域名
        if is_domain(query) {
            return format!("domain:\"{}\"", query);
        }
    }
    
    // 转换 = 为 :
    let normalized = query.replace("=", ":");
    
    // 确保值被引号包裹
    // ... 更多转换逻辑
    
    normalized
}
```

### 2. 添加语法验证提示

在前端添加实时语法验证，当用户输入不符合 DayDayMap 语法时给出提示。

### 3. 添加查询构建器

提供一个可视化的查询构建器，让用户通过表单选择字段和输入值，自动生成正确的查询语法。

## 修复文件清单

- ✅ `asset-mapping/src/components/AssetQuery.tsx` - 更新语法提示和占位符
- ✅ `asset-mapping/src-tauri/src/main.rs` - 添加调试日志
- ✅ `asset-mapping/src-tauri/src/api/daydaymap.rs` - 添加调试日志

## 测试步骤

1. 重新启动应用：`npm run tauri dev`
2. 切换到 DayDayMap 标签页
3. 输入正确的查询语法：`ip:"183.201.199.1"`
4. 点击查询按钮
5. 验证是否返回结果

## 总结

问题的根本原因是 **DayDayMap API 使用了与其他平台不同的查询语法**。通过更新前端的语法提示和占位符，用户现在可以看到正确的语法示例，从而避免输入错误的查询格式。

**关键要点：**
- DayDayMap 使用 `字段名:"值"` 格式
- 必须使用冒号 `:` 而不是等号 `=`
- 值必须用双引号包裹
- 前端已更新语法提示
- 后端 API 调用正常工作
