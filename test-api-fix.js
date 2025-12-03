#!/usr/bin/env node

// 测试修复后的API端点
const VITE_BACKEND_URL = 'https://nunotqruohnfoozorqiz.supabase.co';
const SUPABASE_ANON_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s';

async function testAPI() {
  console.log('🧪 测试修复后的API端点...');
  
  const tests = [
    {
      name: 'timeline API',
      url: `${VITE_BACKEND_URL}/functions/v1/timeline?datetime=1990-01-01T12:00:00Z`,
      method: 'POST'
    },
    {
      name: 'calculate API', 
      url: `${VITE_BACKEND_URL}/functions/v1/calculate?datetime=1990-01-01T12:00:00Z&lat=39.9&lon=116.4`,
      method: 'POST'
    },
    {
      name: 'sky API',
      url: `${VITE_BACKEND_URL}/functions/v1/sky`,
      method: 'POST'
    }
  ];

  for (const test of tests) {
    try {
      console.log(`\n📡 测试 ${test.name}...`);
      console.log(`   URL: ${test.url}`);
      
      const response = await fetch(test.url, {
        method: test.method,
        headers: {
          'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
          'Content-Type': 'application/json'
        }
      });
      
      console.log(`   状态码: ${response.status}`);
      
      if (response.ok) {
        const data = await response.json();
        console.log(`   ✅ 成功: ${JSON.stringify(data, null, 2).substring(0, 100)}...`);
      } else {
        const errorText = await response.text();
        console.log(`   ❌ 失败: ${response.status} - ${errorText.substring(0, 100)}...`);
      }
    } catch (error) {
      console.log(`   ❌ 错误: ${error.message}`);
    }
  }
}

testAPI().then(() => {
  console.log('\n🎉 API测试完成！');
}).catch(error => {
  console.error('❌ 测试失败:', error.message);
});