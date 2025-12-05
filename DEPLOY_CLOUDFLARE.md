# Cloudflare Pages 部署指南

## 🎯 为什么选择 Cloudflare Pages？

- ✅ **中国大陆可访问**（相对 Vercel 更稳定）
- ✅ **免费无限流量**
- ✅ **全球 CDN 加速**
- ✅ **自动 HTTPS**
- ✅ **支持自定义域名**

---

## 📝 部署步骤

### 1. 准备工作

确保前端构建配置正确：

```bash
cd /Users/bill/Desktop/hjjs/huangji-jingshi-web/frontend
```

检查 `package.json` 的 build 命令：

```json
{
  "scripts": {
    "build": "vite build",
    "preview": "vite preview"
  }
}
```

### 2. 通过 Git 部署（推荐）

#### 方式 A: 使用 Cloudflare Dashboard

1. 访问 https://dash.cloudflare.com/
2. 选择 **Workers & Pages** → **Create application** → **Pages**
3. 连接你的 GitHub 仓库：`billlza/huangji-jingshi`
4. 配置构建设置：

```yaml
Framework preset: Vite
Build command: cd huangji-jingshi-web/frontend && npm run build
Build output directory: huangji-jingshi-web/frontend/dist
Root directory: /
Node version: 18
```

5. 环境变量：

```
VITE_BACKEND_URL=https://hjjs-backend.onrender.com
```

6. 点击 **Save and Deploy**

#### 方式 B: 使用 Wrangler CLI

```bash
# 安装 Wrangler
npm install -g wrangler

# 登录 Cloudflare
wrangler login

# 部署
cd /Users/bill/Desktop/hjjs/huangji-jingshi-web/frontend
npm run build
wrangler pages deploy dist --project-name=huangji-jingshi
```

---

## 🔧 配置文件

创建 `wrangler.toml`（可选）：

```toml
name = "huangji-jingshi"
compatibility_date = "2024-01-01"

[site]
bucket = "./dist"

[env.production]
vars = { VITE_BACKEND_URL = "https://hjjs-backend.onrender.com" }
```

---

## 🌐 自定义域名

### 在 Cloudflare 添加自定义域名

1. 进入 Pages 项目设置
2. 选择 **Custom domains**
3. 添加你的域名（如 `huangji.example.com`）
4. Cloudflare 会自动配置 DNS

### DNS 设置（如果域名在其他服务商）

```
类型: CNAME
名称: @  (或 huangji)
目标: your-project.pages.dev
```

---

## 📊 对比

| 平台 | 中国大陆访问 | 免费额度 | 构建时间 |
|-----|------------|---------|---------|
| **Cloudflare Pages** | ⚠️ 较好 | 无限 | 1-3分钟 |
| **Vercel** | ❌ 很差 | 100GB/月 | 30秒-2分钟 |
| **Netlify** | ⚠️ 一般 | 100GB/月 | 1-3分钟 |
| **Zeabur** | ✅ 很好 | 有限免费 | 2-5分钟 |

---

## 🚀 部署后测试

```bash
# 测试访问
curl -I https://huangji-jingshi.pages.dev

# 测试 API 连接
curl https://huangji-jingshi.pages.dev/api/health
```

---

## 📝 注意事项

1. **首次构建**可能需要 5-10 分钟
2. **环境变量**必须以 `VITE_` 开头才能在前端访问
3. **自动部署**：推送到 `main` 分支会自动触发部署
4. **预览部署**：PR 会自动创建预览环境

---

## 🔍 故障排查

### 构建失败

```bash
# 本地测试构建
cd frontend
npm install
npm run build

# 检查 dist 目录
ls -la dist/
```

### 环境变量未生效

检查环境变量名称：
- ✅ `VITE_BACKEND_URL`
- ❌ `BACKEND_URL`（不会被 Vite 读取）

### 404 错误

确保有 SPA 路由重写配置（`_redirects` 或 `vercel.json`）。

Cloudflare Pages 会自动处理 SPA 路由，无需额外配置。

---

## 📚 相关文档

- [Cloudflare Pages 官方文档](https://developers.cloudflare.com/pages/)
- [Vite 部署指南](https://vitejs.dev/guide/static-deploy.html)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)

