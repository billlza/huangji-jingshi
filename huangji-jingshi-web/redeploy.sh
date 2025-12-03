#!/bin/bash

# 重新部署脚本

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

clear
echo -e "${BLUE}🚀 重新触发 Render 部署${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${YELLOW}正在打开 Render Dashboard...${NC}"
echo ""

# 打开 Render Dashboard
open "https://dashboard.render.com" 2>/dev/null || \
xdg-open "https://dashboard.render.com" 2>/dev/null || \
echo -e "${YELLOW}请手动访问: https://dashboard.render.com${NC}"

echo ""
echo -e "${GREEN}📋 请在打开的页面中执行以下步骤：${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1️⃣  找到服务：hjjs-backend"
echo "2️⃣  点击服务名称进入详情页"
echo "3️⃣  点击右上角的 【Manual Deploy】 按钮"
echo "4️⃣  选择 【Deploy latest commit】"
echo "5️⃣  等待部署完成（约 1-2 分钟）"
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}✅ 部署成功标志：${NC}"
echo "   • 服务状态显示绿色 'Live'"
echo "   • 页面顶部显示后端 URL"
echo ""
echo -e "${BLUE}📝 部署成功后：${NC}"
echo "   1. 复制后端 URL（例如：https://hjjs-backend.onrender.com）"
echo "   2. 运行配置脚本："
echo -e "      ${YELLOW}./configure-frontend.sh <你的后端URL>${NC}"
echo ""
echo -e "${GREEN}💡 提示：如果找不到服务，可能需要先完成服务创建${NC}"
echo ""

