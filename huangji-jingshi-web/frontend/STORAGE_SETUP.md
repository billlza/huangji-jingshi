# Supabase Storage 自动配置脚本

这个脚本会自动创建 `avatars` bucket 并配置基本设置。

## 快速开始

### 1. 获取 Supabase 凭据

1. 登录 [Supabase Dashboard](https://app.supabase.com)
2. 选择您的项目
3. 进入 **Project Settings** > **API**
4. 复制以下信息：
   - **Project URL** → 作为 `SUPABASE_URL`
   - **service_role key** → 作为 `SUPABASE_SERVICE_ROLE_KEY`（⚠️ 注意：不是 anon key）

### 2. 设置环境变量

在终端中运行：

```bash
export SUPABASE_URL="https://your-project.supabase.co"
export SUPABASE_SERVICE_ROLE_KEY="your-service-role-key-here"
```

或者创建 `.env` 文件（在 `frontend` 目录下）：

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key-here
```

### 3. 运行脚本

```bash
cd frontend
npm run setup-storage
```

或者直接运行：

```bash
node setup-storage.js
```

## 脚本功能

- ✅ 检查 `avatars` bucket 是否存在
- ✅ 如果不存在，自动创建 bucket
- ✅ 设置 bucket 为公开可读
- ✅ 配置文件大小限制（5MB）
- ✅ 配置允许的 MIME 类型（图片格式）

## 后续步骤

脚本运行后，还需要在 Supabase Dashboard 中设置 Storage Policies：

### 方法 1：使用 Dashboard（推荐）

1. 进入 **Storage** > **avatars** > **Policies**
2. 创建以下策略：

   **策略 1：公开读取**
   - Policy name: `Public Access`
   - Allowed operation: `SELECT`
   - Target roles: `anon`
   - Policy definition: `bucket_id = 'avatars'`

   **策略 2：已认证用户上传**
   - Policy name: `Authenticated users can upload`
   - Allowed operation: `INSERT`
   - Target roles: `authenticated`
   - Policy definition: `bucket_id = 'avatars'`

   **策略 3：已认证用户更新**
   - Policy name: `Users can update own files`
   - Allowed operation: `UPDATE`
   - Target roles: `authenticated`
   - Policy definition: `bucket_id = 'avatars'`

   **策略 4：已认证用户删除**
   - Policy name: `Users can delete own files`
   - Allowed operation: `DELETE`
   - Target roles: `authenticated`
   - Policy definition: `bucket_id = 'avatars'`

### 方法 2：使用 SQL Editor

在 Supabase Dashboard 的 **SQL Editor** 中执行以下 SQL：

```sql
-- 允许所有人读取
CREATE POLICY IF NOT EXISTS "Public Access" ON storage.objects
FOR SELECT USING (bucket_id = 'avatars');

-- 允许已认证用户上传
CREATE POLICY IF NOT EXISTS "Authenticated users can upload" ON storage.objects
FOR INSERT WITH CHECK (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

-- 允许已认证用户更新自己的文件
CREATE POLICY IF NOT EXISTS "Users can update own files" ON storage.objects
FOR UPDATE USING (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

-- 允许已认证用户删除自己的文件
CREATE POLICY IF NOT EXISTS "Users can delete own files" ON storage.objects
FOR DELETE USING (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);
```

## 验证配置

配置完成后，在应用中：
1. 登录账户
2. 进入设置页面
3. 尝试上传头像
4. 如果成功，说明配置完成！

## 故障排除

### 错误：Bucket not found
- 确保运行了配置脚本
- 检查环境变量是否正确设置

### 错误：Permission denied
- 确保使用的是 `service_role` key（不是 `anon` key）
- 检查 Storage Policies 是否正确配置

### 错误：File too large
- 检查 bucket 的文件大小限制设置
- 确保上传的图片小于 5MB

## 安全提示

⚠️ **重要**：
- `service_role` key 具有完全的管理权限，请妥善保管
- 不要将 `service_role` key 提交到代码仓库
- 只在本地开发环境或安全的服务器环境中使用此脚本

