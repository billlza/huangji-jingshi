#!/bin/bash

# 🔧 自动修复 Render Start Command

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

clear
echo -e "${BLUE}🔧 自动修复 Render 配置${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 检查 Render CLI
if ! command -v render &> /dev/null; then
    echo -e "${RED}❌ Render CLI 未安装${NC}"
    echo "正在安装..."
    brew install render
fi

echo -e "${GREEN}✅ Render CLI 已就绪${NC}"
echo ""

# 登录检查
echo -e "${BLUE}🔑 检查登录状态...${NC}"
if ! render whoami &> /dev/null; then
    echo -e "${YELLOW}需要登录 Render CLI${NC}"
    echo -e "${BLUE}浏览器将打开，请授权登录...${NC}"
    echo ""
    render login
    echo ""
fi

echo -e "${GREEN}✅ 已登录${NC}"
echo ""

# 获取服务信息
echo -e "${BLUE}📋 获取服务列表...${NC}"
SERVICE_ID=$(render services list -o json 2>/dev/null | jq -r '.[] | select(.name == "hjjs-backend" or .name == "hjjs-backend-simple") | .id' | head -1)

if [ -z "$SERVICE_ID" ]; then
    echo -e "${RED}❌ 未找到服务 hjjs-backend${NC}"
    echo ""
    echo -e "${YELLOW}请确认：${NC}"
    echo "1. 服务已创建"
    echo "2. 服务名称是 'hjjs-backend' 或 'hjjs-backend-simple'"
    echo ""
    echo "可用的服务："
    render services list
    exit 1
fi

SERVICE_NAME=$(render services list -o json 2>/dev/null | jq -r ".[] | select(.id == \"$SERVICE_ID\") | .name")

echo -e "${GREEN}✅ 找到服务: $SERVICE_NAME${NC}"
echo -e "${BLUE}   Service ID: $SERVICE_ID${NC}"
echo ""

# 更新 Start Command
echo -e "${BLUE}🔧 更新 Start Command...${NC}"
echo ""
echo -e "${YELLOW}原配置:${NC} cd backend && ./target/release/backend"
echo -e "${GREEN}新配置:${NC} ./target/release/backend"
echo ""

# 使用 Render API 更新配置
# 注意：render CLI 可能不直接支持更新配置，需要使用 API

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}⚠️  Render CLI 不支持直接修改配置${NC}"
echo ""
echo -e "${BLUE}请手动完成以下步骤：${NC}"
echo ""
echo "1. 服务已找到：${GREEN}$SERVICE_NAME${NC}"
echo "2. 在 Render Dashboard 中："
echo "   • 进入 Settings"
echo "   • 找到 'Start Command'"
echo "   • 改为：${GREEN}./target/release/backend${NC}"
echo "   • 点击 Save Changes"
echo "   • 点击 Manual Deploy"
echo ""

# 打开服务页面
echo -e "${BLUE}正在打开服务配置页面...${NC}"
open "https://dashboard.render.com/web/$SERVICE_ID/settings" 2>/dev/null || \
xdg-open "https://dashboard.render.com/web/$SERVICE_ID/settings" 2>/dev/null || \
echo -e "${YELLOW}请手动访问: https://dashboard.render.com${NC}"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}💡 快捷指引：${NC}"
echo "1. 在打开的页面中找到 'Start Command'"
echo "2. 改为: ${GREEN}./target/release/backend${NC}"
echo "3. 点击 'Save Changes'"
echo "4. 点击右上角 'Manual Deploy'"
echo ""
echo -e "${BLUE}修改完成后，运行以下命令验证：${NC}"
echo -e "${YELLOW}curl https://hjjs-backend.onrender.com/health${NC}"
echo ""

