# 资产测绘工具 (Asset Mapping Tool)

一个基于 Tauri + React + TypeScript 的跨平台资产测绘工具，支持多个网络空间搜索引擎。

## 功能特性

### 🔍 多平台支持
- **Hunter** - 奇安信 Hunter 网络空间测绘系统
- **FOFA** - 白帽汇 FOFA 网络空间搜索引擎
- **Quake** - 360 Quake 网络空间测绘系统
- **DayDayMap** - 天天地图网络空间搜索引擎

### 🎯 核心功能
- ✅ 多平台资产查询
- ✅ API 密钥管理（支持多密钥轮换）
- ✅ 查询语法转换（平台间语法互转）
- ✅ 数据导出（CSV 格式）
- ✅ 历史记录管理
- ✅ 智能查询联想
- ✅ 实时进度显示
- ✅ 主题切换（浅色/深色/跟随系统）

### 🎨 界面特性
- 现代化 UI 设计
- 响应式布局
- 实时进度弹窗
- 查询语法提示
- 多语言支持（中文/英文）

## 技术栈

### 前端
- **框架**: React 18 + TypeScript
- **UI 库**: Ant Design 5
- **构建工具**: Vite 4
- **状态管理**: React Hooks
- **样式**: CSS Variables + CSS Modules

### 后端
- **框架**: Tauri 2
- **语言**: Rust
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **数据序列化**: Serde

## 安装使用

### 下载预编译版本

从 [Releases](https://github.com/GUN13172/asset-mapping/releases) 页面下载对应平台的安装包：

- **Windows**: `.msi` 或 `.exe`
- **macOS**: `.dmg` (支持 Intel 和 Apple Silicon)
- **Linux**: `.AppImage` 或 `.deb`

### 从源码构建

#### 前置要求

1. **Node.js** >= 18
2. **Rust** >= 1.70
3. **系统依赖**:
   - **Ubuntu/Debian**:
     ```bash
     sudo apt-get update
     sudo apt-get install -y libwebkit2gtk-4.0-dev libwebkit2gtk-4.1-dev \
       libappindicator3-dev librsvg2-dev patchelf
     ```
   - **macOS**: 无需额外依赖
   - **Windows**: 无需额外依赖

#### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/GUN13172/asset-mapping.git
cd asset-mapping/asset-mapping

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

## 使用指南

### 1. 配置 API 密钥

首次使用需要配置各平台的 API 密钥：

1. 进入 **API密钥管理** 页面
2. 点击 **添加密钥**
3. 选择平台并输入密钥信息
4. 点击 **验证** 确认密钥有效

**支持的平台**:
- Hunter: 需要 API Key
- FOFA: 需要 Email + API Key
- Quake: 需要 API Key
- DayDayMap: 需要 API Key

### 2. 资产查询

1. 进入 **资产测绘** 页面
2. 选择目标平台
3. 输入查询语句（支持智能联想）
4. 设置查询参数（页数、每页数量、时间范围）
5. 点击 **开始查询**

**查询示例**:
```
Hunter:   ip="1.1.1.1"
FOFA:     ip="1.1.1.1"
Quake:    ip:"1.1.1.1"
DayDayMap: ip="1.1.1.1"
```

### 3. 语法转换

不同平台的查询语法可以互相转换：

1. 进入 **语法转换** 页面
2. 选择源平台和目标平台
3. 输入查询语句
4. 点击 **转换** 获取目标平台语法

### 4. 数据导出

支持两种导出方式：

**方式一：查询后导出**
1. 完成查询后，点击 **导出结果**
2. 选择导出目录
3. 等待导出完成

**方式二：历史记录导出**
1. 进入 **历史记录** 页面
2. 找到目标查询记录
3. 点击 **导出资产** 按钮

导出格式为 CSV，包含完整的资产信息。

## 配置说明

### 应用配置

配置文件位于：
- **开发模式**: `src-tauri/config.json`
- **生产模式**: 应用资源目录

### 数据存储

- **API 密钥**: 加密存储在本地配置文件
- **历史记录**: Json文件（应用数据目录）
- **导出文件**: 用户选择的目录

## 开发指南

### 项目结构

```
asset-mapping/
├── src/                    # 前端源码
│   ├── components/         # React 组件
│   ├── hooks/             # 自定义 Hooks
│   ├── styles/            # 样式文件
│   └── App.tsx            # 主应用
├── src-tauri/             # Tauri 后端
│   ├── src/
│   │   ├── api/          # API 调用模块
│   │   ├── config/       # 配置管理
│   │   ├── converter/    # 语法转换
│   │   ├── error/        # 错误处理
│   │   └── main.rs       # 主入口
│   ├── icons/            # 应用图标
│   └── tauri.conf.json   # Tauri 配置
├── docs/                  # 文档
└── package.json          # 项目配置
```

### 添加新平台

1. 在 `src-tauri/src/api/` 创建新平台模块
2. 实现 `search`、`export`、`validate_api_key` 函数
3. 在 `src-tauri/src/api/mod.rs` 注册新平台
4. 在前端添加对应的 UI 支持

### 调试

```bash
# 前端调试
npm run dev

# 后端调试
cd src-tauri
cargo run

# 完整应用调试
npm run tauri dev
```

## 常见问题

### Q: API 密钥验证失败？
A: 请检查：
1. 密钥是否正确
2. 网络连接是否正常
3. API 配额是否充足

### Q: 导出失败？
A: 请确保：
1. 有足够的磁盘空间
2. 导出目录有写入权限
3. 查询结果不为空

### Q: 查询超时？
A: 可能原因：
1. 网络不稳定
2. API 服务响应慢
3. 查询结果过多

## 更新日志

### v1.0.0 (2026-02-06)
- ✨ 初始版本发布
- ✅ 支持 4 个主流平台
- ✅ 完整的 API 密钥管理
- ✅ 查询语法转换
- ✅ 数据导出功能
- ✅ 历史记录管理
- ✅ 主题切换支持

## 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [React](https://react.dev/) - UI 框架
- [Ant Design](https://ant.design/) - UI 组件库
- [Rust](https://www.rust-lang.org/) - 系统编程语言

## 联系方式

- 作者: ayy
- 项目主页: [GitHub](https://github.com/GUN13172/asset-mapping)
- 问题反馈: [Issues](https://github.com/GUN13172/asset-mapping/issues)

---

⭐ 如果这个项目对你有帮助，请给个 Star！
