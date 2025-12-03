# Render 后端部署完整指南

## ✅ 代码修复完成

已修复以下问题：
1. ✅ 添加缺失的 `once_cell::sync::Lazy` 导入
2. ✅ 修复字符串拼接语法错误
3. ✅ 优化 CORS 配置（使用 permissive 模式）
4. ✅ 改进数据文件加载逻辑
5. ✅ 更新 Render 配置文件

## 🚀 部署步骤

### 方法 1: 使用 Blueprint（推荐，最简单）

1. **登录 Render**
   - 访问 https://dashboard.render.com
   - 使用 GitHub 账号登录

2. **创建 Blueprint**
   - 点击 "New +" → "Blueprint"
   - 连接你的 GitHub 仓库：`billlza/huangji-jingshi`（或你的仓库名）
   - Render 会自动检测项目根目录的 `render.yaml`

3. **配置文件选择**
   - Render 会显示检测到的 `render.yaml` 文件
   - 使用项目根目录的 `render.yaml` 或 `render-deploy.yaml`

4. **点击 "Apply"**
   - Render 会自动开始构建和部署

### 方法 2: 手动创建 Web Service

1. **创建新服务**
   - 在 Render Dashboard，点击 "New +" → "Web Service"
   - 连接 GitHub 仓库

2. **配置服务**
   ```
   Name: hjjs-backend
   Environment: Rust
   Branch: main
   Root Directory: huangji-jingshi-web
   Build Command: cd backend && cargo build --release --bin backend
   Start Command: cd backend && ./target/release/backend
   Plan: Free
   ```

3. **高级设置**
   - **Health Check Path**: `/health`
   - **Auto-Deploy**: Yes

4. **环境变量**
   ```
   RUST_LOG=info
   ```
   注意：PORT 会自动由 Render 提供，不需要手动设置

5. **创建服务**
   - 点击 "Create Web Service"
   - 等待首次构建（大约 10-15 分钟）

## 📝 重要配置说明

### 端口配置
Render 会自动提供 `PORT` 环境变量，代码会自动读取：
```rust
let port = env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse::<u16>()
    .unwrap_or(3000);
```

### 健康检查
后端提供 `/health` 端点用于健康检查：
```
GET https://your-service.onrender.com/health
```

### 数据文件
如果需要数据文件，确保它们在正确的位置：
```
huangji-jingshi-web/
  backend/
    data/
      celestial/
      history.json
      major_events.json
```

## 🔍 部署后验证

### 1. 检查构建日志
在 Render Dashboard 中查看 "Logs" 标签页，确保：
- ✅ Cargo 成功下载依赖
- ✅ 编译成功完成
- ✅ 服务成功启动
- ✅ 看到 "🚀 皇极经世后端服务启动中..."

### 2. 测试健康检查
```bash
curl https://your-service.onrender.com/health
```

预期响应：
```json
{
  "status": "ok",
  "message": "皇极经世后端服务正常运行",
  "timestamp": "2025-12-03T...",
  "version": "1.0.0-fixed",
  "data_loaded": false
}
```

### 3. 测试根路径
```bash
curl https://your-service.onrender.com/
```

预期响应：
```json
{
  "service": "皇极经世后端服务",
  "status": "running",
  "version": "1.0.0-fixed",
  "endpoints": [...]
}
```

### 4. 测试 API 端点
```bash
curl -X POST https://your-service.onrender.com/api/calculate \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
```

## ⚡ 获取后端 URL

部署成功后，你的后端 URL 格式为：
```
https://hjjs-backend.onrender.com
```

或者
```
https://[你的服务名].onrender.com
```

## 🔧 配置前端连接后端

### 步骤 1: 在 Vercel 添加环境变量

1. 访问 https://vercel.com/dashboard
2. 选择你的项目 `huangji-jingshi`
3. Settings → Environment Variables
4. 添加：
   ```
   Key: VITE_BACKEND_URL
   Value: https://hjjs-backend.onrender.com
   Environments: Production, Preview, Development
   ```

### 步骤 2: 重新部署前端

在 Vercel Dashboard：
- Deployments → 最新部署 → "..." → "Redeploy"

或者推送代码触发自动部署。

## 🐛 常见问题排查

### 问题 1: 构建失败 "Lazy not found"
**原因**: 缺少 `once_cell` 导入  
**解决**: 已修复，确保使用最新代码

### 问题 2: 服务启动后立即崩溃
**检查**:
- 查看 Render 日志中的错误信息
- 确保 `cargo build --release --bin backend` 成功
- 检查 `./target/release/backend` 文件是否存在

### 问题 3: 404 错误
**检查**:
- 确保服务状态是 "Live"（绿色）
- 确保 URL 正确
- 检查 Health Check 是否通过

### 问题 4: 502 Bad Gateway
**原因**: 服务可能还在启动中  
**解决**: 等待 1-2 分钟，服务需要时间启动

### 问题 5: 冷启动慢
**原因**: 免费计划在 15 分钟无请求后会休眠  
**解决方案**:
- 升级到付费计划（$7/月）
- 或使用 UptimeRobot 每 5 分钟 ping 一次

## 📊 监控和维护

### 查看日志
Render Dashboard → 你的服务 → Logs

### 重新部署
Render Dashboard → 你的服务 → Manual Deploy → Deploy latest commit

### 查看指标
Render Dashboard → 你的服务 → Metrics
- CPU 使用率
- 内存使用率
- 请求数量

## 🎉 成功标志

当你看到以下内容时，说明部署成功：
- ✅ Render 服务状态显示 "Live"（绿色）
- ✅ `/health` 端点返回 200 OK
- ✅ Vercel 前端可以成功调用后端 API
- ✅ 浏览器控制台没有 CORS 错误

## 📞 需要帮助？

如果遇到问题：
1. 检查 Render 构建日志
2. 检查 Render 运行日志
3. 测试 `/health` 端点
4. 检查环境变量配置
5. 确认使用最新的代码

---

部署愉快！🚀

