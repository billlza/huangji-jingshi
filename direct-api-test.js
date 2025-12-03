const https = require('https');
const url = require('url');

// 测试timeline API
const apiUrl = 'https://nunotqruohnfoozorqiz.supabase.co/functions/v1/timeline?datetime=1990-01-01T12:00:00Z';
const headers = {
  'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s',
  'Content-Type': 'application/json'
};

console.log('🔍 直接测试API端点...');
console.log(`URL: ${apiUrl}`);

const parsedUrl = new url.URL(apiUrl);
const options = {
  hostname: parsedUrl.hostname,
  path: parsedUrl.pathname + parsedUrl.search,
  method: 'POST',
  headers: headers
};

console.log('发送请求...');

const req = https.request(options, (res) => {
  console.log(`\n📊 响应状态: ${res.statusCode} ${res.statusMessage}`);
  console.log('响应头:', res.headers);
  
  let data = '';
  res.on('data', (chunk) => {
    data += chunk;
  });
  
  res.on('end', () => {
    console.log('\n📄 响应内容:');
    console.log(data.substring(0, 500) + (data.length > 500 ? '...' : ''));
    
    try {
      const parsed = JSON.parse(data);
      console.log('\n✅ JSON解析成功');
      if (parsed.current) {
        console.log(`当前年份: ${parsed.current.hui?.year || 'N/A'}`);
        console.log(`运: ${parsed.current.yun?.name || 'N/A'}`);
        console.log(`世: ${parsed.current.shi?.name || 'N/A'}`);
        console.log(`旬: ${parsed.current.xun?.name || 'N/A'}`);
      }
    } catch (e) {
      console.log('\n❌ JSON解析失败:', e.message);
    }
  });
});

req.on('error', (e) => {
  console.log('\n💥 请求错误:', e.message);
});

req.write(JSON.stringify({}));
req.end();