#!/bin/bash

echo "🧹 清理 docs/ 目录..."
echo ""

# 从 Git 缓存中删除 docs/ 目录（但保留本地文件）
echo "1️⃣ 从 Git 追踪中移除 docs/ 目录..."
git rm -r --cached docs/

echo ""
echo "2️⃣ 提交更改..."
git add .gitignore
git commit -m "chore: 从版本控制中移除 docs/ 运行日志目录"

echo ""
echo "✅ 完成！"
echo ""
echo "📋 下一步："
echo "1. 推送到 GitHub:"
echo "   git push origin main"
echo ""
echo "2. docs/ 目录仍然保留在本地，但不会再上传到 GitHub"
echo ""
echo "注意: 如果仓库已经有其他人克隆，他们需要执行："
echo "   git pull"
echo "   git rm -r docs/"
