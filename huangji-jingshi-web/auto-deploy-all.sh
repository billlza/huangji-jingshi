#!/bin/bash

# 🚀 一键自动化部署脚本
# 完整部署后端和前端

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

clear
echo -e "${BLUE}"
echo "╔════════════════════════════════════════════════════╗"
echo "║      🌟 皇极经世 - 一键自动化部署脚本 🌟        ║"
echo "╚════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# 步骤 1: 检查 Git 状态
echo -e "${BLUE}📋 步骤 1/4: 检查 Git 状态${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if git status --porcelain | grep -q .; then
    echo -e "${YELLOW}⚠️  检测到未提交的更改${NC}"
    git status --short
    echo ""
    read -p "是否提交并推送这些更改？(y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git add .
        read -p "请输入提交信息（直接回车使用默认信息）: " commit_msg
        if [ -z "$commit_msg" ]; then
            commit_msg="更新部署配置"
        fi
        git commit -m "$commit_msg"
        git push origin main
        echo -e "${GREEN}✅ 代码已推送到 GitHub${NC}"
    else
        echo -e "${YELLOW}跳过提交，使用现有代码${NC}"
    fi
else
    echo -e "${GREEN}✅ 工作区干净，代码已是最新${NC}"
fi
echo ""

# 步骤 2: 部署后端到 Render
echo -e "${BLUE}📋 步骤 2/4: 部署后端到 Render${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${YELLOW}请选择部署方式：${NC}"
echo "1) 使用浏览器手动部署（推荐，更稳定）"
echo "2) 使用 Render CLI 自动部署"
echo "3) 跳过（后端已部署）"
echo ""
read -p "请选择 [1-3]: " deploy_choice

case $deploy_choice in
    1)
        echo ""
        echo -e "${BLUE}🌐 打开 Render Dashboard...${NC}"
        echo ""
        echo -e "${GREEN}请在浏览器中完成以下步骤：${NC}"
        echo ""
        echo "1. 登录 Render"
        echo "2. 点击 'New +' → 'Web Service'"
        echo "3. 选择 GitHub 仓库"
        echo "4. 配置服务（详见下方）："
        echo ""
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}Name:${NC} hjjs-backend"
        echo -e "${GREEN}Environment:${NC} Rust"
        echo -e "${GREEN}Root Directory:${NC} huangji-jingshi-web"
        echo -e "${GREEN}Build Command:${NC} cd backend && cargo build --release --bin backend"
        echo -e "${GREEN}Start Command:${NC} cd backend && ../target/release/backend"
        echo -e "${GREEN}Health Check Path:${NC} /health"
        echo ""
        echo -e "${GREEN}Environment Variables:${NC}"
        echo "  RUST_LOG = info"
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        
        # 打开浏览器
        open "https://dashboard.render.com/create?type=web" 2>/dev/null || \
        xdg-open "https://dashboard.render.com/create?type=web" 2>/dev/null || \
        echo -e "${YELLOW}请手动访问: https://dashboard.render.com/create?type=web${NC}"
        
        echo ""
        read -p "部署完成后，请输入后端 URL (例如: https://hjjs-backend.onrender.com): " BACKEND_URL
        ;;
    2)
        ./deploy-to-render.sh
        read -p "请输入后端 URL: " BACKEND_URL
        ;;
    3)
        read -p "请输入现有的后端 URL: " BACKEND_URL
        ;;
    *)
        echo -e "${RED}无效选择${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✅ 后端 URL: $BACKEND_URL${NC}"
echo ""

# 步骤 3: 配置前端
echo -e "${BLUE}📋 步骤 3/4: 配置前端连接后端${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ -n "$BACKEND_URL" ]; then
    ./configure-frontend.sh "$BACKEND_URL"
else
    echo -e "${RED}❌ 后端 URL 为空，无法配置前端${NC}"
    exit 1
fi

# 步骤 4: 验证部署
echo ""
echo -e "${BLUE}📋 步骤 4/4: 验证部署${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}🔍 测试后端健康检查...${NC}"
if curl -s -f -m 10 "$BACKEND_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 后端服务正常${NC}"
    curl -s "$BACKEND_URL/health" | jq '.' 2>/dev/null || curl -s "$BACKEND_URL/health"
else
    echo -e "${YELLOW}⚠️  后端可能还在启动中，请稍后手动验证${NC}"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}🎉 部署流程完成！${NC}"
echo ""
echo -e "${BLUE}📊 部署摘要：${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  ${GREEN}后端 URL:${NC} $BACKEND_URL"
echo -e "  ${GREEN}前端 URL:${NC} https://huangji-jingshi.vercel.app"
echo ""
echo -e "${BLUE}🔍 验证步骤：${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 访问后端: $BACKEND_URL/health"
echo "2. 访问前端: https://huangji-jingshi.vercel.app/tools"
echo "3. 打开浏览器控制台 (F12) → Network 标签"
echo "4. 检查 API 请求返回 200 状态码"
echo "5. 确认没有 CORS 错误"
echo ""
echo -e "${BLUE}📝 注意事项：${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "- Render 免费版首次部署需要 10-15 分钟"
echo "- Render 免费版 15 分钟无请求会休眠"
echo "- 首次请求可能较慢（冷启动）"
echo ""
echo -e "${GREEN}✨ 感谢使用皇极经世部署脚本！${NC}"
echo ""

