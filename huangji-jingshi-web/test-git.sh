#!/bin/bash
echo "=== Git 状态检查 ==="
cd /workspace/huangji-jingshi-web
git status

echo "=== 添加文件到暂存区 ==="
git add .

echo "=== 提交修改 ==="
git commit -m "修复API端点路径"

echo "=== 推送到GitHub ==="
git push

echo "=== 检查远程状态 ==="
git status