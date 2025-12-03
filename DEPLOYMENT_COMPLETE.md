# 皇极经世网站部署完成指南

## 🎉 项目部署准备就绪

您的皇极经世网站已经完成所有准备工作！以下是详细的部署指南：

## 📋 项目概述

**技术栈：**
- 前端：React + TypeScript + Vite + Tailwind CSS
- 后端：Rust + Axum Web框架
- 数据库：Supabase（已配置并认证）
- 静态资源：天文数据和皇极经世推算数据

**核心功能：**
- 皇极经世推算和展示
- 天文计算和星象显示
- 历史事件查询和关联
- 用户账户和头像上传
- 响应式Web界面

## 🚀 部署选项

### 选项1: 使用GitHub + Vercel（推荐）

#### 前端部署到Vercel：

1. **上传代码到GitHub**
   ```bash
   cd huangji-jingshi-web
   git init
   git add .
   git commit -m "皇极经世网站 - 准备部署"
   git branch -M main
   git remote add origin <your-github-repo-url>
   git push -u origin main
   ```

2. **在Vercel部署前端**
   - 访问 https://vercel.com
   - 点击 "New Project"
   - 导入GitHub仓库
   - 配置设置：
     - **Root Directory**: `huangji-jingshi-web/frontend`
     - **Build Command**: `npm ci && npm run build`
     - **Output Directory**: `dist`
     - **Framework Preset**: Vite
   - 添加环境变量：
     - `VITE_SUPABASE_URL`: `https://nunotqruohnfoozorqiz.supabase.co`
     - `VITE_SUPABASE_ANON_KEY`: 从Supabase仪表板获取
   - 点击 "Deploy"

#### 后端部署到Render：

1. **上传代码到GitHub**（如果还没有）

2. **在Render部署后端**
   - 访问 https://render.com
   - 点击 "New +" > "Blueprint"
   - 连接GitHub仓库
   - Render会自动读取`render.yaml`配置
   - 设置环境变量：
     - `SUPABASE_URL`: `https://nunotqruohnfoozorqiz.supabase.co`
     - `SUPABASE_KEY`: 您的service_role key
     - `RUST_LOG`: `info`
   - 点击 "Apply"

3. **更新前端环境变量**
   在Vercel中，添加后端URL：
   - `VITE_BACKEND_URL`: `https://your-backend-url.onrender.com`
   - 重新部署前端

### 选项2: 本地构建部署

#### 系统要求：
- Node.js 20.19+ （前端）
- Rust 1.70+ （后端）
- Git

#### 前端部署：
```bash
cd huangji-jingshi-web/frontend

# 安装依赖
npm install

# 构建项目
npm run build

# 部署dist目录到任何静态托管服务
# 如：Netlify, AWS S3, GitHub Pages等
```

#### 后端部署：
```bash
cd huangji-jingshi-web/backend

# 构建Rust项目
cargo build --release

# 运行服务
./target/release/backend
```

### 选项3: Docker部署

项目已包含Dockerfile：
```bash
# 构建镜像
docker build -t huangji-jingshi .

# 运行容器
docker run -p 8080:8080 huangji-jingshi
```

## ✅ 验证部署

部署完成后，请验证以下功能：

1. **首页访问**
   - 确认页面正常加载
   - 检查CSS和JS资源加载

2. **用户认证**
   - 测试注册功能
   - 测试登录功能
   - 测试头像上传功能

3. **皇极经世功能**
   - 测试日期选择和推算
   - 验证天文数据显示
   - 检查历史事件查询

4. **响应式设计**
   - 测试移动端适配
   - 检查各种屏幕尺寸

## 🔧 环境变量说明

### 前端环境变量：
- `VITE_SUPABASE_URL`: Supabase项目URL
- `VITE_SUPABASE_ANON_KEY`: Supabase匿名密钥
- `VITE_BACKEND_URL`: 后端API URL（可选）

### 后端环境变量：
- `SUPABASE_URL`: Supabase项目URL
- `SUPABASE_KEY`: Supabase服务密钥
- `RUST_LOG`: 日志级别（推荐：info）
- `TIMEZONEDB_KEY`: 时区数据库API密钥（可选）
- `DELTA_T_PROVIDER_DEFAULT`: Delta-T数据提供商（可选）
- `ASTRO_ACCURACY_DEFAULT`: 天文计算精度（可选）

## 📁 项目结构

```
huangji-jingshi-web/
├── frontend/                 # React前端
│   ├── src/                 # 源代码
│   ├── public/              # 静态资源
│   ├── dist/                # 构建输出
│   └── package.json         # 依赖配置
├── backend/                  # Rust后端
│   ├── src/                 # 源代码
│   ├── data/                # 数据文件
│   └── Cargo.toml           # Rust配置
├── huangji_core/             # 核心算法库
└── 配置文件
    ├── render.yaml          # Render部署配置
    └── vercel.json          # Vercel配置
```

## 🛠️ 自定义和扩展

### 修改配置：
1. **更新Supabase设置**：
   - 在Supabase仪表板中配置存储桶和策略
   - 设置用户认证和权限

2. **添加新功能**：
   - 前端：在`frontend/src`中添加组件
   - 后端：在`backend/src`中添加API端点

3. **数据更新**：
   - 历史事件：更新`backend/data/history.json`
   - 年份映射：更新`huangji_core/data/year_mapping.json`

## 🚨 故障排除

### 常见问题：

1. **构建失败**
   - 检查Node.js版本（需要20.19+）
   - 确认所有依赖已安装
   - 查看构建日志

2. **运行时错误**
   - 检查环境变量配置
   - 确认Supabase连接正常
   - 查看浏览器控制台错误

3. **头像上传失败**
   - 确认Supabase存储桶已创建
   - 验证存储策略配置
   - 检查用户权限设置

## 📞 支持

如果遇到部署问题：

1. 检查本指南的故障排除部分
2. 查看项目根目录的`DEPLOY.md`文件
3. 检查各平台的部署日志

## 🎯 下一步

1. 选择合适的部署方式
2. 按照指南执行部署
3. 测试所有功能
4. 配置自定义域名（可选）
5. 设置监控和备份

---

**皇极经世网站部署包已准备完成！** 🎉

现在您可以选择最适合的部署方式，将这个精美的皇极经世网站上线运行。
