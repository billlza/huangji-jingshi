# 后端部署指南 - Render

## 快速部署后端到 Render

### 步骤 1: 登录 Render
访问 https://render.com 并登录

### 步骤 2: 部署后端服务

**方法 A: 使用 Blueprint（推荐）**

1. 在 Render Dashboard 点击 "New +" > "Blueprint"
2. 连接 GitHub 仓库：`billlza/huangji-jingshi`
3. Render 会自动读取 `render.yaml` 配置
4. 设置环境变量：
   - `SUPABASE_URL` = `https://nunotqruohnfoozorqiz.supabase.co`
   - `SUPABASE_KEY` = 您的 service_role key（之前给过）
5. 点击 "Apply"

**方法 B: 手动创建 Web Service**

1. 点击 "New +" > "Web Service"
2. 连接 GitHub 仓库：`billlza/huangji-jingshi`
3. 配置：
   - **Name**: `hjjs-backend`
   - **Root Directory**: `huangji-jingshi-web`
   - **Environment**: `Rust`
   - **Build Command**: `cd backend && cargo build --release`
   - **Start Command**: `cd backend && ./target/release/backend`
   - **Plan**: Free（或您选择的计划）
4. 添加环境变量：
   - `SUPABASE_URL` = `https://nunotqruohnfoozorqiz.supabase.co`
   - `SUPABASE_KEY` = `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc2NDQ2NjY2MiwiZXhwIjoyMDgwMDQyNjYyfQ.a_-2C0z0jgAu3bD-BFBG4ONi1kKyz7fn4cnSlBlY-eE`
   - `RUST_LOG` = `info`
5. 点击 "Create Web Service"

### 步骤 3: 获取后端 URL

部署完成后，Render 会提供一个 URL，例如：
`https://hjjs-backend.onrender.com`

### 步骤 4: 更新前端环境变量

在 Vercel Dashboard 中：
1. 进入项目设置 > Environment Variables
2. 添加或更新：
   - **Key**: `VITE_BACKEND_URL`
   - **Value**: 后端 URL（例如：`https://hjjs-backend.onrender.com`）
3. 重新部署前端

---

## 后端环境变量清单

| Key | Value |
|-----|-------|
| `SUPABASE_URL` | `https://nunotqruohnfoozorqiz.supabase.co` |
| `SUPABASE_KEY` | `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` (service_role key) |
| `RUST_LOG` | `info` (可选) |

---

## 注意事项

1. **首次部署可能需要 10-15 分钟**（编译 Rust 项目）
2. **免费计划有冷启动**：如果 15 分钟无请求，服务会休眠，首次请求会较慢
3. **CORS 已配置**：后端已设置允许所有来源的跨域请求

