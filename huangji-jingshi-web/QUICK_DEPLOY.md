# 快速部署指南

## 方法 1: 使用自动化脚本（推荐）

```bash
cd huangji-jingshi-web
./auto-deploy.sh
```

脚本会自动：
1. ✅ 检查 Node.js 和 npm
2. ✅ 安装依赖
3. ✅ 构建项目
4. ✅ 安装/检查 Vercel CLI
5. ✅ 登录 Vercel（如果需要）
6. ✅ 部署到生产环境

---

## 方法 2: 手动部署步骤

### 步骤 1: 安装 Vercel CLI

```bash
npm install -g vercel
```

### 步骤 2: 登录 Vercel

```bash
vercel login
```

### 步骤 3: 构建项目

```bash
cd huangji-jingshi-web/frontend
npm ci
npm run build
```

### 步骤 4: 部署

```bash
cd ..
vercel --prod
```

---

## 方法 3: 通过 GitHub + Vercel（最推荐）

### 步骤 1: 推送代码到 GitHub

```bash
git add .
git commit -m "准备部署"
git push origin main
```

### 步骤 2: 在 Vercel 网站部署

1. 访问 https://vercel.com
2. 点击 "New Project"
3. 导入 GitHub 仓库
4. 配置：
   - **Root Directory**: `huangji-jingshi-web/frontend`
   - **Build Command**: `npm ci && npm run build`
   - **Output Directory**: `dist`
   - **Framework Preset**: Vite
5. 添加环境变量：
   - `VITE_SUPABASE_URL` = `https://nunotqruohnfoozorqiz.supabase.co`
   - `VITE_SUPABASE_ANON_KEY` = 你的 anon key
6. 点击 "Deploy"

---

## 环境变量配置

### 必需的环境变量：

- `VITE_SUPABASE_URL` - Supabase 项目 URL
  - 值: `https://nunotqruohnfoozorqiz.supabase.co`

- `VITE_SUPABASE_ANON_KEY` - Supabase anon key
  - 在 Supabase Dashboard > Project Settings > API 中获取

### 在 Vercel 中设置环境变量：

1. 进入项目 Dashboard
2. 点击 "Settings" > "Environment Variables"
3. 添加上述变量
4. 选择环境（Production, Preview, Development）
5. 保存后重新部署

---

## 验证部署

部署成功后：

1. ✅ 访问 Vercel 提供的 URL
2. ✅ 检查页面是否正常加载
3. ✅ 测试登录功能
4. ✅ 测试头像上传功能

---

## 故障排除

### 构建失败

- 检查 Node.js 版本（需要 18+）
- 检查环境变量是否正确设置
- 查看 Vercel 构建日志

### 运行时错误

- 检查浏览器控制台错误
- 确认环境变量已正确设置
- 检查 Supabase 配置

### 头像上传失败

- 确认 Supabase Storage bucket `avatars` 已创建
- 确认 Storage Policies 已设置
- 检查浏览器控制台的错误信息

---

## 持续部署

如果使用 GitHub + Vercel：

- ✅ 每次推送到 main 分支会自动部署
- ✅ Pull Request 会创建预览部署
- ✅ 可以设置自定义域名

