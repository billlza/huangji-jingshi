# Vercel部署问题解决方案

## 🔍 常见Vercel部署错误及解决方案

### 错误1: Node.js版本不兼容

**错误表现：**
```
Error: Unsupported engine
Node.js version 18.x is not supported
```

**解决方案：**

#### 方案A: 更新package.json引擎要求
```json
{
  "engines": {
    "node": ">=20.0.0"
  }
}
```

#### 方案B: 使用Vercel Node.js版本配置
在项目根目录创建`.nvmrc`文件：
```
20.19.0
```

#### 方案C: 修改依赖版本（推荐）
更新package.json中的关键依赖：

```json
{
  "dependencies": {
    "@supabase/supabase-js": "^2.79.0", // 降级到兼容版本
    "react-router-dom": "^6.26.2"      // 降级到兼容版本
  },
  "engines": {
    "node": ">=18.18.0"
  }
}
```

### 错误2: TypeScript编译错误

**解决方案：**
1. 检查tsconfig.json配置
2. 确保所有类型定义正确
3. 移除或更新不兼容的类型依赖

### 错误3: 构建命令失败

**当前构建命令：**
```json
{
  "buildCommand": "cd frontend && npm ci && npm run build"
}
```

**优化建议：**
```json
{
  "buildCommand": "cd frontend && npm install --legacy-peer-deps && npm run build"
}
```

### 错误4: 静态资源404错误

**解决方案：**
确保Vercel配置正确：
```json
{
  "rewrites": [
    {
      "source": "/(.*)",
      "destination": "/index.html"
    }
  ],
  "headers": [
    {
      "source": "/assets/(.*)",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    }
  ]
}
```

## 🛠️ Vercel部署步骤（优化版）

### 1. GitHub仓库准备
确保您的GitHub仓库结构：
```
repository-name/
├── frontend/
│   ├── src/
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
├── backend/
└── vercel.json (在根目录)
```

### 2. Vercel项目设置
1. **导入GitHub仓库**
2. **配置项目设置：**
   - Framework Preset: `Vite`
   - Root Directory: `/`（根目录）
   - Build Command: `npm run build`
   - Output Directory: `frontend/dist`
   - Install Command: `npm install`

### 3. 环境变量配置
在Vercel项目设置中添加：

```
VITE_SUPABASE_URL=https://nunotqruohnfoozorqiz.supabase.co
VITE_SUPABASE_ANON_KEY=您的Supabase匿名密钥
```

### 4. Node.js版本设置
在Vercel项目设置中：
- Node.js Version: `20.x` 或 `18.x`

## 🚨 立即可用的解决方案

### 快速修复脚本
创建 `fix-vercel.js`：
```javascript
const fs = require('fs');
const path = require('path');

const packageJsonPath = path.join(__dirname, 'frontend', 'package.json');

// 读取package.json
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

// 添加engines字段
packageJson.engines = {
  node: ">=18.18.0"
};

// 降级不兼容的依赖
packageJson.dependencies["@supabase/supabase-js"] = "^2.79.0";
packageJson.dependencies["react-router-dom"] = "^6.26.2";

// 保存修改
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));
console.log('✅ package.json已修复');
```

### 执行修复
```bash
node fix-vercel.js
git add frontend/package.json
git commit -m "修复Vercel部署兼容性问题"
git push
```

## 📋 部署检查清单

### 部署前检查
- [ ] GitHub仓库已推送最新代码
- [ ] package.json已更新Node.js要求
- [ ] 所有依赖版本兼容Node 18+
- [ ] 环境变量已配置
- [ ] Vercel项目设置正确

### 部署后验证
- [ ] 网站可以正常访问
- [ ] 所有静态资源加载正常
- [ ] 用户认证功能正常
- [ ] 皇极经世推算功能正常
- [ ] 移动端适配正常

## 🔧 环境变量配置详解

### 前端环境变量
```bash
# Supabase配置（必需）
VITE_SUPABASE_URL=https://nunotqruohnfoozorqiz.supabase.co
VITE_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# 后端URL（部署后设置）
VITE_BACKEND_URL=https://your-backend.onrender.com
```

### 获取Supabase环境变量
1. 登录Supabase Dashboard
2. 选择您的项目
3. 进入Settings > API
4. 复制Project URL和anon/public key

## 🆘 如果问题依然存在

请提供以下信息：
1. Vercel部署日志截图
2. 具体的错误信息
3. 项目仓库地址（如果可以公开）
4. Node.js版本设置

我会根据具体错误信息提供针对性的解决方案。

---

**皇极经世网站Vercel部署修复指南** 📚