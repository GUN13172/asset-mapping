# 查询语句转换功能使用指南

## 功能介绍

查询语句转换功能允许您在不同的网络空间测绘平台之间转换查询语句，支持以下平台：

- **FOFA** - 白帽汇网络空间测绘系统
- **QUAKE** - 360网络空间测绘系统
- **Hunter** - 鹰图网络空间测绘系统
- **DayDayMap** - 网络空间测绘平台

## 核心特性

✅ **智能转换** - 自动识别并转换字段名和操作符  
✅ **语法验证** - 实时验证查询语句的正确性  
✅ **批量转换** - 一键转换到所有支持的平台  
✅ **示例模板** - 内置常用查询示例  
✅ **一键复制** - 快速复制转换结果

## 使用方法

### 1. 基本转换流程

1. **选择源平台** - 从下拉菜单中选择您查询语句所属的平台
2. **输入查询语句** - 在文本框中输入您的原始查询语句
3. **选择转换模式**：
   - **转换到所有平台** - 一次性转换到其他所有支持的平台
   - **转换到指定平台** - 只转换到您选择的目标平台
4. **验证语法**（可选）- 点击"验证语法"按钮检查语句是否正确
5. **开始转换** - 点击"开始转换"按钮执行转换
6. **复制结果** - 在转换结果中点击"复制"按钮获取转换后的查询语句

### 2. 字段映射表

不同平台对相同概念使用不同的字段名，系统会自动转换：

| 通用概念 | FOFA | QUAKE | Hunter | DayDayMap |
|---------|------|-------|--------|-----------|
| IP地址 | ip | ip | ip | ip |
| 端口 | port | port | ip.port | port |
| 域名 | domain | domain | domain | domain |
| 主机名 | host | host | domain | host |
| 操作系统 | os | os | ip.os | os |
| 服务器 | server | server | header.server | server |
| 网页标题 | title | title | web.title | title |
| 网页正文 | body | body | web.body | body |
| HTTP头 | header | headers | header | header |
| 国家 | country | country | country | country |
| 地区 | region | province | province | region |
| 城市 | city | city | city | city |

### 3. 操作符映射表

| 操作符类型 | FOFA | QUAKE | Hunter | DayDayMap |
|----------|------|-------|--------|-----------|
| 等于 | = | : | = | = |
| 与 | && | AND | && | && |
| 或 | \|\| | OR | \|\| | \|\| |
| 不等于 | != | NOT | != | != |

### 4. 查询示例

#### FOFA 查询示例
```
ip="8.8.8.8"
title="登录" && country="CN"
body="powered by" && port="80"
domain="example.com" && server="nginx"
```

#### QUAKE 查询示例
```
ip:"8.8.8.8"
title:"登录" AND country:"CN"
body:"powered by" AND port:"80"
domain:"example.com" AND server:"nginx"
```

#### Hunter 查询示例
```
ip="8.8.8.8"
web.title="登录" && country="CN"
web.body="powered by" && ip.port="80"
domain="example.com" && header.server="nginx"
```

### 5. 转换示例

**原始查询（FOFA）：**
```
ip="192.168.1.1" && port="80" && title="管理后台"
```

**转换到 QUAKE：**
```
ip:"192.168.1.1" AND port:"80" AND title:"管理后台"
```

**转换到 Hunter：**
```
ip="192.168.1.1" && ip.port="80" && web.title="管理后台"
```


## 注意事项

⚠️ **字段支持差异** - 不同平台支持的字段可能有所不同，部分高级字段可能无法转换

⚠️ **语法限制** - 转换结果仅供参考，建议在目标平台上测试验证

⚠️ **复杂查询** - 对于包含特殊语法或高级功能的查询，可能需要手动调整

⚠️ **引号使用** - 请确保查询值使用双引号 `"` 而非单引号 `'`

## 技术实现

本功能基于 [ConvertiX](https://github.com/HACK-THE-WORLD/ConvertiX) 项目集成，采用以下技术：

- **后端**: Rust + Tauri - 高性能查询解析和转换
- **前端**: React + TypeScript + Ant Design - 现代化用户界面
- **配置**: JSON 配置文件 - 灵活的平台字段映射

## 配置文件

转换规则存储在 `src-tauri/config.json` 文件中，您可以根据需要扩展或修改：

```json
{
  "fofa": {
    "fields": {
      "ip": "ip",
      "port": "port",
      "title": "title"
    },
    "operators": {
      "equal": "=",
      "and": "&&",
      "or": "||",
      "not_equal": "!="
    }
  }
}
```

## 常见问题

**Q: 为什么转换后的查询在目标平台无法使用？**  
A: 不同平台的字段支持可能不同，建议先在目标平台测试查询是否有效。

**Q: 如何添加新的平台支持？**  
A: 编辑 `src-tauri/config.json` 文件，添加新平台的字段和操作符映射即可。

**Q: 转换支持正则表达式吗？**  
A: 目前仅支持基本的字段和操作符转换，正则表达式等高级语法需要手动调整。

**Q: 可以批量转换多个查询语句吗？**  
A: 当前版本仅支持单个查询语句转换，批量转换功能将在后续版本中添加。

## 更新日志

### v1.0.0 (2025-10)
- ✨ 首次集成 ConvertiX 查询转换功能
- ✅ 支持 5 大测绘平台互转
- 🎨 美观的用户界面
- 📋 一键复制转换结果
- 🔍 实时语法验证

## 技术支持

如有问题或建议，请通过以下方式联系：

- GitHub Issues: [ConvertiX Issues](https://github.com/HACK-THE-WORLD/ConvertiX/issues)
- 项目文档: [README](./README.md)

---

**感谢 [ConvertiX](https://github.com/HACK-THE-WORLD/ConvertiX) 项目提供核心转换引擎！**

