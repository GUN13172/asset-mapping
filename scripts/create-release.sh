#!/bin/bash

# 创建 GitHub Release 脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 检查参数
if [ -z "$1" ]; then
    echo -e "${RED}❌ 错误: 请提供版本号${NC}"
    echo "用法: ./create-release.sh <version>"
    echo "示例: ./create-release.sh 1.0.0"
    exit 1
fi

VERSION=$1
TAG="v${VERSION}"

echo "🚀 创建 Release: ${TAG}"
echo ""

# 检查是否在正确的目录
if [ ! -f "package.json" ]; then
    echo -e "${RED}❌ 错误: 请在 asset-mapping 目录下运行此脚本${NC}"
    exit 1
fi

# 检查 Git 状态
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}❌ 错误: 有未提交的更改，请先提交${NC}"
    exit 1
fi

# 检查是否在 main 分支
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${YELLOW}⚠️  警告: 当前不在 main 分支 (当前: ${CURRENT_BRANCH})${NC}"
    read -p "是否继续? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 检查 tag 是否已存在
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}❌ 错误: Tag ${TAG} 已存在${NC}"
    exit 1
fi

echo "📤 推送代码到 GitHub..."
git push origin $CURRENT_BRANCH

echo ""
echo "🏷️  创建并推送 tag..."
git tag -a "$TAG" -m "Release ${TAG}"
git push origin "$TAG"

echo ""
echo -e "${GREEN}✓${NC} Release 创建成功！"
echo ""
echo "GitHub Actions 正在构建多平台版本..."
echo "查看构建进度: https://github.com/GUN13172/asset-mapping/actions"
echo ""
echo "构建完成后，访问以下链接查看 Release:"
echo "https://github.com/GUN13172/asset-mapping/releases/tag/${TAG}"
