# 🔧 Render 部署故障排查

## 📊 检查结果

### 测试的 URL：
- ❌ `https://hjjs-backend.onrender.com/health` - 连接失败
- ⚠️ `https://hjjs-backend-simple.onrender.com/health` - 返回 404

### 分析：
`hjjs-backend-simple` 服务有响应但返回 404，说明服务在运行但配置有问题。

---

## 🎯 请在 Render Dashboard 中检查以下内容

### 1. 服务配置检查清单

#### ✅ Root Directory（最重要！）
```
应该是: huangji-jingshi-web
```
**如果是 `backend` 或其他，这就是问题所在！**

#### ✅ Build Command
```
应该是: cd backend && cargo build --release --bin backend
```

#### ✅ Start Command（关键！）
```
正确的: cd backend && ../target/release/backend
错误的: ./target/release/backend (会找不到文件)
```

#### ✅ Health Check Path
```
应该是: /health
不是: /api/health
```

---

## 🔧 如何修复

### 方法 1: 修改现有服务配置

1. 在 Render Dashboard 点击 `hjjs-backend-simple` 服务
2. 点击 **Settings** 标签
3. 检查并修改以下配置：

```
Root Directory:    huangji-jingshi-web
Build Command:     cd backend && cargo build --release --bin backend
Start Command:     cd backend && ../target/release/backend
```

4. 保存后点击 **Manual Deploy** → **Deploy latest commit**

---

### 方法 2: 删除并重新创建服务

如果配置太混乱，建议重新创建：

1. **删除现有服务**
   - Settings → 滚动到底部 → Delete Web Service

2. **重新创建**
   - New + → Web Service
   - 连接 GitHub 仓库
   - 填写正确配置（见下方）

---

## 📋 正确的完整配置

```yaml
Name:              hjjs-backend
Environment:       Rust
Region:            Singapore
Branch:            main

Root Directory:    huangji-jingshi-web

Build Command:     
cd backend && cargo build --release --bin backend

Start Command:     
cd backend && ../target/release/backend

Health Check Path: /health

Environment Variables:
  RUST_LOG = info

Instance Type:     Free
Auto-Deploy:       Yes
```

---

## 🔍 查看日志排查问题

### 在 Render Dashboard:
1. 点击服务名称
2. 点击 **Logs** 标签
3. 查看最新的日志

### 期望看到的正常日志：
```
🚀 皇极经世后端服务启动中...
📁 数据路径: ...
🌐 启动服务器，端口: 10000
```

### 常见错误日志：

#### 错误 1: "No such file or directory"
```
./target/release/backend: No such file or directory
```
**原因**: Start Command 路径错误
**解决**: 改为 `cd backend && ../target/release/backend`

#### 错误 2: "failed to read"
```
Error: failed to read data files
```
**原因**: Root Directory 设置错误
**解决**: 改为 `huangji-jingshi-web`

---

## ✅ 验证修复成功

修复后，运行以下命令测试：

```bash
# 替换成你的实际 URL
curl https://your-backend.onrender.com/health
```

**预期响应：**
```json
{
  "status": "ok",
  "message": "皇极经世后端服务正常运行",
  "version": "1.0.0-fixed",
  "data_loaded": false
}
```

---

## 🆘 还是不行？

请提供以下信息：
1. Render 服务的完整配置截图
2. Render 日志的最后 20 行
3. 服务的当前状态（Live / Failed / Building）

我会帮你进一步分析！

---

## 📞 快速命令

```bash
# 测试后端
curl https://hjjs-backend-simple.onrender.com/health

# 查看配置
cat huangji-jingshi-web/RENDER_CONFIG.txt

# 打开 Render Dashboard
open https://dashboard.render.com
```

