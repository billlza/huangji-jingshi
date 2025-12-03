// 测试黄极经世后端API
const BASE_URL = 'https://nunotqruohnfoozorqiz.supabase.co/functions/v1';
const API_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s';

async function testAPI() {
  console.log('🔍 测试黄极经世后端API连接...\n');
  
  try {
    // 测试健康检查
    console.log('1. 测试健康检查端点...');
    const healthResponse = await fetch(`${BASE_URL}/health`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${API_KEY}`,
        'Content-Type': 'application/json'
      }
    });
    
    console.log(`   状态码: ${healthResponse.status}`);
    console.log(`   状态文本: ${healthResponse.statusText}`);
    
    if (healthResponse.ok) {
      const healthData = await healthResponse.json();
      console.log(`   响应数据:`, JSON.stringify(healthData, null, 2));
      console.log('   ✅ 健康检查成功\n');
    } else {
      const errorText = await healthResponse.text();
      console.log(`   ❌ 健康检查失败: ${errorText}\n`);
    }
    
    // 测试计算端点
    console.log('2. 测试计算端点...');
    const calculateResponse = await fetch(`${BASE_URL}/calculate?datetime=2024-01-01T12:00:00Z&lat=39.9042&lon=116.4074`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${API_KEY}`,
        'Content-Type': 'application/json'
      }
    });
    
    console.log(`   状态码: ${calculateResponse.status}`);
    console.log(`   状态文本: ${calculateResponse.statusText}`);
    
    if (calculateResponse.ok) {
      const calcData = await calculateResponse.json();
      console.log(`   响应数据:`, JSON.stringify(calcData, null, 2));
      console.log('   ✅ 计算端点成功\n');
    } else {
      const errorText = await calculateResponse.text();
      console.log(`   ❌ 计算端点失败: ${errorText}\n`);
    }
    
    // 测试时间线端点
    console.log('3. 测试时间线端点...');
    const timelineResponse = await fetch(`${BASE_URL}/timeline?datetime=2024-01-01T12:00:00Z`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${API_KEY}`,
        'Content-Type': 'application/json'
      }
    });
    
    console.log(`   状态码: ${timelineResponse.status}`);
    console.log(`   状态文本: ${timelineResponse.statusText}`);
    
    if (timelineResponse.ok) {
      const timelineData = await timelineResponse.json();
      console.log(`   响应数据:`, JSON.stringify(timelineData, null, 2));
      console.log('   ✅ 时间线端点成功\n');
    } else {
      const errorText = await timelineResponse.text();
      console.log(`   ❌ 时间线端点失败: ${errorText}\n`);
    }
    
  } catch (error) {
    console.error('❌ 测试过程中发生错误:', error);
  }
}

// 运行测试
testAPI();