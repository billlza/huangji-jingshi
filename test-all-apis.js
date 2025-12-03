#!/usr/bin/env node

// 完整测试所有API端点
const VITE_BACKEND_URL = 'https://nunotqruohnfoozorqiz.supabase.co';
const SUPABASE_ANON_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s';

async function testAllAPIs() {
  console.log('🔍 全面测试API状态...\n');
  
  const tests = [
    {
      name: 'timeline API - 1990年1月1日',
      url: `${VITE_BACKEND_URL}/functions/v1/timeline?datetime=1990-01-01T12:00:00Z`,
      method: 'POST',
      body: {}
    },
    {
      name: 'calculate API - 1990年北京',
      url: `${VITE_BACKEND_URL}/functions/v1/calculate?datetime=1990-01-01T12:00:00Z&lat=39.9&lon=116.4`,
      method: 'POST',
      body: {}
    },
    {
      name: 'sky API - 天空星图',
      url: `${VITE_BACKEND_URL}/functions/v1/sky`,
      method: 'POST',
      body: {}
    },
    {
      name: 'history API - 历史记录',
      url: `${VITE_BACKEND_URL}/functions/v1/history?year=1990&mode=yun&limit=3`,
      method: 'POST',
      body: {}
    }
  ];

  let successCount = 0;
  let totalTests = tests.length;

  for (const [index, test] of tests.entries()) {
    console.log(`\n📡 测试 ${index + 1}/${totalTests}: ${test.name}`);
    console.log(`   URL: ${test.url}`);
    
    try {
      const response = await fetch(test.url, {
        method: test.method,
        headers: {
          'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(test.body)
      });
      
      console.log(`   状态码: ${response.status} ${response.statusText}`);
      
      if (response.ok) {
        const data = await response.json();
        console.log(`   ✅ 成功响应:`);
        console.log(`      数据类型: ${typeof data}`);
        console.log(`      数据大小: ${JSON.stringify(data).length} 字符`);
        if (data.current) {
          console.log(`      当前年份: ${data.current.hui?.year || 'N/A'}`);
        }
        successCount++;
      } else {
        const errorText = await response.text();
        console.log(`   ❌ 失败响应:`);
        console.log(`      状态: ${response.status}`);
        console.log(`      错误: ${errorText.substring(0, 100)}...`);
      }
    } catch (error) {
      console.log(`   💥 网络错误:`);
      console.log(`      错误: ${error.message}`);
    }
  }

  console.log('\n' + '='.repeat(50));
  console.log(`📊 测试总结: ${successCount}/${totalTests} 个API端点正常`);
  console.log(`💯 成功率: ${Math.round(successCount/totalTests*100)}%`);
  
  if (successCount === totalTests) {
    console.log('🎉 所有API端点正常工作！前端应该可以正常调用。');
  } else if (successCount > 0) {
    console.log('⚠️  部分API端点正常，网站功能可能受限。');
  } else {
    console.log('🚨 所有API端点失败，需要检查后端配置。');
  }
}

// 运行测试
testAllAPIs().catch(console.error);