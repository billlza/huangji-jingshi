# 部署指南

## 部署选项

### 方案 1: Vercel（推荐 - 前端）

Vercel 提供优秀的 React/Vite 应用部署体验。

#### 步骤：

1. **安装 Vercel CLI**（如果还没有）：
   ```bash
   npm i -g vercel
   ```

2. **登录 Vercel**：
   ```bash
   vercel login
   ```

3. **在项目根目录部署**：
   ```bash
   cd huangji-jingshi-web
   vercel
   ```

4. **设置环境变量**：
   在 Vercel Dashboard > Project Settings > Environment Variables 中添加：
   - `VITE_SUPABASE_URL` = 你的 Supabase 项目 URL
   - `VITE_SUPABASE_ANON_KEY` = 你的 Supabase anon key
   - `VITE_API_BASE_URL` = 后端 API 地址（如果使用独立后端）

5. **重新部署**：
   ```bash
   vercel --prod
   ```

---

### 方案 2: Render（全栈）

Render 可以同时部署前端和后端。

#### 步骤：

1. **登录 Render Dashboard**：https://render.com

2. **部署后端**：
   - 点击 "New +" > "Web Service"
   - 连接 GitHub 仓库
   - 设置：
     - **Name**: `hjjs-backend`
     - **Root Directory**: `huangji-jingshi-web`
     - **Environment**: `Rust`
     - **Build Command**: `cd backend && cargo build --release`
     - **Start Command**: `cd backend && ./target/release/backend`
   - 添加环境变量：
     - `SUPABASE_URL`
     - `SUPABASE_KEY`
     - `RUST_LOG=info`

3. **部署前端**：
   - 点击 "New +" > "Static Site"
   - 连接 GitHub 仓库
   - 设置：
     - **Name**: `hjjs-frontend`
     - **Root Directory**: `huangji-jingshi-web/frontend`
     - **Build Command**: `npm ci && npm run build`
     - **Publish Directory**: `dist`
   - 添加环境变量：
     - `VITE_SUPABASE_URL`
     - `VITE_SUPABASE_ANON_KEY`
     - `VITE_API_BASE_URL` = 后端服务 URL（例如：`https://hjjs-backend.onrender.com`）

4. **使用 render.yaml**（推荐）：
   - 将 `render.yaml` 文件推送到 GitHub
   - 在 Render Dashboard 中选择 "New Blueprint"
   - 连接仓库，Render 会自动读取 `render.yaml` 配置

---

### 方案 3: Netlify（前端）

Netlify 也提供优秀的静态站点部署。

#### 步骤：

1. **安装 Netlify CLI**：
   ```bash
   npm i -g netlify-cli
   ```

2. **登录**：
   ```bash
   netlify login
   ```

3. **初始化项目**：
   ```bash
   cd huangji-jingshi-web/frontend
   netlify init
   ```

4. **创建 netlify.toml**：
   ```toml
   [build]
     command = "npm run build"
     publish = "dist"

   [[redirects]]
     from = "/*"
     to = "/index.html"
     status = 200
   ```

5. **设置环境变量**：
   在 Netlify Dashboard > Site Settings > Environment Variables 中添加

6. **部署**：
   ```bash
   netlify deploy --prod
   ```

---

## 环境变量清单

### 前端环境变量：
- `VITE_SUPABASE_URL` - Supabase 项目 URL
- `VITE_SUPABASE_ANON_KEY` - Supabase anon key
- `VITE_API_BASE_URL` - 后端 API 地址（可选，如果使用独立后端）

### 后端环境变量：
- `SUPABASE_URL` - Supabase 项目 URL
- `SUPABASE_KEY` - Supabase service_role key
- `RUST_LOG` - 日志级别（可选，默认 info）

---

## 构建和测试

### 本地构建测试：

```bash
# 前端
cd huangji-jingshi-web/frontend
npm ci
npm run build
npm run preview  # 预览构建结果

# 后端
cd huangji-jingshi-web/backend
cargo build --release
./target/release/backend
```

---

## 注意事项

1. **CORS 配置**：确保后端允许前端域名的跨域请求
2. **环境变量安全**：不要将敏感密钥提交到代码仓库
3. **API 代理**：如果前端和后端部署在不同域名，需要配置 API 代理或 CORS
4. **静态资源**：确保 `public/data/` 目录中的文件被正确复制到 `dist/`

---

## 快速部署（Vercel）

```bash
cd huangji-jingshi-web
vercel --prod
```

然后在 Vercel Dashboard 中设置环境变量即可。

