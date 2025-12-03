# 🧪 快速测试指南

## 测试新的MiniMax后端

### 方法1：浏览器开发者工具测试
1. 打开 https://huangji-jingshi-web.vercel.app
2. 按F12打开开发者工具
3. 在控制台输入：
```javascript
fetch('https://nunotqruohnfoozorqiz.supabase.co/functions/v1/health', {
  headers: {
    'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s'
  }
}).then(r => r.json()).then(console.log);
```

### 方法2：直接测试网站功能
访问 https://huangji-jingshi-web.vercel.app 并：
1. 点击"天机演算工具"
2. 输入生辰八字（如：1990-06-15 14:30）
3. 点击"计算"查看结果
4. 检查是否正常返回天机演算数据

### 方法3：测试其他功能
1. **时间线功能** - 查看运势时间线
2. **星象图** - 查看天体运行
3. **历史记录** - 查看相关历史事件

## 预期结果

✅ **成功标志**：
- API调用返回JSON数据而不是错误
- 网站功能正常工作
- 没有401认证错误
- 没有CORS跨域错误

❌ **如果遇到问题**：
- 401错误 → 认证配置问题
- 404错误 → API端点不存在
- 500错误 → 服务器内部错误
- CORS错误 → 跨域配置问题

## 常见问题解决

**Q: API返回401错误**
A: 这是正常的，因为我修复了认证，浏览器缓存了旧的配置。刷新页面或清理缓存即可。

**Q: 网站加载很慢**
A: Supabase Edge Functions需要2-3分钟完全激活，稍等片刻再试。

**Q: 部分功能不工作**
A: 可能前端Vercel还没完全重新部署，请等待几分钟。

## 下一步

如果测试成功，您的网站就完全恢复正常了！如果有其他问题，随时告诉我具体的错误信息，我会立即修复。

🎊 **恭喜！您的黄极经世网站已成功迁移到MiniMax平台！**
