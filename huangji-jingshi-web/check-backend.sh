#!/bin/bash

# 检查后端服务状态

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔍 检查 Render 后端服务状态${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 可能的 URL
URLS=(
    "https://hjjs-backend.onrender.com"
    "https://hjjs-backend-simple.onrender.com"
)

echo -e "${BLUE}测试可能的后端 URL...${NC}"
echo ""

for URL in "${URLS[@]}"; do
    echo -e "${YELLOW}测试: $URL/health${NC}"
    
    # 测试健康检查
    response=$(curl -s -m 10 -w "\n%{http_code}" "$URL/health" 2>/dev/null)
    status_code=$(echo "$response" | tail -n 1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$status_code" = "200" ]; then
        echo -e "${GREEN}✅ 服务在线！${NC}"
        echo -e "${GREEN}URL: $URL${NC}"
        echo ""
        echo -e "${BLUE}响应内容:${NC}"
        echo "$body" | jq '.' 2>/dev/null || echo "$body"
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}🎉 后端已准备就绪！${NC}"
        echo ""
        echo -e "${BLUE}下一步：配置前端${NC}"
        echo -e "运行: ${YELLOW}./configure-frontend.sh $URL${NC}"
        echo ""
        exit 0
    elif [ "$status_code" = "000" ]; then
        echo -e "${RED}❌ 连接失败（服务可能还未启动或 URL 不正确）${NC}"
    else
        echo -e "${RED}❌ 返回状态码: $status_code${NC}"
    fi
    echo ""
done

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}⚠️  未能自动检测到后端服务${NC}"
echo ""
echo -e "${BLUE}可能的原因：${NC}"
echo "1. 服务还在部署中（首次部署需要 10-15 分钟）"
echo "2. 服务 URL 与预期不同"
echo "3. 服务配置有问题"
echo ""
echo -e "${BLUE}请手动检查：${NC}"
echo "1. 打开 Render Dashboard"
echo "2. 找到 hjjs-backend 服务"
echo "3. 检查服务状态（应该是绿色的 'Live'）"
echo "4. 复制页面顶部显示的 URL"
echo "5. 在浏览器中访问: <URL>/health"
echo ""

# 打开 Render Dashboard
read -p "是否打开 Render Dashboard 检查？(y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}正在打开 Render Dashboard...${NC}"
    open "https://dashboard.render.com" 2>/dev/null || \
    xdg-open "https://dashboard.render.com" 2>/dev/null || \
    echo -e "${YELLOW}请手动访问: https://dashboard.render.com${NC}"
fi

echo ""
echo -e "${BLUE}💡 提示：${NC}"
echo "如果服务状态显示 'Live'，但这里连接失败，"
echo "请确认实际的后端 URL 并手动测试："
echo ""
echo -e "${YELLOW}curl https://<你的实际URL>/health${NC}"
echo ""

