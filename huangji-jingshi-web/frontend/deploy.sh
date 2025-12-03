# 部署脚本

echo "🚀 开始部署..."

# 检查是否在正确的目录
if [ ! -f "package.json" ]; then
  echo "❌ 错误：请在 frontend 目录下运行此脚本"
  exit 1
fi

# 安装依赖
echo "📦 安装依赖..."
npm ci

# 构建项目
echo "🔨 构建项目..."
npm run build

# 检查构建结果
if [ ! -d "dist" ]; then
  echo "❌ 构建失败：dist 目录不存在"
  exit 1
fi

echo "✅ 构建完成！"
echo ""
echo "📝 下一步："
echo "   1. 如果使用 Vercel: vercel --prod"
echo "   2. 如果使用 Netlify: netlify deploy --prod"
echo "   3. 如果使用 Render: 推送到 GitHub，Render 会自动部署"
echo ""
echo "💡 确保在部署平台设置了以下环境变量："
echo "   - VITE_SUPABASE_URL"
echo "   - VITE_SUPABASE_ANON_KEY"
echo "   - VITE_API_BASE_URL (可选)"

