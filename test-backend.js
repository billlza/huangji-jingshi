// 测试Node.js后端是否正常工作
const axios = require('axios');

async function testBackend() {
  const baseUrl = 'https://hjjs-backend.onrender.com';
  
  console.log('🧪 测试后端API...');
  
  try {
    // 测试健康检查端点
    console.log('📍 测试 /health 端点...');
    const healthResponse = await axios.get(`${baseUrl}/health`, {
      timeout: 5000
    });
    console.log('✅ 健康检查成功:', healthResponse.data);
    
    // 测试天象数据端点
    console.log('📍 测试 /sky 端点...');
    const skyResponse = await axios.get(`${baseUrl}/sky`, {
      timeout: 5000
    });
    console.log('✅ 天象数据成功:', skyResponse.data.celestial_data ? '数据正常' : '数据异常');
    
    console.log('🎉 后端API测试完成！');
    
  } catch (error) {
    console.log('❌ 测试失败:', error.message);
    if (error.code === 'ECONNABORTED') {
      console.log('🔄 后端可能还在部署中，请稍等片刻再测试');
    }
  }
}

// 如果运行在Node.js环境中，执行测试
if (typeof window === 'undefined') {
  testBackend();
}

module.exports = { testBackend };