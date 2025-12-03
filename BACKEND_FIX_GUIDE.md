# 后端部署修复指南

## 📋 当前状态
✅ Node.js后端已创建 (`server.js` + `package.json`)
✅ Render配置已修复 (`render.yaml`)
✅ 使用原服务名 `hjjs-backend`
✅ 代码已准备就绪

## 🚀 立即解决步骤

### 方法1：手动触发Render部署
1. 访问你的Render Dashboard: https://dashboard.render.com/project/prj-d4ngq0m3jp1c73am1u60
2. 点击你的 `hjjs-backend` 服务
3. 点击 "Manual Deploy" > "Deploy latest commit"

### 方法2：检查构建日志
如果部署失败，请查看构建日志中的错误信息。

## 🔧 预期结果
成功后端应响应以下端点：
- `GET /health` - 健康检查
- `POST /calculate` - 天机演算
- `POST /timeline` - 时间线推算  
- `GET /sky` - 天象数据
- `GET /history` - 历史记录

## 📊 配置详情

### render.yaml
```yaml
services:
  - type: web
    name: hjjs-backend           # 原服务名
    env: node                    # Node.js环境
    runtime: node
    rootDir: backend             # 正确的路径
    buildCommand: npm install
    startCommand: npm start
```

### package.json
```json
{
  "name": "huangji-jingshi-backend",
  "main": "server.js",
  "scripts": {
    "start": "node server.js"
  },
  "dependencies": {
    "express": "^4.18.2",
    "cors": "^2.8.5"
  }
}
```

## ⚡ 快速测试
部署完成后，用以下URL测试：
- https://hjjs-backend.onrender.com/health
- https://hjjs-backend.onrender.com/sky

## 🔗 前端更新
成功后需要更新Vercel环境变量：
- `VITE_BACKEND_URL` → `https://hjjs-backend.onrender.com`