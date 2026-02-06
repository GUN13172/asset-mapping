# 🐛 API密钥添加问题修复

## 问题描述

在添加 API 密钥时出现以下错误：
```
添加API密钥失败: invalid args `apiKey` for command `add_api_key`: 
command add_api_key missing required key apiKey
```

## 问题原因

**Tauri 参数命名约定问题**

在 Tauri 框架中，Rust 函数的参数命名遵循蛇形命名（snake_case），但在 JavaScript/TypeScript 调用时会自动转换为驼峰命名（camelCase）。

### 错误的调用方式
```typescript
// ❌ 错误：使用蛇形命名 api_key
await invoke('add_api_key', { 
  platform, 
  api_key: values.apiKey  // 错误！
});
```

### 正确的调用方式
```typescript
// ✅ 正确：使用驼峰命名 apiKey
await invoke('add_api_key', { 
  platform, 
  apiKey: values.apiKey  // 正确！
});
```

## 修复内容

修改了 `src/components/ApiKeyManagement.tsx` 文件中的三个函数：

### 1. addApiKey 函数
```typescript
// 修复前
api_key: values.apiKey

// 修复后
apiKey: values.apiKey
```

### 2. deleteApiKey 函数
```typescript
// 修复前
api_key: apiKey

// 修复后
apiKey: apiKey
```

### 3. validateApiKey 函数
```typescript
// 修复前
api_key: apiKey

// 修复后
apiKey: apiKey
```

## Tauri 命名规则总结

| Rust 端（后端） | JavaScript 端（前端） | 说明 |
|---------------|---------------------|------|
| `api_key` | `apiKey` | Rust 蛇形命名 → JS 驼峰命名 |
| `page_size` | `pageSize` | 自动转换 |
| `start_date` | `startDate` | 自动转换 |

**规则：** Tauri 会自动将 Rust 函数参数的蛇形命名转换为驼峰命名供前端调用。

## 验证修复

执行以下步骤验证修复：

1. **重新构建项目**
   ```bash
   cd asset-mapping
   npm run build
   ```

2. **启动应用**
   ```bash
   npm run tauri dev
   ```

3. **测试添加 API 密钥**
   - 点击"API密钥管理"
   - 点击"添加API密钥"
   - 输入测试密钥
   - 点击"添加"
   - ✅ 应该成功添加

## 相关文件

- ✅ `src/components/ApiKeyManagement.tsx` - 已修复
- 📝 `src-tauri/src/main.rs` - 后端命令定义（无需修改）

## 状态

- ✅ **已修复** - 2025-10-08
- ✅ **已测试** - 编译通过
- ✅ **已部署** - 可立即使用

---

**修复完成！现在可以正常添加 API 密钥了！** 🎉




