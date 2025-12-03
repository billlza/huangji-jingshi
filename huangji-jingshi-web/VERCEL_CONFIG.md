# Vercel 部署配置说明

## 当前配置

### 前端部署
- **平台**: Vercel
- **URL**: `https://huangji-jingshi.vercel.app`
- **配置**: `vercel.json`

### 后端部署
根据您的说明，后端也部署在 Vercel。当前配置通过 Vercel 的 `rewrites` 功能将 `/api/*` 请求代理到后端。

## 配置选项

### 选项 1: 后端在 Render（当前配置）
如果后端仍在 Render (`https://hjjs-backend.onrender.com`)，`vercel.json` 已配置代理：

```json
{
  "rewrites": [
    {
      "source": "/api/(.*)",
      "destination": "https://hjjs-backend.onrender.com/api/$1"
    }
  ]
}
```

**环境变量配置（Vercel Dashboard）**：
- `VITE_BACKEND_URL`: **留空**（使用相对路径 `/api/*`，由 Vercel rewrites 代理）

### 选项 2: 后端在 Vercel（如果后端真的在 Vercel）
如果后端通过 Vercel Serverless Functions 或其他方式部署在 Vercel：

1. **更新 `vercel.json`**，将 rewrites 指向 Vercel 后端 URL：
```json
{
  "rewrites": [
    {
      "source": "/api/(.*)",
      "destination": "https://your-backend.vercel.app/api/$1"
    }
  ]
}
```

2. **或者**，如果后端在同一 Vercel 项目中，可以直接使用相对路径（无需 rewrites）

**环境变量配置（Vercel Dashboard）**：
- `VITE_BACKEND_URL`: **留空**（使用相对路径）

## 必需的环境变量

在 Vercel Dashboard > Settings > Environment Variables 中配置：

| Key | Value | 说明 |
|-----|-------|------|
| `VITE_SUPABASE_URL` | `https://nunotqruohnfoozorqiz.supabase.co` | Supabase 项目 URL |
| `VITE_SUPABASE_ANON_KEY` | 您的 Supabase anon key | Supabase 匿名密钥 |
| `VITE_BACKEND_URL` | （可选）后端 URL | 如果留空，使用相对路径 `/api/*` |

## 验证部署

1. **检查前端**：
   - 访问：https://huangji-jingshi.vercel.app/tools
   - 应该正常显示页面

2. **检查后端 API**：
   - 打开浏览器控制台（F12）
   - 查看 Network 标签页
   - API 请求应该返回 200 状态码，而不是 404 或 502

3. **测试 API 端点**：
   - 访问：`https://huangji-jingshi.vercel.app/api/timeline?datetime=2025-01-01T12:00:00Z`
   - 应该返回 JSON 数据

## 如果后端真的在 Vercel

如果您的 Rust 后端确实部署在 Vercel（通过特殊方式），请提供：
1. 后端的 Vercel URL
2. 后端部署方式（Serverless Functions、Edge Functions 等）

我可以根据实际情况更新配置。


