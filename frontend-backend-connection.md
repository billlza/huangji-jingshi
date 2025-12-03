# 前后端连接配置指南

## 🎉 部署状态

✅ **前端部署成功**
- Vercel URL: `https://huangji-jingshi.vercel.app`
- 状态: Ready Latest

✅ **后端部署成功**  
- Render URL: `https://hjjs-backend.onrender.com`
- 状态: 运行中

## 🔧 需要完成的配置

### 1. 更新前端环境变量

您的后端已经成功部署到: `https://hjjs-backend.onrender.com`

现在需要在前端Vercel项目中添加环境变量 `VITE_BACKEND_URL`：

#### 在Vercel中添加环境变量：

1. **进入项目设置**
   - 访问: https://vercel.com/dashboard
   - 点击您的项目 "huangji-jingshi"

2. **配置环境变量**
   - 进入 Settings → Environment Variables
   - 添加新变量：
     - **Name**: `VITE_BACKEND_URL`
     - **Value**: `https://hjjs-backend.onrender.com`
     - **Environment**: Production, Preview, Development (全部选中)

3. **重新部署**
   - 在项目页面点击 "Deployments"
   - 点击最新部署的 "Deploy" 按钮重新部署
   - 或者推送新的Git提交触发自动部署

### 2. 验证前后端连接

部署完成后，访问您的网站并测试：

```bash
# 访问前端
https://huangji-jingshi.vercel.app

# 直接测试后端API
https://hjjs-backend.onrender.com/health
# 或其他API端点
```

### 3. 完整环境变量配置清单

确保Vercel中有以下环境变量：

```bash
# Supabase配置（必需）
VITE_SUPABASE_URL=https://nunotqruohnfoozorqiz.supabase.co
VITE_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# 后端API URL（新添加）
VITE_BACKEND_URL=https://hjjs-backend.onrender.com
```

## 🚀 部署验证测试

### 功能测试清单

1. **基础功能**
   - [ ] 前端网站正常加载
   - [ ] CSS样式正确显示
   - [ ] 页面路由正常工作

2. **皇极经世功能**
   - [ ] 日期选择器正常工作
   - [ ] 推算功能正常返回结果
   - [ ] 天文数据显示正确

3. **用户认证**
   - [ ] 用户注册功能正常
   - [ ] 用户登录功能正常
   - [ ] 头像上传功能正常

4. **API连接测试**
   - [ ] 前端能成功调用后端API
   - [ ] 没有CORS错误
   - [ ] 数据正常交互

### 常见问题排查

#### 问题1: 前端无法连接后端
**症状**: 前端页面加载但API调用失败
**解决方案**: 
- 检查VITE_BACKEND_URL环境变量
- 确认后端服务正常运行
- 检查CORS配置

#### 问题2: CORS错误
**症状**: 浏览器控制台显示CORS错误
**解决方案**: 
- 确认后端设置了正确的CORS头
- 检查后端允许的源域名

#### 问题3: Supabase连接问题
**症状**: 用户认证功能异常
**解决方案**: 
- 确认Supabase环境变量正确
- 检查Supabase项目状态

## 📱 前端代码中的API调用位置

前端代码中可能需要更新API调用的地方：

```typescript
// src/lib/api.ts 或类似的API配置文件
const BACKEND_URL = import.meta.env.VITE_BACKEND_URL || 'http://localhost:8080';

export const api = {
  // 皇极经世推算
  calculate: async (date: string) => {
    const response = await fetch(`${BACKEND_URL}/api/calculate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ date }),
    });
    return response.json();
  },
  
  // 天文数据获取
  getAstronomicalData: async (year: number) => {
    const response = await fetch(`${BACKEND_URL}/api/astronomical/${year}`);
    return response.json();
  },
};
```

## 🔧 快速修复步骤

### 步骤1: 在Vercel添加环境变量
1. 登录Vercel控制台
2. 进入项目设置 → Environment Variables
3. 添加 `VITE_BACKEND_URL=https://hjjs-backend.onrender.com`

### 步骤2: 重新部署
1. 在Vercel项目页面点击 "Redeploy"
2. 或推送代码到GitHub触发自动部署

### 步骤3: 测试验证
1. 访问 https://huangji-jingshi.vercel.app
2. 测试各个功能模块
3. 检查浏览器控制台是否有错误

## 🎯 下一步优化

### 可选配置
1. **自定义域名**
   - 为前端配置自定义域名
   - 为后端配置自定义域名

2. **HTTPS优化**
   - 确保证书有效
   - 强制HTTPS重定向

3. **性能优化**
   - 启用Vercel Analytics
   - 配置CDN缓存

---

**皇极经世网站部署即将完成！只需要最后这一步配置即可正常运行。** 🎉