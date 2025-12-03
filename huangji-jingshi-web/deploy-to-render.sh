#!/bin/bash

# 🚀 Render 自动化部署脚本
# 用于快速部署皇极经世后端到 Render

set -e  # 遇到错误立即退出

echo "🚀 皇极经世后端 - Render 自动化部署脚本"
echo "================================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 检查是否已登录 GitHub
echo -e "${BLUE}📋 检查 Git 状态...${NC}"
if ! git remote -v | grep -q "github.com"; then
    echo -e "${RED}❌ 未检测到 GitHub 远程仓库${NC}"
    exit 1
fi

REPO_URL=$(git remote get-url origin)
echo -e "${GREEN}✅ GitHub 仓库: $REPO_URL${NC}"
echo ""

# 检查是否安装了 Render CLI
echo -e "${BLUE}🔍 检查 Render CLI...${NC}"
if command -v render &> /dev/null; then
    echo -e "${GREEN}✅ Render CLI 已安装${NC}"
    RENDER_CLI_INSTALLED=true
else
    echo -e "${YELLOW}⚠️  Render CLI 未安装${NC}"
    RENDER_CLI_INSTALLED=false
fi
echo ""

# 如果未安装 Render CLI，提供手动部署指南
if [ "$RENDER_CLI_INSTALLED" = false ]; then
    echo -e "${YELLOW}📖 Render CLI 未安装，将打开浏览器进行手动部署${NC}"
    echo ""
    echo -e "${BLUE}请按照以下步骤操作：${NC}"
    echo ""
    echo "1️⃣  在打开的浏览器中登录 Render"
    echo "2️⃣  点击 'New +' → 'Web Service'"
    echo "3️⃣  选择你的 GitHub 仓库"
    echo "4️⃣  填写配置："
    echo ""
    echo -e "${GREEN}   Name:${NC} hjjs-backend"
    echo -e "${GREEN}   Environment:${NC} Rust"
    echo -e "${GREEN}   Root Directory:${NC} huangji-jingshi-web"
    echo -e "${GREEN}   Build Command:${NC} cd backend && cargo build --release --bin backend"
    echo -e "${GREEN}   Start Command:${NC} cd backend && ../target/release/backend"
    echo -e "${GREEN}   Health Check Path:${NC} /health"
    echo ""
    echo "5️⃣  添加环境变量："
    echo -e "${GREEN}   RUST_LOG:${NC} info"
    echo ""
    echo "6️⃣  点击 'Create Web Service'"
    echo ""
    
    # 询问是否打开浏览器
    read -p "是否现在打开 Render Dashboard？(y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}🌐 正在打开浏览器...${NC}"
        open "https://dashboard.render.com/create?type=web" 2>/dev/null || \
        xdg-open "https://dashboard.render.com/create?type=web" 2>/dev/null || \
        echo -e "${YELLOW}请手动访问: https://dashboard.render.com/create?type=web${NC}"
    fi
    
    echo ""
    echo -e "${GREEN}💡 提示：部署完成后运行以下命令配置前端：${NC}"
    echo -e "${BLUE}   ./configure-frontend.sh <你的Render后端URL>${NC}"
    echo ""
    exit 0
fi

# 使用 Render CLI 部署
echo -e "${BLUE}🔑 检查 Render 认证状态...${NC}"
if ! render whoami &> /dev/null; then
    echo -e "${YELLOW}需要登录 Render CLI${NC}"
    echo -e "${BLUE}正在打开浏览器进行认证...${NC}"
    render login
fi

echo -e "${GREEN}✅ Render 认证成功${NC}"
echo ""

# 创建或更新服务
echo -e "${BLUE}🚀 开始部署到 Render...${NC}"
echo ""

# 检查是否已存在服务
SERVICE_NAME="hjjs-backend"
echo -e "${BLUE}检查是否已存在服务: $SERVICE_NAME${NC}"

if render services list | grep -q "$SERVICE_NAME"; then
    echo -e "${YELLOW}⚠️  服务已存在，将触发重新部署${NC}"
    render deploy --service="$SERVICE_NAME"
else
    echo -e "${BLUE}📝 创建新服务...${NC}"
    
    # 使用 Blueprint 部署
    if [ -f "render-deploy.yaml" ]; then
        echo -e "${GREEN}✅ 找到 render-deploy.yaml，使用 Blueprint 部署${NC}"
        render blueprint launch
    else
        echo -e "${YELLOW}⚠️  未找到 render-deploy.yaml${NC}"
        echo -e "${BLUE}请使用浏览器手动创建服务${NC}"
        open "https://dashboard.render.com/create?type=web"
        exit 1
    fi
fi

echo ""
echo -e "${GREEN}✅ 部署命令已执行${NC}"
echo ""
echo -e "${BLUE}📊 查看部署状态：${NC}"
echo -e "   访问: https://dashboard.render.com"
echo ""
echo -e "${BLUE}⏱️  预计等待时间：${NC}"
echo -e "   首次部署: 10-15 分钟"
echo -e "   后续部署: 5-8 分钟"
echo ""
echo -e "${GREEN}🎉 部署流程已启动！${NC}"
echo ""
echo -e "${YELLOW}下一步：${NC}"
echo "1. 等待 Render 构建完成"
echo "2. 获取后端 URL（格式：https://hjjs-backend.onrender.com）"
echo "3. 运行: ./configure-frontend.sh <后端URL>"
echo ""

