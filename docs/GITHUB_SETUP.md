# GitHub 部署完整指南

## 📋 部署清单

### 第一步：准备项目

- [x] 项目代码已完成
- [x] 已创建 `.gitignore` 文件
- [x] 已创建 `README.md`
- [x] 已创建 `LICENSE`
- [x] 已创建 `CHANGELOG.md`
- [x] 已配置 GitHub Actions workflows
- [ ] 已更新 README 中的用户名占位符

### 第二步：创建 GitHub 仓库

1. **登录 GitHub**
   - 访问 https://github.com
   - 登录你的账号

2. **创建新仓库**
   - 点击右上角 `+` → `New repository`
   - Repository name: `asset-mapping`
   - Description: `资产测绘工具 - 跨平台网络空间搜索引擎客户端`
   - 选择 Public 或 Private
   - **不要**勾选 "Initialize this repository with a README"
   - 点击 `Create repository`

### 第三步：更新项目配置

在推送代码前，需要替换以下文件中的 `GUN13172` 为你的 GitHub 用户名：

**需要修改的文件：**
1. `README.md` - 多处链接
2. `CHANGELOG.md` - Release 链接
3. `docs/DEPLOYMENT.md` - 示例命令
4. `scripts/create-release.sh` - GitHub 链接

**快速替换命令：**
```bash
# 在 asset-mapping 目录下执行
# 将 YOUR_GITHUB_USERNAME 替换为你的实际用户名

find . -type f \( -name "*.md" -o -name "*.sh" \) -not -path "*/node_modules/*" -not -path "*/target/*" -exec sed -i.bak 's/GUN13172/YOUR_GITHUB_USERNAME/g' {} \;

# 清理备份文件
find . -name "*.bak" -delete
```

### 第四步：初始化 Git 并推送

```bash
# 1. 进入项目目录
cd asset-mapping

# 2. 初始化 Git（如果还没有）
git init

# 3. 添加所有文件
git add .

# 4. 提交
git commit -m "Initial commit: v1.0.0"

# 5. 添加远程仓库（替换 GUN13172）
git remote add origin https://github.com/GUN13172/asset-mapping.git

# 6. 推送到 main 分支
git branch -M main
git push -u origin main
```

### 第五步：配置 GitHub Actions 权限

1. 进入仓库页面
2. 点击 `Settings` 标签
3. 左侧菜单选择 `Actions` → `General`
4. 找到 "Workflow permissions" 部分
5. 选择 **"Read and write permissions"**
6. 勾选 **"Allow GitHub Actions to create and approve pull requests"**
7. 点击 `Save`

### 第六步：创建第一个 Release

#### 方式一：通过 GitHub 网页（推荐）

1. 在仓库页面，点击右侧 `Releases`
2. 点击 `Create a new release`
3. 填写信息：
   - **Choose a tag**: 输入 `v1.0.0`，点击 "Create new tag: v1.0.0 on publish"
   - **Release title**: `Asset Mapping Tool v1.0.0`
   - **Description**: 复制以下内容

```markdown
## 🎉 首次发布

资产测绘工具 v1.0.0 正式发布！

### ✨ 主要功能

- 支持 Hunter、FOFA、Quake、DayDayMap 四大平台
- API 密钥管理（多密钥支持、自动轮换）
- 查询语法转换（平台间互转）
- 数据导出（CSV 格式）
- 历史记录管理
- 智能查询联想
- 主题切换（浅色/深色/跟随系统）

### 📦 安装说明

#### Windows
下载 `.msi` 或 `.exe` 文件，双击安装

#### macOS
- Intel Mac: 下载 `x64.dmg`
- Apple Silicon: 下载 `aarch64.dmg`

双击 DMG 文件，拖动到 Applications 文件夹

#### Linux
- AppImage: 下载后添加执行权限 `chmod +x *.AppImage`
- Debian/Ubuntu: 下载 `.deb` 文件，运行 `sudo dpkg -i *.deb`

### 📖 文档

- [使用指南](https://github.com/GUN13172/asset-mapping#使用指南)
- [开发文档](https://github.com/GUN13172/asset-mapping/blob/main/docs/DEPLOYMENT.md)

### 🐛 问题反馈

如有问题，请提交 [Issue](https://github.com/GUN13172/asset-mapping/issues)
```

