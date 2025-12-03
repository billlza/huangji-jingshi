# 🧪 API测试脚本

## 测试新的MiniMax后端API

### 健康检查测试
```bash
# 测试健康检查端点
curl -X POST "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/health" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s" \
  -H "Content-Type: application/json"
```

### 天机演算测试
```bash
# 测试计算端点
curl -X POST "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/calculate" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s" \
  -H "Content-Type: application/json" \
  -d '{"datetime":"2025-12-03T12:00:00Z","lat":39.9042,"lon":116.4074}'
```

### 前端功能测试

访问 https://huangji-jingshi-web.vercel.app 测试：
1. **天机演算工具** - 输入生辰八字，查看计算结果
2. **时间线** - 查看运势时间线变化
3. **星象图** - 查看天体运行图
4. **历史记录** - 查看相关历史事件

## 预期结果

- ✅ 健康检查返回状态信息
- ✅ 天机演算返回八字计算结果
- ✅ 前端所有功能正常工作
- ✅ 无CORS或401认证错误

如果遇到问题，请提供具体的错误信息！
