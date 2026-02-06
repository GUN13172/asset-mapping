# 🚀 立即部署 - 三步完成

## 第一步：替换用户名 (1分钟)

```bash
# 在 asset-mapping 目录下执行
# 将 YOUR_GITHUB_USERNAME 替换为你的 GitHub 用户名

# macOS 用户：
find . -type f \( -name "*.md" -o -name "*.sh" \) \
  -not -path "*/node_modules/*" \
  -not -path "*/target/*" \
  -not -path "*/dist/*" \
  -exec sed -i '' 's/GUN13172/YOUR_GITHUB_USERNAME/g' {} \;

# Linux 用户：
find . -type f \( -name "*.md" -o -name "*.sh" \) \
  -not -path "*/node_modules/*" \
  -not -path "*/target/*" \
  -not -path "*/dist/*" \
  -exec sed -i 's/GUN13172/YOUR_GITHUB_USERNAME/g' {} \;
```

## 第二步：创建 GitHub 仓库 (2分钟)

1. 访问 https://github.com/new
2. Repository name: `asset-mapping`
3. 选择 Public
4. **不要**勾选任何初始化选项
5. 点击 Create repository
6. 进入 Settings → Actions → General
7. Workflow permissions 选择 "Read and write permissions"
8. 勾选 "Allow GitHub Actions to create and approve pull requests"
9. 点击 Save

## 第三步：推送并发布 (2分钟)

```bash
# 在 asset-mapping 目录下执行
# 替换 YOUR_GITHUB_USERNAME 为你的用户名

git init
git add .
git commit -m "Initial commit: v1.0.0"
git remote add origin https://github.com/YOUR_GITHUB_USERNAME/asset-mapping.git
git branch -M main
git push -u origin main
git tag v1.0.0
git push origin v1.0.0
```

## ✅ 完成！

现在访问：
- **Actions**: https://github.com/YOUR_GITHUB_USERNAME/asset-mapping/actions
- **Releases**: https://github.com/YOUR_GITHUB_USERNAME/asset-mapping/releases

等待 15-30 分钟，构建完成后会自动上传安装包。

## 📚 详细文档

- [快速开始](QUICK_START.md)
- [完整指南](docs/GITHUB_SETUP.md)
- [部署总结](docs/打包部署总结.md)
