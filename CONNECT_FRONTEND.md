# 🔗 连接前端到后端

## ✅ 后端状态

- ✅ **后端已成功部署**
- ✅ **后端 URL**: `https://hjjs-backend.onrender.com`
- ✅ **健康检查**: 正常
- ✅ **所有 API 端点**: 正常工作

**测试结果：**
```json
{
  "status": "ok",
  "message": "皇极经世后端服务正常运行",
  "version": "1.0.0-fixed"
}
```

---

## ❌ 前端问题

前端代码需要 `VITE_BACKEND_URL` 环境变量，但 Vercel 上还没配置。

**前端代码中的使用：**
```typescript
// Tools.tsx
const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
fetch(`${API_BASE}/api/sky-and-fortune?${q}`)

// Timeline.tsx  
const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
fetch(`${API_BASE}/api/timeline?datetime=${currentYear}`)

// Dashboard.tsx
const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
fetch(`${API_BASE}/api/timeline?datetime=${y}`)
```

**如果 `VITE_BACKEND_URL` 为空**，会调用 `/api/xxx`（相对路径），而 Vercel 上没有这些 API！

---

## 🔧 解决方案：在 Vercel 配置环境变量

### 步骤 1: 打开项目设置

1. 访问 https://vercel.com/dashboard
2. 找到并点击 `huangji-jingshi` 项目
3. 点击顶部的 **"Settings"** 标签

### 步骤 2: 添加环境变量

1. 在左侧菜单点击 **"Environment Variables"**
2. 点击 **"Add New"** 或 **"Add Another"** 按钮
3. 填写：

```
Name:  VITE_BACKEND_URL
Value: https://hjjs-backend.onrender.com
```

4. **Environments** 选择：
   - ✅ Production
   - ✅ Preview  
   - ✅ Development

5. 点击 **"Save"** 按钮

### 步骤 3: 重新部署前端

环境变量需要重新部署才能生效：

1. 点击顶部的 **"Deployments"** 标签
2. 找到最新的部署
3. 点击右侧的 **"..."** 按钮
4. 选择 **"Redeploy"**
5. **确认** "Redeploy"
6. 等待 2-3 分钟

---

## ✅ 验证成功

### 1. 检查构建日志

在 Vercel 部署日志中应该看到：
```
Environment Variables
  VITE_BACKEND_URL: https://hjjs-backend.onrender.com
```

### 2. 访问前端

访问：https://huangji-jingshi.vercel.app/tools

### 3. 打开浏览器控制台

按 `F12` 打开开发者工具，切换到 **Console** 标签

**成功的日志：**
```
[StarMap] Using local static: /data/
API 请求成功
```

**不再看到：**
```
❌ Failed to fetch
❌ 404 Not Found
❌ CORS error
```

### 4. 检查 Network

在 **Network** 标签中：
- ✅ 请求 `https://hjjs-backend.onrender.com/api/...` 返回 **200**
- ✅ Response 有数据
- ✅ 没有 CORS 错误

---

## 📋 完整的环境变量列表

确保 Vercel 上配置了以下环境变量：

| Key | Value | 说明 |
|-----|-------|------|
| `VITE_BACKEND_URL` | `https://hjjs-backend.onrender.com` | 后端 API URL |
| `VITE_SUPABASE_URL` | `https://nunotqruohnfoozorqiz.supabase.co` | Supabase URL |
| `VITE_SUPABASE_ANON_KEY` | (你的 key) | Supabase 密钥 |

---

## 🐛 故障排查

### 问题 1: 前端仍然连接失败

**检查：**
1. Vercel 环境变量是否保存成功
2. 是否重新部署了前端
3. 浏览器是否缓存了旧代码（硬刷新 Ctrl+Shift+R）

### 问题 2: CORS 错误

**解决：** 后端已配置 `CorsLayer::permissive()`，应该没有 CORS 问题。
如果还有，检查：
1. 确认后端 URL 正确（https，不是 http）
2. 清除浏览器缓存

### 问题 3: 404 错误

**原因：** 可能是 API 路径不对
**检查：** 确认调用的是 `/api/xxx`，不是 `/xxx`

---

## 🎯 快速测试后端

```bash
# 健康检查
curl https://hjjs-backend.onrender.com/health

# Timeline API
curl https://hjjs-backend.onrender.com/api/timeline?datetime=2025-01-01T12:00:00Z

# Calculate API
curl -X POST https://hjjs-backend.onrender.com/api/calculate \
  -H "Content-Type: application/json" \
  -d '{"test":"data"}'
```

所有应该返回 **200 OK**！

---

## 📞 下一步

1. **现在就去 Vercel 配置环境变量**
2. **重新部署前端**
3. **告诉我结果**，我帮你验证！

**Vercel Dashboard 已经打开，现在就配置吧！** 🚀

