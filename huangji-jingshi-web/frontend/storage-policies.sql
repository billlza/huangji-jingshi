-- Supabase Storage 权限策略配置
-- 请在 Supabase Dashboard > SQL Editor 中执行此 SQL

-- 允许所有人读取 avatars bucket 中的文件
CREATE POLICY IF NOT EXISTS "Public Access" ON storage.objects
FOR SELECT USING (bucket_id = 'avatars');

-- 允许已认证用户上传文件到 avatars bucket
CREATE POLICY IF NOT EXISTS "Authenticated users can upload" ON storage.objects
FOR INSERT WITH CHECK (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

-- 允许已认证用户更新 avatars bucket 中的文件
CREATE POLICY IF NOT EXISTS "Users can update own files" ON storage.objects
FOR UPDATE USING (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

-- 允许已认证用户删除 avatars bucket 中的文件
CREATE POLICY IF NOT EXISTS "Users can delete own files" ON storage.objects
FOR DELETE USING (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

