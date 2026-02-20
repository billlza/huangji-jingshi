#!/usr/bin/env node

/**
 * Supabase Storage Bucket 配置脚本
 * 自动创建 avatars bucket 并设置权限
 *
 * 使用方法：
 * 1. 设置环境变量：
 *    export SUPABASE_URL="your-project-url"
 *    export SUPABASE_SERVICE_ROLE_KEY="your-service-role-key"
 *
 * 2. 运行脚本：
 *    node setup-storage.js
 */

// 尝试从 .env 文件读取（如果存在）
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const envVars = {};
try {
  const envFile = readFileSync(join(__dirname, '.env'), 'utf-8');
  envFile.split('\n').forEach((line) => {
    const trimmed = line.trim();
    if (trimmed && !trimmed.startsWith('#')) {
      const match = trimmed.match(/^([^=]+)=(.*)$/);
      if (match) {
        const key = match[1].trim();
        const value = match[2].trim().replace(/^["']|["']$/g, '');
        envVars[key] = value;
      }
    }
  });
} catch {
  // .env 文件不存在，使用环境变量
}

const SUPABASE_URL =
  process.env.SUPABASE_URL ||
  envVars.SUPABASE_URL ||
  process.env.VITE_SUPABASE_URL ||
  envVars.VITE_SUPABASE_URL;
const SUPABASE_SERVICE_ROLE_KEY =
  process.env.SUPABASE_SERVICE_ROLE_KEY || envVars.SUPABASE_SERVICE_ROLE_KEY;

if (!SUPABASE_URL || !SUPABASE_SERVICE_ROLE_KEY) {
  console.error('❌ 错误：缺少必要的环境变量');
  console.log('\n请设置以下环境变量：');
  console.log('  SUPABASE_URL=your-project-url');
  console.log('  SUPABASE_SERVICE_ROLE_KEY=your-service-role-key');
  console.log('\n💡 提示：');
  console.log('  1. 在 Supabase Dashboard 中进入 Project Settings > API');
  console.log('  2. 复制 Project URL 作为 SUPABASE_URL');
  console.log('  3. 复制 service_role key（不是 anon key）作为 SUPABASE_SERVICE_ROLE_KEY');
  process.exit(1);
}

async function createBucket() {
  console.log('\n🚀 开始配置 Supabase Storage...\n');

  try {
    // 检查 bucket 是否已存在
    console.log('📦 检查 avatars bucket 是否存在...');
    const listResponse = await fetch(`${SUPABASE_URL}/storage/v1/bucket`, {
      method: 'GET',
      headers: {
        apikey: SUPABASE_SERVICE_ROLE_KEY,
        Authorization: `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
      },
    });

    if (!listResponse.ok) {
      throw new Error(`无法列出 buckets: ${listResponse.statusText}`);
    }

    const buckets = await listResponse.json();
    const existingBucket = buckets.find((b) => b.name === 'avatars');

    if (existingBucket) {
      console.log('✅ avatars bucket 已存在');
      console.log('   名称:', existingBucket.name);
      console.log('   公开:', existingBucket.public ? '是' : '否');
      console.log('   创建时间:', new Date(existingBucket.created_at).toLocaleString('zh-CN'));

      if (!existingBucket.public) {
        console.log('\n⚠️  bucket 不是公开的，正在更新...');
        const updateResponse = await fetch(`${SUPABASE_URL}/storage/v1/bucket/avatars`, {
          method: 'PUT',
          headers: {
            apikey: SUPABASE_SERVICE_ROLE_KEY,
            Authorization: `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            public: true,
            file_size_limit: 5242880, // 5MB
            allowed_mime_types: ['image/jpeg', 'image/png', 'image/gif', 'image/webp'],
          }),
        });

        if (updateResponse.ok) {
          console.log('✅ bucket 已更新为公开可读');
        } else {
          console.warn('⚠️  更新 bucket 失败，请手动在 Dashboard 中设置为 Public');
        }
      }
    } else {
      console.log('📦 创建 avatars bucket...');

      // 创建 bucket
      const createResponse = await fetch(`${SUPABASE_URL}/storage/v1/bucket`, {
        method: 'POST',
        headers: {
          apikey: SUPABASE_SERVICE_ROLE_KEY,
          Authorization: `Bearer ${SUPABASE_SERVICE_ROLE_KEY}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: 'avatars',
          public: true,
          file_size_limit: 5242880, // 5MB
          allowed_mime_types: ['image/jpeg', 'image/png', 'image/gif', 'image/webp'],
        }),
      });

      if (!createResponse.ok) {
        const error = await createResponse.text();
        throw new Error(`创建 bucket 失败: ${error}`);
      }

      console.log('✅ avatars bucket 创建成功');
    }

    console.log('\n✅ Storage bucket 配置完成！');
    console.log('\n📝 接下来的步骤：');
    console.log('   1. 在 Supabase Dashboard 中进入 Storage > avatars');
    console.log('   2. 检查 Policies 标签页，确保有以下策略：');
    console.log('\n   或者使用以下 SQL 在 Supabase SQL Editor 中执行：\n');
    console.log(`-- 允许所有人读取
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
    `);
    console.log('\n✨ 配置完成后，头像功能即可正常使用！\n');
  } catch (error) {
    console.error('\n❌ 配置失败:', error.message);
    console.log('\n💡 如果遇到权限错误，请确保：');
    console.log('   1. 使用的是 service_role key（不是 anon key）');
    console.log('   2. service_role key 有管理 Storage 的权限');
    process.exit(1);
  }
}

createBucket();
