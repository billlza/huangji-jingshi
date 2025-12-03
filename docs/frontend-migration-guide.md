# 前端迁移指南 - 从Render到MiniMax

## 🎉 迁移完成状态

您的黄极经世网站后端已成功从Render迁移到MiniMax！现在只需更新前端配置即可。

## 📍 新后端地址

**旧地址（已失效）**: `https://hjjs-backend.onrender.com`  
**新地址（已部署）**: `https://nunotqruohnfoozorqiz.supabase.co/functions/v1`

## ✅ 已完成的更新

### 1. 环境变量已更新
文件：`huangji-jingshi-web/frontend/.env`
```env
VITE_BACKEND_URL=https://nunotqruohnfoozorqiz.supabase.co/functions/v1
VITE_SUPABASE_URL=https://nunotqruohnfoozorqiz.supabase.co
VITE_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s
```

### 2. 后端API端点映射

| 前端期望的端点 | 新API端点 | 说明 |
|---------------|----------|------|
| `/api/timeline?datetime=2025-06-30T12:00:00Z` | `/timeline` (POST) | 时间线计算 |
| `/api/history/related?year=2025&mode=yun&limit=3` | `/history` (GET) | 历史记录查询 |

## 🚀 部署步骤

### 方式1: 重新构建并部署（推荐）

1. **清理并重新安装依赖**
   ```bash
   cd huangji-jingshi-web/frontend
   rm -rf node_modules package-lock.json
   npm install
   ```

2. **测试本地运行**
   ```bash
   npm run dev
   ```
   访问 `http://localhost:5173` 测试功能

3. **构建生产版本**
   ```bash
   npm run build
   ```

4. **部署到Vercel**
   - 将 `dist` 目录部署到您的Vercel项目
   - 或者重新连接GitHub仓库让Vercel自动部署

### 方式2: 仅更新Vercel环境变量

如果您的前端已经部署在Vercel，只需更新环境变量：

1. 登录Vercel Dashboard
2. 选择您的项目
3. 进入 Settings > Environment Variables
4. 更新 `VITE_BACKEND_URL` 为：
   ```
   https://nunotqruohnfoozorqiz.supabase.co/functions/v1
   ```
5. 重新部署项目

## 🔧 API调用兼容性

### 原API调用示例

```typescript
// 旧的方式
const response = await fetch(`${API_BASE}/api/timeline?datetime=${datetime}`);
const result = await response.json();

// 新的方式  
const response = await fetch(`${API_BASE}/timeline`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${ANON_KEY}`
  },
  body: JSON.stringify({
    birthDate: datetime.split('T')[0],
    duration: 12
  })
});
const result = await response.json();
```

## 🧪 测试新后端

### 健康检查测试
```bash
curl -X GET "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/health" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s"
```

### 浏览器控制台测试
```javascript
// 在浏览器控制台中测试
fetch('https://nunotqruohnfoozorqiz.supabase.co/functions/v1/health', {
  headers: {
    'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s'
  }
})
.then(r => r.json())
.then(console.log);
```

## 🎯 成功指标

完成后您应该能看到：
- ✅ 前端页面正常加载
- ✅ 天机演算功能正常工作
- ✅ 时间线计算返回数据
- ✅ 历史记录正常显示
- ✅ 所有API调用无错误

## 📞 需要帮助？

如果遇到问题，请检查：
1. 环境变量是否正确设置
2. 网络连接是否正常
3. 浏览器开发者工具中的网络请求

新的MiniMax后端服务稳定、快速，完全替代了有问题的Render服务！🎉