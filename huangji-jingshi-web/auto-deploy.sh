#!/bin/bash

# 自动化部署脚本
# 使用方法: ./auto-deploy.sh

set -e

echo "🚀 皇极经世 - 自动化部署脚本"
echo "================================"
echo ""

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 未找到 Node.js"
    echo "   请先安装 Node.js: https://nodejs.org/"
    exit 1
fi

# 检查 npm
if ! command -v npm &> /dev/null; then
    echo "❌ 错误: 未找到 npm"
    exit 1
fi

echo "✅ Node.js 版本: $(node --version)"
echo "✅ npm 版本: $(npm --version)"
echo ""

# 进入前端目录
cd "$(dirname "$0")"
FRONTEND_DIR="$(pwd)"
echo "📁 工作目录: $FRONTEND_DIR"
echo ""

# 检查环境变量
echo "🔍 检查环境变量..."
if [ -f ".env.local" ]; then
    echo "✅ 找到 .env.local 文件"
    source .env.local
else
    echo "⚠️  未找到 .env.local 文件"
    echo "   请确保设置了以下环境变量："
    echo "   - VITE_SUPABASE_URL"
    echo "   - VITE_SUPABASE_ANON_KEY"
    echo ""
fi

# 安装依赖
echo "📦 安装依赖..."
npm ci

# 构建项目
echo "🔨 构建项目..."
npm run build

# 检查构建结果
if [ ! -d "dist" ]; then
    echo "❌ 构建失败: dist 目录不存在"
    exit 1
fi

echo "✅ 构建完成！"
echo ""

# 检查 Vercel CLI
if ! command -v vercel &> /dev/null; then
    echo "📦 安装 Vercel CLI..."
    npm install -g vercel
fi

echo "✅ Vercel CLI 已就绪"
echo ""

# 检查是否已登录
echo "🔐 检查 Vercel 登录状态..."
if vercel whoami &> /dev/null; then
    echo "✅ 已登录 Vercel"
    VERCEL_USER=$(vercel whoami)
    echo "   用户: $VERCEL_USER"
else
    echo "⚠️  未登录 Vercel"
    echo "   正在打开登录页面..."
    vercel login
fi

echo ""
echo "🚀 开始部署到 Vercel..."
echo ""

# 部署到生产环境
cd ..
vercel --prod --yes

echo ""
echo "✅ 部署完成！"
echo ""
echo "📝 下一步："
echo "   1. 在 Vercel Dashboard 中检查部署状态"
echo "   2. 确保设置了以下环境变量："
echo "      - VITE_SUPABASE_URL"
echo "      - VITE_SUPABASE_ANON_KEY"
echo "   3. 如果环境变量未设置，请在 Vercel Dashboard > Settings > Environment Variables 中添加"
echo ""

