# 前端集成指南

## 概述

本指南帮助您将现有Vercel前端与新的Supabase Edge Functions后端集成。

## 配置步骤

### 1. 更新环境变量

在Vercel项目中设置以下环境变量：

```env
NEXT_PUBLIC_API_BASE_URL=https://nunotqruohnfoozorqiz.supabase.co/functions/v1
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s
```

### 2. 创建API客户端

```typescript
// lib/api.ts

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || 
  'https://nunotqruohnfoozorqiz.supabase.co/functions/v1';
const ANON_KEY = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || 
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s';

// 通用请求函数
async function apiRequest<T>(
  endpoint: string, 
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;
  
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${ANON_KEY}`,
      ...options.headers,
    },
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error?.message || 'API请求失败');
  }

  return response.json();
}

// API方法
export const api = {
  // 健康检查
  async health() {
    return apiRequest<{
      status: string;
      timestamp: string;
      service: string;
      version: string;
    }>('/health');
  },

  // 天机演算计算
  async calculate(data: {
    birthDate: string;
    birthTime?: string;
    location?: string;
  }) {
    return apiRequest<{
      success: boolean;
      data: {
        sunSign: { name: string; degree: number; element: string; quality: string };
        moonSign: { name: string; degree: number; element: string; quality: string };
        ascendant: { name: string; degree: number; element: string; quality: string };
        elements: Record<string, number>;
        houses: Array<{ house: number; sign: string; planets: string[] }>;
        birthInfo: { date: string; time: string; location: string };
      };
    }>('/calculate', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  // 时间线计算
  async timeline(data: {
    birthDate: string;
    duration?: number;
  }) {
    return apiRequest<{
      success: boolean;
      data: {
        birthDate: string;
        duration: number;
        timeline: Array<{
          month: string;
          year: number;
          monthIndex: number;
          fortune: {
            overall: string;
            career: number;
            love: number;
            wealth: number;
            health: number;
            luckyDays: number[];
            luckyColor: string;
            luckyNumber: number;
          };
          events: Array<{
            date: string;
            type: string;
            impact: string;
            area: string;
            intensity: number;
            description: string;
          }>;
          advice: string;
        }>;
      };
    }>('/timeline', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  // 天象数据
  async sky(params?: {
    date?: string;
    latitude?: number;
    longitude?: number;
  }) {
    const searchParams = new URLSearchParams();
    if (params?.date) searchParams.set('date', params.date);
    if (params?.latitude) searchParams.set('latitude', String(params.latitude));
    if (params?.longitude) searchParams.set('longitude', String(params.longitude));
    
    const query = searchParams.toString();
    return apiRequest<{
      success: boolean;
      data: {
        date: string;
        location: { latitude: number; longitude: number };
        planets: Array<{
          name: string;
          symbol: string;
          longitude: number;
          sign: string;
          degree: number;
          retrograde: boolean;
        }>;
        aspects: Array<{
          planet1: string;
          planet2: string;
          aspect: string;
          exactAngle: number;
          actualAngle: number;
          orb: number;
          nature: string;
        }>;
        moonPhase: {
          phase: string;
          illumination: number;
          age: number;
          emoji: string;
        };
        sunrise: string;
        sunset: string;
      };
    }>(`/sky${query ? `?${query}` : ''}`);
  },

  // 历史记录
  async history(params?: {
    limit?: number;
    page?: number;
  }) {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.set('limit', String(params.limit));
    if (params?.page) searchParams.set('page', String(params.page));
    
    const query = searchParams.toString();
    return apiRequest<{
      success: boolean;
      data: {
        events: Array<{
          id: number;
          date: string;
          title: string;
          type: string;
          category: string;
          description: string;
          significance: string;
          visibility: string;
          magnitude?: number;
          duration?: string;
        }>;
        pagination: {
          total: number;
          page: number;
          limit: number;
          totalPages: number;
        };
      };
    }>(`/history${query ? `?${query}` : ''}`);
  },
};
```

### 3. 使用示例

#### React组件示例

```tsx
// components/CalculationForm.tsx
import { useState } from 'react';
import { api } from '@/lib/api';

export function CalculationForm() {
  const [birthDate, setBirthDate] = useState('');
  const [birthTime, setBirthTime] = useState('');
  const [location, setLocation] = useState('');
  const [result, setResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      const response = await api.calculate({
        birthDate,
        birthTime,
        location,
      });
      setResult(response.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : '计算失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="date"
        value={birthDate}
        onChange={(e) => setBirthDate(e.target.value)}
        required
      />
      <input
        type="time"
        value={birthTime}
        onChange={(e) => setBirthTime(e.target.value)}
      />
      <input
        type="text"
        value={location}
        onChange={(e) => setLocation(e.target.value)}
        placeholder="出生地点"
      />
      <button type="submit" disabled={loading}>
        {loading ? '计算中...' : '开始演算'}
      </button>
      
      {error && <p className="error">{error}</p>}
      
      {result && (
        <div className="result">
          <h3>太阳星座: {result.sunSign.name}</h3>
          <h3>月亮星座: {result.moonSign.name}</h3>
          <h3>上升星座: {result.ascendant.name}</h3>
        </div>
      )}
    </form>
  );
}
```

#### 使用React Query

```tsx
// hooks/useCalculation.ts
import { useQuery, useMutation } from '@tanstack/react-query';
import { api } from '@/lib/api';

export function useHealth() {
  return useQuery({
    queryKey: ['health'],
    queryFn: () => api.health(),
    refetchInterval: 30000, // 每30秒检查一次
  });
}

export function useCalculation() {
  return useMutation({
    mutationFn: api.calculate,
  });
}

export function useTimeline() {
  return useMutation({
    mutationFn: api.timeline,
  });
}

export function useSky(params?: Parameters<typeof api.sky>[0]) {
  return useQuery({
    queryKey: ['sky', params],
    queryFn: () => api.sky(params),
  });
}

export function useHistory(params?: Parameters<typeof api.history>[0]) {
  return useQuery({
    queryKey: ['history', params],
    queryFn: () => api.history(params),
  });
}
```

### 4. 从Render迁移

如果您的前端之前使用 `https://hjjs-backend.onrender.com`，只需：

1. **全局替换API地址**：
   ```
   旧: https://hjjs-backend.onrender.com
   新: https://nunotqruohnfoozorqiz.supabase.co/functions/v1
   ```

2. **添加认证头**：
   确保所有请求都包含 `Authorization: Bearer <ANON_KEY>` 头。

3. **更新环境变量**：
   在Vercel仪表板中更新API地址环境变量。

### 5. 测试连接

在浏览器控制台测试连接：

```javascript
// 测试健康检查
fetch('https://nunotqruohnfoozorqiz.supabase.co/functions/v1/health', {
  headers: {
    'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s'
  }
})
.then(r => r.json())
.then(console.log);
```

---

## 常见问题

### Q: CORS错误怎么办？
A: 所有Edge Functions已配置 `Access-Control-Allow-Origin: *`，应该不会有CORS问题。如果仍有问题，检查请求头是否正确。

### Q: 请求超时？
A: Edge Functions的冷启动时间通常在500ms以内。如果持续超时，检查网络连接。

### Q: 认证失败？
A: 确保使用正确的ANON_KEY，并以 `Bearer ` 前缀添加到Authorization头。