4. 点击 `Publish release`

#### 方式二：使用脚本

```bash
# 在 asset-mapping 目录下执行
./scripts/create-release.sh 1.0.0
```

### 第七步：等待构建完成

1. 点击仓库顶部的 `Actions` 标签
2. 查看 "Release" workflow 的运行状态
3. 构建时间约 15-30 分钟
4. 构建完成后，安装包会自动上传到 Release 页面

**构建的文件：**
- Windows: `.msi` 和 `.exe`
- macOS: `.dmg` (x64 和 aarch64)
- Linux: `.AppImage` 和 `.deb`

## 🔍 验证部署

### 检查清单

- [ ] 代码已成功推送到 GitHub
- [ ] GitHub Actions workflow 已触发
- [ ] 所有平台构建成功（无红色 ❌）
- [ ] Release 页面显示所有安装包
- [ ] README 中的链接正确
- [ ] 下载并测试至少一个平台的安装包

### 测试安装包

1. 从 Release 页面下载对应平台的安装包
2. 安装并运行应用
3. 测试基本功能：
   - 添加 API 密钥
   - 执行查询
   - 导出数据
   - 切换主题

## 🚨 常见问题

### 问题 1: Actions 权限错误

**错误信息：**
```
Error: Resource not accessible by integration
```

**解决方案：**
检查 Actions 权限设置（见第五步）

### 问题 2: 构建失败

**检查步骤：**
1. 点击失败的 workflow
2. 查看详细日志
3. 根据错误信息修复代码
4. 重新推送或重新运行 workflow

### 问题 3: 安装包未上传

**可能原因：**
- Release 设置为 Draft（草稿）
- 构建失败
- 权限不足

**解决方案：**
1. 检查 Release 是否为 Draft 状态
2. 查看 Actions 日志
3. 确认权限设置正确

### 问题 4: macOS 安装包无法打开

**错误信息：**
```
"asset-mapping" is damaged and can't be opened
```

**解决方案：**
```bash
# 移除隔离属性
xattr -cr /Applications/asset-mapping.app
```

或者在系统设置中允许运行未签名的应用。

## 📚 后续操作

### 更新版本

1. 修改版本号（3个文件）：
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`

2. 更新 `CHANGELOG.md`

3. 提交并推送：
```bash
git add .
git commit -m "chore: bump version to 1.1.0"
git push
```

4. 创建新 tag：
```bash
git tag v1.1.0
git push origin v1.1.0
```

5. 在 GitHub 创建新 Release

### 使用自动化脚本

```bash
# 准备发布（更新版本号、测试构建）
./scripts/prepare-release.sh

# 创建 Release
./scripts/create-release.sh 1.1.0
```

### 添加 Badge

在 `README.md` 顶部添加状态徽章：

```markdown
![Release](https://img.shields.io/github/v/release/GUN13172/asset-mapping)
![Build](https://github.com/GUN13172/asset-mapping/workflows/Release/badge.svg)
![License](https://img.shields.io/github/license/GUN13172/asset-mapping)
![Downloads](https://img.shields.io/github/downloads/GUN13172/asset-mapping/total)
```

## 🎯 最佳实践

### 版本管理

遵循[语义化版本](https://semver.org/lang/zh-CN/)：
- **主版本号**：不兼容的 API 修改
- **次版本号**：向下兼容的功能性新增
- **修订号**：向下兼容的问题修正

### 分支策略

- `main` - 稳定版本
- `develop` - 开发版本
- `feature/*` - 功能分支
- `hotfix/*` - 紧急修复

### Release 策略

- 使用 Draft Release 进行预发布测试
- 重大更新使用 Pre-release 标记
- 每个 Release 包含详细的更新日志

### 安全建议

1. 不要在代码中硬编码敏感信息
2. 使用 GitHub Secrets 存储密钥
3. 定期更新依赖包
4. 启用 Dependabot 自动更新

## 📞 获取帮助

- [Tauri 文档](https://tauri.app/v1/guides/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [项目 Issues](https://github.com/GUN13172/asset-mapping/issues)

---

**祝你部署顺利！** 🎉

如有问题，欢迎提交 Issue 或 Pull Request。
