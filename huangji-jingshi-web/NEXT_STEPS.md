# 后端部署完成后的配置步骤

## ✅ 后端已部署
后端地址：`https://hjjs-backend.onrender.com`

## 📝 下一步：配置前端连接后端

### 步骤 1: 在 Vercel 中添加环境变量

1. 访问 https://vercel.com/dashboard
2. 进入您的项目：`huangji-jingshi`
3. 点击 **Settings** > **Environment Variables**
4. 添加新的环境变量：
   - **Key**: `VITE_BACKEND_URL`
   - **Value**: `https://hjjs-backend.onrender.com`
   - **Environment**: 选择所有环境（Production, Preview, Development）
5. 点击 **Save**

### 步骤 2: 重新部署前端

在 Vercel Dashboard 中：
1. 进入 **Deployments** 页面
2. 找到最新的部署
3. 点击右侧的 **"..."** 菜单
4. 选择 **"Redeploy"**
5. 确认重新部署

或者，Vercel 会在您推送代码时自动重新部署。

### 步骤 3: 验证部署

部署完成后，访问：
- https://huangji-jingshi.vercel.app/tools

应该能看到：
- ✅ Timeline 数据正常显示
- ✅ 天象和运势计算正常
- ✅ 历史事件数据正常
- ✅ 不再显示 "Timeline data unavailable" 错误

---

## 🔍 如果还有问题

1. **检查后端是否运行**：
   - 访问：https://hjjs-backend.onrender.com/health
   - 应该返回：`OK`

2. **检查 CORS 配置**：
   - 后端已配置允许所有来源，应该没问题

3. **查看浏览器控制台**：
   - 按 F12 打开开发者工具
   - 查看 Console 和 Network 标签页
   - 检查是否有 API 请求错误

---

## 📋 完整的环境变量清单（Vercel）

确保以下环境变量都已设置：

| Key | Value |
|-----|-------|
| `VITE_SUPABASE_URL` | `https://nunotqruohnfoozorqiz.supabase.co` |
| `VITE_SUPABASE_ANON_KEY` | 您的 Supabase anon key |
| `VITE_BACKEND_URL` | `https://hjjs-backend.onrender.com` |

