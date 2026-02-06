# 🚀 快速启动指南

## 立即开始

### 1️⃣ 启动开发模式

```bash
cd asset-mapping
npm run tauri dev
```

### 2️⃣ 使用查询转换功能

1. 打开应用后，点击左侧菜单的 **"语句转换"** 📝
2. 选择源平台（如 FOFA）
3. 输入查询语句，例如：`ip="8.8.8.8" && port="80"`
4. 点击 **"开始转换"** 按钮
5. 查看转换结果，点击 **"复制"** 按钮使用

### 3️⃣ 转换示例

**FOFA 查询：**
```
title="登录" && country="CN"
```

**自动转换到 QUAKE：**
```
title:"登录" AND country:"CN"
```

**自动转换到 Hunter：**
```
web.title="登录" && country="CN"
```

## 功能特性

✨ **5大平台互转**
- FOFA ⇄ QUAKE ⇄ Hunter ⇄ DayDayMap

🔍 **智能验证**
- 实时语法检查
- 友好错误提示

⚡ **快速操作**
- 一键转换所有平台
- 示例查询快速加载
- 结果一键复制

## 项目结构

```
asset-mapping/
├── src/                          # 前端代码
│   ├── App.tsx                   # 主应用（已更新）
│   └── components/
│       └── QueryConverter.tsx    # 查询转换组件（新增）
├── src-tauri/                    # 后端代码
│   ├── src/
│   │   ├── main.rs              # 主程序（已更新）
│   │   ├── converter/           # 转换引擎（新增）
│   │   ├── error/               # 错误处理（新增）
│   │   └── config/
│   │       └── platform.rs      # 平台配置（新增）
│   └── config.json              # 转换配置（新增）
├── CONVERTER_GUIDE.md           # 详细使用指南
├── INTEGRATION_SUMMARY.md       # 集成技术总结
└── QUICKSTART.md               # 本文件
```

## 构建生产版本

```bash
npm run tauri build
```

构建产物位于：`src-tauri/target/release/bundle/`

## 故障排查

### 问题：转换失败
**解决方案：** 检查 `src-tauri/config.json` 文件是否存在

### 问题：语法验证失败
**解决方案：** 确保查询语句使用正确的平台语法和操作符

### 问题：复制功能不工作
**解决方案：** 确保浏览器/系统允许剪贴板访问

## 更多信息

- 📖 [详细使用指南](CONVERTER_GUIDE.md)
- 🔧 [技术集成总结](INTEGRATION_SUMMARY.md)
- 🌟 [ConvertiX 原项目](https://github.com/HACK-THE-WORLD/ConvertiX)

---

**享受查询转换的便利！** 🎉

