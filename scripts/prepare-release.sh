#!/bin/bash

# 准备发布脚本
# 用于在发布前检查和准备项目

set -e

echo "🚀 准备发布资产测绘工具..."
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查是否在正确的目录
if [ ! -f "package.json" ]; then
    echo -e "${RED}❌ 错误: 请在 asset-mapping 目录下运行此脚本${NC}"
    exit 1
fi

echo "📋 检查项目状态..."

# 检查 Git 状态
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}⚠️  警告: 有未提交的更改${NC}"
    git status --short
    echo ""
    read -p "是否继续? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 获取当前版本
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo -e "${GREEN}✓${NC} 当前版本: ${CURRENT_VERSION}"

# 询问新版本号
echo ""
echo "请输入新版本号 (当前: ${CURRENT_VERSION}):"
read NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    echo -e "${RED}❌ 版本号不能为空${NC}"
    exit 1
fi

# 验证版本号格式
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}❌ 版本号格式错误，应为: x.y.z${NC}"
    exit 1
fi

echo ""
echo "📝 更新版本号..."

# 更新 package.json
sed -i.bak "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" package.json && rm package.json.bak
echo -e "${GREEN}✓${NC} 更新 package.json"

# 更新 Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml && rm src-tauri/Cargo.toml.bak
echo -e "${GREEN}✓${NC} 更新 Cargo.toml"

# 更新 tauri.conf.json
sed -i.bak "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json && rm src-tauri/tauri.conf.json.bak
echo -e "${GREEN}✓${NC} 更新 tauri.conf.json"

echo ""
echo "🧪 运行测试构建..."

# 安装依赖
echo "📦 安装依赖..."
npm install

# 构建前端
echo "🔨 构建前端..."
npm run build

# 检查 Rust 代码
echo "🦀 检查 Rust 代码..."
cd src-tauri
cargo check
cd ..

echo ""
echo -e "${GREEN}✓${NC} 测试构建成功"

echo ""
echo "📄 更新 CHANGELOG.md..."
echo "请手动编辑 CHANGELOG.md 添加版本 ${NEW_VERSION} 的更新内容"
echo "按 Enter 继续..."
read

# 提交更改
echo ""
echo "💾 提交更改..."
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "chore: bump version to ${NEW_VERSION}"

echo ""
echo -e "${GREEN}✓${NC} 准备完成！"
echo ""
echo "下一步操作："
echo "1. 推送到 GitHub:"
echo "   git push origin main"
echo ""
echo "2. 创建并推送 tag:"
echo "   git tag v${NEW_VERSION}"
echo "   git push origin v${NEW_VERSION}"
echo ""
echo "3. 在 GitHub 上创建 Release"
echo ""
echo "或者运行以下命令自动完成："
echo "   ./scripts/create-release.sh ${NEW_VERSION}"
