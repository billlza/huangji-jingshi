# 皇极经世网站部署指南

## 当前状态
- ✅ 项目文件已准备好
- ✅ Supabase已配置并授权
- ❌ 前端构建需要Node.js 20.19+，当前环境为18.19.0

## 推荐部署方案

### 方案1: 使用Vercel一键部署（推荐）

1. **上传到GitHub**
   ```bash
   git init
   git add .
   git commit -m "皇极经世网站源码"
   git branch -M main
   git remote add origin <your-github-repo>
   git push -u origin main
   ```

2. **在Vercel部署**
   - 访问 https://vercel.com
   - 点击 "New Project"
   - 导入GitHub仓库
   - 设置根目录：`huangji-jingshi-web/frontend`
   - 构建命令：`npm ci && npm run build`
   - 输出目录：`dist`
   - 环境变量：
     - `VITE_SUPABASE_URL` = `https://nunotqruohnfoozorqiz.supabase.co`
     - `VITE_SUPABASE_ANON_KEY` = 从Supabase仪表板获取

### 方案2: 本地构建后部署

1. **要求**
   - Node.js 20.19+ 或更高版本
   - npm 或 yarn

2. **构建步骤**
   ```bash
   cd huangji-jingshi-web/frontend
   npm install
   npm run build
   ```

3. **部署dist目录到任何静态托管服务**

### 方案3: 使用Docker部署
项目包含Dockerfile，可使用容器化部署。

## 环境变量
- `VITE_SUPABASE_URL`: Supabase项目URL
- `VITE_SUPABASE_ANON_KEY`: Supabase匿名密钥
- `VITE_BACKEND_URL`: 后端API URL（部署后端后设置）

## 注意事项
1. 确保Supabase存储桶"avatars"已创建
2. 验证Supabase存储策略已配置
3. 部署后端服务后，更新前端环境变量
