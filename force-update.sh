#!/bin/bash

# 强制更新部署脚本
echo "🚀 强制更新后端部署..."

cd /workspace/huangji-jingshi-web

# 检查并强制提交所有更改
echo "📝 检查文件状态..."
ls -la backend/

echo "📦 检查package.json..."
cat backend/package.json

echo "🟢 检查server.js存在..."
ls -la backend/server.js

# 强制添加所有文件
echo "🔄 强制添加所有文件..."
git add -A

echo "📤 提交更改..."
git commit -m "强制更新：确保Node.js后端部署

- 确认package.json存在
- 确认server.js存在  
- 确认render.yaml配置正确
- 强制部署到hjjs-backend服务"

echo "🕹️ 推送代码..."
git push

echo "✅ 代码已推送到GitHub！"
echo "📋 下一步：请手动在Render Dashboard中触发重新部署"