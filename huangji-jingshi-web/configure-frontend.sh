#!/bin/bash

# 🔗 前端配置脚本
# 用于配置 Vercel 前端连接到 Render 后端

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "🔗 配置 Vercel 前端连接后端"
echo "================================================"
echo ""

# 检查参数
if [ -z "$1" ]; then
    echo -e "${RED}❌ 错误：请提供后端 URL${NC}"
    echo ""
    echo "用法: ./configure-frontend.sh <后端URL>"
    echo "示例: ./configure-frontend.sh https://hjjs-backend.onrender.com"
    echo ""
    exit 1
fi

BACKEND_URL=$1

# 验证 URL 格式
if [[ ! $BACKEND_URL =~ ^https?:// ]]; then
    echo -e "${RED}❌ 错误：URL 必须以 http:// 或 https:// 开头${NC}"
    exit 1
fi

echo -e "${BLUE}后端 URL: ${GREEN}$BACKEND_URL${NC}"
echo ""

# 测试后端健康检查
echo -e "${BLUE}🔍 测试后端连接...${NC}"
if curl -s -f -m 10 "$BACKEND_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 后端服务正常运行${NC}"
    
    # 显示健康检查响应
    HEALTH_RESPONSE=$(curl -s "$BACKEND_URL/health")
    echo -e "${BLUE}健康检查响应:${NC}"
    echo "$HEALTH_RESPONSE" | jq '.' 2>/dev/null || echo "$HEALTH_RESPONSE"
    echo ""
else
    echo -e "${YELLOW}⚠️  警告：无法连接到后端（可能还在启动中）${NC}"
    echo ""
    read -p "是否继续配置前端？(y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 检查是否安装了 Vercel CLI
echo -e "${BLUE}🔍 检查 Vercel CLI...${NC}"
if command -v vercel &> /dev/null; then
    echo -e "${GREEN}✅ Vercel CLI 已安装${NC}"
    VERCEL_CLI_INSTALLED=true
else
    echo -e "${YELLOW}⚠️  Vercel CLI 未安装${NC}"
    VERCEL_CLI_INSTALLED=false
fi
echo ""

if [ "$VERCEL_CLI_INSTALLED" = true ]; then
    echo -e "${BLUE}🔧 配置 Vercel 环境变量...${NC}"
    echo ""
    
    # 切换到前端目录
    cd frontend
    
    # 设置环境变量
    echo -e "${BLUE}设置 VITE_BACKEND_URL...${NC}"
    vercel env add VITE_BACKEND_URL production <<< "$BACKEND_URL" || true
    vercel env add VITE_BACKEND_URL preview <<< "$BACKEND_URL" || true
    vercel env add VITE_BACKEND_URL development <<< "$BACKEND_URL" || true
    
    echo ""
    echo -e "${GREEN}✅ 环境变量已设置${NC}"
    echo ""
    
    # 询问是否重新部署
    read -p "是否立即重新部署前端？(y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}🚀 正在重新部署...${NC}"
        vercel --prod
        echo ""
        echo -e "${GREEN}🎉 前端部署完成！${NC}"
    else
        echo -e "${YELLOW}💡 稍后可以运行以下命令重新部署：${NC}"
        echo -e "${BLUE}   cd frontend && vercel --prod${NC}"
    fi
    
    cd ..
else
    echo -e "${YELLOW}📖 请手动配置 Vercel 环境变量：${NC}"
    echo ""
    echo "1️⃣  访问: https://vercel.com/dashboard"
    echo "2️⃣  选择项目: huangji-jingshi"
    echo "3️⃣  Settings → Environment Variables"
    echo "4️⃣  添加变量："
    echo ""
    echo -e "${GREEN}   Key:${NC} VITE_BACKEND_URL"
    echo -e "${GREEN}   Value:${NC} $BACKEND_URL"
    echo -e "${GREEN}   Environments:${NC} Production, Preview, Development (全选)"
    echo ""
    echo "5️⃣  点击 Save"
    echo "6️⃣  Deployments → 最新部署 → ... → Redeploy"
    echo ""
    
    # 询问是否打开浏览器
    read -p "是否打开 Vercel Dashboard？(y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}🌐 正在打开浏览器...${NC}"
        open "https://vercel.com/dashboard" 2>/dev/null || \
        xdg-open "https://vercel.com/dashboard" 2>/dev/null || \
        echo -e "${YELLOW}请手动访问: https://vercel.com/dashboard${NC}"
    fi
fi

echo ""
echo -e "${GREEN}✅ 配置完成！${NC}"
echo ""
echo -e "${BLUE}📋 配置摘要：${NC}"
echo -e "   后端 URL: ${GREEN}$BACKEND_URL${NC}"
echo -e "   前端 URL: ${GREEN}https://huangji-jingshi.vercel.app${NC}"
echo ""
echo -e "${BLUE}🔍 验证步骤：${NC}"
echo "1. 访问前端: https://huangji-jingshi.vercel.app/tools"
echo "2. 打开浏览器控制台 (F12) → Network 标签"
echo "3. 检查 API 请求是否返回 200 状态码"
echo "4. 确认没有 CORS 错误"
echo ""
echo -e "${GREEN}🎉 大功告成！${NC}"
echo ""

