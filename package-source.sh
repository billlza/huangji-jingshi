#!/bin/bash

# 打包源代码脚本（排除构建产物）

set -e

echo "📦 开始打包源代码..."

# 项目根目录
PROJECT_ROOT="/Users/bill/Desktop/hjjs"
DESKTOP="/Users/bill/Desktop"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
ZIP_NAME="huangji-jingshi-source-${TIMESTAMP}.zip"
TEMP_DIR=$(mktemp -d)

echo "📁 临时目录: $TEMP_DIR"
echo "📦 输出文件: $DESKTOP/$ZIP_NAME"
echo ""

# 进入项目目录
cd "$PROJECT_ROOT"

# 创建排除列表文件
EXCLUDE_FILE="$TEMP_DIR/exclude.txt"
cat > "$EXCLUDE_FILE" << 'EOF'
# 构建产物
node_modules/
dist/
target/
build/
*.log
*.swp
*.swo
*~

# 版本控制
.git/
.gitignore

# 部署相关
.vercel/
.vercelignore
.env.local
.env*.local

# 系统文件
.DS_Store
Thumbs.db
.vscode/
.idea/
*.iml

# 临时文件
*.tmp
*.temp
.cache/
.tmp/

# 文档和说明（可选，如果需要可以注释掉）
# *.md
# !README.md
EOF

echo "📋 排除规则："
cat "$EXCLUDE_FILE" | grep -v "^#" | grep -v "^$" | sed 's/^/  - /'
echo ""

# 使用 zip 命令打包（排除指定文件）
echo "🔄 正在打包..."
cd "$PROJECT_ROOT"

# 使用 find 和 zip 打包，排除构建产物
zip -r "$DESKTOP/$ZIP_NAME" . \
  -x "*/node_modules/*" \
  -x "*/dist/*" \
  -x "*/target/*" \
  -x "*/build/*" \
  -x "*/.git/*" \
  -x "*/.vercel/*" \
  -x "*/.DS_Store" \
  -x "*/Thumbs.db" \
  -x "*/.vscode/*" \
  -x "*/.idea/*" \
  -x "*.log" \
  -x "*.swp" \
  -x "*.swo" \
  -x "*~" \
  -x "*.tmp" \
  -x "*.temp" \
  -x "*/.cache/*" \
  -x "*/package-lock.json" \
  -x "*/Cargo.lock" \
  > /dev/null 2>&1

# 检查文件大小
FILE_SIZE=$(du -h "$DESKTOP/$ZIP_NAME" | cut -f1)

echo "✅ 打包完成！"
echo ""
echo "📦 文件信息："
echo "   路径: $DESKTOP/$ZIP_NAME"
echo "   大小: $FILE_SIZE"
echo ""
echo "📝 包含内容："
echo "   ✅ 所有源代码文件"
echo "   ✅ 配置文件（package.json, Cargo.toml 等）"
echo "   ✅ 静态资源（public/ 目录）"
echo "   ✅ 文档文件"
echo ""
echo "❌ 已排除："
echo "   - node_modules/"
echo "   - dist/"
echo "   - target/"
echo "   - .git/"
echo "   - .vercel/"
echo "   - 构建产物和临时文件"
echo ""

# 清理临时文件
rm -rf "$TEMP_DIR"

echo "✨ 完成！文件已保存到桌面"







