# 🚀 快速开始 - 部署到 GitHub

## 一键部署命令

```bash
# 1. 进入项目目录
cd asset-mapping

# 2. 替换用户名（将 YOUR_GITHUB_USERNAME 改为你的 GitHub 用户名）
find . -type f \( -name "*.md" -o -name "*.sh" \) -not -path "*/node_modules/*" -not -path "*/target/*" -exec sed -i '' 's/GUN13172/YOUR_GITHUB_USERNAME/g' {} \;

# 3. 初始化 Git
git init
git add .
git commit -m "Initial commit: v1.0.0"

# 4. 推送到 GitHub（替换 YOUR_GITHUB_USERNAME）
git remote add origin https://github.com/YOUR_GITHUB_USERNAME/asset-mapping.git
git branch -M main
git push -u origin main

# 5. 创建 Release（会自动触发构建）
git tag v1.0.0
git push origin v1.0.0
```

## 📋 前置步骤

### 1. 在 GitHub 创建仓库

1. 访问 https://github.com/new
2. Repository name: `asset-mapping`
3. 选择 Public 或 Private
4. **不要**勾选任何初始化选项
5. 点击 Create repository

### 2. 配置 Actions 权限

1. 进入仓库 Settings → Actions → General
2. Workflow permissions 选择 "Read and write permissions"
3. 勾选 "Allow GitHub Actions to create and approve pull requests"
4. 点击 Save

## 🎯 完成后

### 查看构建进度
https://github.com/YOUR_GITHUB_USERNAME/asset-mapping/actions

### 查看 Release
https://github.com/YOUR_GITHUB_USERNAME/asset-mapping/releases

### 构建时间
约 15-30 分钟，完成后会自动上传安装包

## 📦 构建产物

- **Windows**: `.msi` 和 `.exe`
- **macOS**: `.dmg` (Intel + Apple Silicon)
- **Linux**: `.AppImage` 和 `.deb`

## 📖 详细文档

- [完整部署指南](docs/GITHUB_SETUP.md)
- [部署文档](docs/DEPLOYMENT.md)
- [项目 README](README.md)

## 🆘 遇到问题？

1. 查看 [常见问题](docs/GITHUB_SETUP.md#常见问题)
2. 查看 [Actions 日志](https://github.com/YOUR_GITHUB_USERNAME/asset-mapping/actions)
3. 提交 [Issue](https://github.com/YOUR_GITHUB_USERNAME/asset-mapping/issues)

---

**就这么简单！** 🎉
