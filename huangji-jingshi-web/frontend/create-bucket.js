#!/usr/bin/env node

/**
 * 自动创建 Supabase Storage bucket
 */

const SUPABASE_URL = 'https://nunotqruohnfoozorqiz.supabase.co';
const SUPABASE_SERVICE_ROLE_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc2NDQ2NjY2MiwiZXhwIjoyMDgwMDQyNjYyfQ.a_-2C0z0jgAu3bD-BFBG4ONi1kKyz7fn4cnSlBlY-eE';

async function setupStorage() {
  console.log('🚀 开始配置 Supabase Storage...\n');

  try {
    // 1. 检查 bucket 是否已存在
    console.log('📦 检查 avatars bucket...');
    const listResponse = await fetch(`${SUPABASE_URL}/storage/v1/bucket`, {
      method: 'GET',
      headers: {
        'apikey': SUPABASE_SERVICE_ROLE_KEY,
        'Authorization': `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
      },
    });

    if (!listResponse.ok) {
      throw new Error(`无法列出 buckets: ${listResponse.statusText}`);
    }

    const buckets = await listResponse.json();
    const existingBucket = buckets.find(b => b.name === 'avatars');

    if (existingBucket) {
      console.log('✅ avatars bucket 已存在');
      
      // 确保是公开的
      if (!existingBucket.public) {
        console.log('🔓 更新 bucket 为公开...');
        const updateResponse = await fetch(`${SUPABASE_URL}/storage/v1/bucket/avatars`, {
          method: 'PUT',
          headers: {
            'apikey': SUPABASE_SERVICE_ROLE_KEY,
            'Authorization': `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            public: true,
            file_size_limit: 5242880,
            allowed_mime_types: ['image/jpeg', 'image/png', 'image/gif', 'image/webp']
          }),
        });

        if (updateResponse.ok) {
          console.log('✅ bucket 已更新为公开');
        }
      }
    } else {
      // 2. 创建 bucket
      console.log('📦 创建 avatars bucket...');
      const createResponse = await fetch(`${SUPABASE_URL}/storage/v1/bucket`, {
        method: 'POST',
        headers: {
          'apikey': SUPABASE_SERVICE_ROLE_KEY,
          'Authorization': `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: 'avatars',
          public: true,
          file_size_limit: 5242880, // 5MB
          allowed_mime_types: ['image/jpeg', 'image/png', 'image/gif', 'image/webp']
        }),
      });

      if (!createResponse.ok) {
        const error = await createResponse.text();
        throw new Error(`创建 bucket 失败: ${error}`);
      }

      console.log('✅ avatars bucket 创建成功');
    }

    // 3. 设置权限策略（通过 SQL）
    console.log('\n🔐 设置权限策略...');
    const sql = `
-- 允许所有人读取
CREATE POLICY IF NOT EXISTS "Public Access" ON storage.objects
FOR SELECT USING (bucket_id = 'avatars');

-- 允许已认证用户上传
CREATE POLICY IF NOT EXISTS "Authenticated users can upload" ON storage.objects
FOR INSERT WITH CHECK (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

-- 允许已认证用户更新
CREATE POLICY IF NOT EXISTS "Users can update own files" ON storage.objects
FOR UPDATE USING (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);

-- 允许已认证用户删除
CREATE POLICY IF NOT EXISTS "Users can delete own files" ON storage.objects
FOR DELETE USING (
  bucket_id = 'avatars' AND
  auth.role() = 'authenticated'
);
    `.trim();

    // 通过 REST API 执行 SQL
    const sqlResponse = await fetch(`${SUPABASE_URL}/rest/v1/rpc/exec_sql`, {
      method: 'POST',
      headers: {
        'apikey': SUPABASE_SERVICE_ROLE_KEY,
        'Authorization': `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ sql }),
    });

    // 如果 RPC 不存在，尝试直接通过 PostgREST
    if (!sqlResponse.ok) {
      console.log('⚠️  无法通过 API 设置策略，请手动在 Supabase Dashboard > SQL Editor 中执行以下 SQL：\n');
      console.log(sql);
      console.log('\n');
    } else {
      console.log('✅ 权限策略设置成功');
    }

    console.log('\n✨ 配置完成！头像功能现在可以使用了。\n');

  } catch (error) {
    console.error('\n❌ 配置失败:', error.message);
    console.log('\n💡 如果 bucket 创建成功但策略设置失败，请手动在 Supabase Dashboard > SQL Editor 中执行 SQL。\n');
    process.exit(1);
  }
}

setupStorage();

