# 部署指南

本文档介绍如何将项目部署到 GitHub 并使用 GitHub Actions 自动构建多平台版本。

## 准备工作

### 1. 创建 GitHub 仓库

1. 登录 [GitHub](https://github.com)
2. 点击右上角 `+` → `New repository`
3. 填写仓库信息：
   - Repository name: `asset-mapping`
   - Description: `资产测绘工具 - 跨平台网络空间搜索引擎客户端`
   - Public/Private: 根据需要选择
4. 点击 `Create repository`

### 2. 本地准备

确保你的项目已经完成以下步骤：

```bash
# 进入项目目录
cd asset-mapping

# 初始化 Git（如果还没有）
git init

# 添加所有文件
git add .

# 提交
git commit -m "Initial commit: v1.0.0"
```

## 推送到 GitHub

### 方式一：HTTPS

```bash
# 添加远程仓库（替换 GUN13172）
git remote add origin https://github.com/GUN13172/asset-mapping.git

# 推送到 main 分支
git branch -M main
git push -u origin main
```

### 方式二：SSH

```bash
# 添加远程仓库（替换 GUN13172）
git remote add origin git@github.com:GUN13172/asset-mapping.git

# 推送到 main 分支
git branch -M main
git push -u origin main
```

## 配置 GitHub Actions

项目已经包含了 GitHub Actions 配置文件：

- `.github/workflows/release.yml` - 发布构建（创建 tag 时触发）
- `.github/workflows/build-test.yml` - 测试构建（push/PR 时触发）

### 权限设置

1. 进入仓库 Settings → Actions → General
2. 找到 "Workflow permissions"
3. 选择 "Read and write permissions"
4. 勾选 "Allow GitHub Actions to create and approve pull requests"
5. 点击 Save

## 发布新版本

### 方式一：创建 Release（推荐）

1. 在 GitHub 仓库页面，点击右侧 `Releases`
2. 点击 `Create a new release`
3. 填写信息：
   - Tag: `v1.0.0`（必须以 v 开头）
   - Release title: `Asset Mapping Tool v1.0.0`
   - Description: 从 `CHANGELOG.md` 复制更新内容
4. 点击 `Publish release`

GitHub Actions 会自动开始构建所有平台的安装包。

### 方式二：命令行创建 Tag

```bash
# 创建并推送 tag
git tag v1.0.0
git push origin v1.0.0
```

## 构建过程

GitHub Actions 会自动构建以下平台的安装包：

### Windows
- `asset-mapping_1.0.0_x64_en-US.msi` - MSI 安装包
- `asset-mapping_1.0.0_x64-setup.exe` - EXE 安装程序

### macOS
- `asset-mapping_1.0.0_x64.dmg` - Intel Mac 安装包
- `asset-mapping_1.0.0_aarch64.dmg` - Apple Silicon 安装包

### Linux
- `asset-mapping_1.0.0_amd64.AppImage` - AppImage 格式
- `asset-mapping_1.0.0_amd64.deb` - Debian/Ubuntu 包

构建时间约 15-30 分钟，完成后会自动上传到 Release 页面。

## 查看构建状态

### 方式一：GitHub 网页

1. 进入仓库页面
2. 点击顶部 `Actions` 标签
3. 查看最新的 workflow 运行状态

### 方式二：Badge

在 README.md 中添加构建状态徽章：

```markdown
![Build Status](https://github.com/GUN13172/asset-mapping/workflows/Release/badge.svg)
```

## 构建失败排查

### 常见问题

#### 1. 权限错误
```
Error: Resource not accessible by integration
```

**解决方案**：检查 Actions 权限设置（见上文"权限设置"）

#### 2. 依赖安装失败

**Ubuntu**:
```bash
# 检查系统依赖
sudo apt-get install -y libwebkit2gtk-4.0-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev patchelf
```

**macOS**: 通常无需额外依赖

**Windows**: 通常无需额外依赖

#### 3. Rust 编译错误

检查 `Cargo.toml` 中的依赖版本是否正确。

#### 4. 前端构建错误

```bash
# 本地测试构建
npm install
npm run build
```

### 查看详细日志

1. 进入 Actions 页面
2. 点击失败的 workflow
3. 点击具体的 job
4. 展开失败的 step 查看详细日志

## 本地测试构建

在推送到 GitHub 之前，建议先在本地测试构建：

```bash
# 进入项目目录
cd asset-mapping

# 安装依赖
npm install

# 构建
npm run tauri build
```

构建产物位于：
- Windows: `src-tauri/target/release/bundle/msi/` 和 `src-tauri/target/release/bundle/nsis/`
- macOS: `src-tauri/target/release/bundle/dmg/` 和 `src-tauri/target/release/bundle/macos/`
- Linux: `src-tauri/target/release/bundle/appimage/` 和 `src-tauri/target/release/bundle/deb/`

## 更新版本

### 1. 更新版本号

需要同时更新以下文件：

**package.json**:
```json
{
  "version": "1.1.0"
}
```

**src-tauri/Cargo.toml**:
```toml
[package]
version = "1.1.0"
```

**src-tauri/tauri.conf.json**:
```json
{
  "package": {
    "version": "1.1.0"
  }
}
```

### 2. 更新 CHANGELOG.md

在 `CHANGELOG.md` 顶部添加新版本的更新内容。

### 3. 提交并创建新 tag

```bash
# 提交更改
git add .
git commit -m "chore: bump version to 1.1.0"
git push

# 创建并推送 tag
git tag v1.1.0
git push origin v1.1.0
```

### 4. 创建 Release

在 GitHub 上创建新的 Release，GitHub Actions 会自动构建。

## 手动触发构建

如果需要手动触发构建（不创建 Release）：

1. 进入 Actions 页面
2. 选择 "Release" workflow
3. 点击右侧 "Run workflow"
4. 选择分支
5. 点击 "Run workflow"

注意：手动触发的构建不会自动创建 Release，需要手动上传构建产物。

## 自定义构建

### 修改构建目标

编辑 `.github/workflows/release.yml`：

```yaml
strategy:
  matrix:
    include:
      # 只构建 Windows
      - platform: 'windows-latest'
        args: ''
```

### 添加构建步骤

在 workflow 文件中添加自定义步骤：

```yaml
- name: Custom step
  run: |
    echo "Custom build step"
    # 你的命令
```

### 修改 Release 信息

编辑 `.github/workflows/release.yml` 中的 `tauri-action` 配置：

```yaml
- name: Build the app
  uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  with:
    tagName: ${{ github.ref_name }}
    releaseName: '自定义名称 v__VERSION__'
    releaseBody: '自定义描述'
    releaseDraft: false  # 改为 false 直接发布
    prerelease: false
```

## 安全建议

### 1. 保护敏感信息

不要在代码中硬编码：
- API 密钥
- 密码
- 私钥

使用 GitHub Secrets 存储敏感信息：
1. Settings → Secrets and variables → Actions
2. 点击 "New repository secret"
3. 在 workflow 中使用：`${{ secrets.YOUR_SECRET }}`

### 2. 代码签名

#### Windows

需要代码签名证书：
```yaml
env:
  TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
  TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
```

#### macOS

需要 Apple Developer 证书：
```yaml
env:
  APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
  APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  APPLE_ID: ${{ secrets.APPLE_ID }}
  APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
```

## 监控和通知

### 添加构建通知

可以配置 GitHub Actions 在构建完成时发送通知：

1. Slack 通知
2. Email 通知
3. Discord 通知

示例（Slack）：

```yaml
- name: Slack Notification
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Build completed!'
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
  if: always()
```

## 故障恢复

### 构建失败后重试

1. 进入 Actions 页面
2. 点击失败的 workflow
3. 点击右上角 "Re-run jobs"
4. 选择 "Re-run failed jobs" 或 "Re-run all jobs"

### 回滚版本

如果新版本有问题，可以：

1. 删除有问题的 Release 和 tag
2. 创建新的修复版本
3. 或者在 Release 页面标记为 "Pre-release"

```bash
# 删除本地 tag
git tag -d v1.0.0

# 删除远程 tag
git push origin :refs/tags/v1.0.0
```

## 参考资源

- [Tauri 官方文档](https://tauri.app/v1/guides/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri Action](https://github.com/tauri-apps/tauri-action)
- [语义化版本](https://semver.org/lang/zh-CN/)

## 技术支持

如果遇到问题：

1. 查看 [GitHub Issues](https://github.com/GUN13172/asset-mapping/issues)
2. 查看 [Tauri Discord](https://discord.com/invite/tauri)
3. 提交新的 Issue

---

最后更新：2026-02-06
