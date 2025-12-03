# 天机演算后端API文档

## 概述

本文档描述了天机演算系统的后端API接口。所有API基于Supabase Edge Functions实现，提供稳定可靠的服务。

### 基础URL

```
https://nunotqruohnfoozorqiz.supabase.co/functions/v1
```

### 认证

所有请求需要在Header中携带API密钥：

```
Authorization: Bearer <SUPABASE_ANON_KEY>
```

ANON KEY: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im51bm90cXJ1b2huZm9vem9ycWl6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ0NjY2NjIsImV4cCI6MjA4MDA0MjY2Mn0.Ih9vBM9RxZ1fGVXcY6j33pKShh-LsHSLUGLewRNF-0s`

---

## API端点

### 1. 健康检查 - GET /health

检查服务运行状态。

**请求**
```bash
curl -X GET \
  "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/health" \
  -H "Authorization: Bearer <SUPABASE_ANON_KEY>"
```

**响应**
```json
{
  "status": "healthy",
  "timestamp": "2025-12-03T04:04:21.906Z",
  "service": "huangji-jingshi-backend",
  "version": "1.0.0"
}
```

---

### 2. 天机演算计算 - POST /calculate

根据出生信息计算星座、元素分布和宫位信息。

**请求**
```bash
curl -X POST \
  "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/calculate" \
  -H "Authorization: Bearer <SUPABASE_ANON_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "birthDate": "1990-01-01",
    "birthTime": "08:30",
    "location": "北京"
  }'
```

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| birthDate | string | 是 | 出生日期，格式：YYYY-MM-DD |
| birthTime | string | 否 | 出生时间，格式：HH:mm，默认12:00 |
| location | string | 否 | 出生地点 |

**响应**
```json
{
  "success": true,
  "data": {
    "sunSign": {
      "name": "摩羯座",
      "degree": 15,
      "element": "土",
      "quality": "本位"
    },
    "moonSign": {
      "name": "狮子座",
      "degree": 22,
      "element": "火",
      "quality": "固定"
    },
    "ascendant": {
      "name": "天蝎座",
      "degree": 8,
      "element": "水",
      "quality": "固定"
    },
    "elements": {
      "火": 4,
      "土": 3,
      "风": 1,
      "水": 2
    },
    "houses": [
      {
        "house": 1,
        "sign": "天蝎座",
        "planets": ["上升点", "冥王星"]
      }
    ],
    "birthInfo": {
      "date": "1990-01-01",
      "time": "08:30",
      "location": "北京"
    }
  }
}
```

---

### 3. 时间线计算 - POST /timeline

生成未来月度天象时间线预测。

**请求**
```bash
curl -X POST \
  "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/timeline" \
  -H "Authorization: Bearer <SUPABASE_ANON_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "birthDate": "1990-01-01",
    "duration": 12
  }'
```

**请求参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| birthDate | string | 是 | 出生日期，格式：YYYY-MM-DD |
| duration | number | 否 | 预测月数，默认12个月 |

**响应**
```json
{
  "success": true,
  "data": {
    "birthDate": "1990-01-01",
    "duration": 12,
    "timeline": [
      {
        "month": "一月",
        "year": 2026,
        "monthIndex": 1,
        "fortune": {
          "overall": "良好",
          "career": 8,
          "love": 7,
          "wealth": 6,
          "health": 9,
          "luckyDays": [3, 15, 22],
          "luckyColor": "金色",
          "luckyNumber": 8
        },
        "events": [
          {
            "date": "2026-01-15",
            "type": "新月",
            "impact": "新开始",
            "area": "事业",
            "intensity": 3,
            "description": "适合开启新项目"
          }
        ],
        "advice": "运势良好，稳步推进，适度冒险。"
      }
    ]
  }
}
```

---

### 4. 天象数据 - GET /sky

获取指定日期和地点的天象数据。

**请求**
```bash
curl -X GET \
  "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/sky?date=2025-12-03&latitude=39.9042&longitude=116.4074" \
  -H "Authorization: Bearer <SUPABASE_ANON_KEY>"
```

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| date | string | 否 | 日期，格式：YYYY-MM-DD，默认当天 |
| latitude | number | 否 | 纬度，默认39.9042（北京） |
| longitude | number | 否 | 经度，默认116.4074（北京） |

**响应**
```json
{
  "success": true,
  "data": {
    "date": "2025-12-03",
    "location": {
      "latitude": 39.9042,
      "longitude": 116.4074
    },
    "planets": [
      {
        "name": "太阳",
        "symbol": "Sun",
        "longitude": 251.34,
        "sign": "射手座",
        "degree": 11.34,
        "retrograde": false
      },
      {
        "name": "月亮",
        "symbol": "Moon",
        "longitude": 152.47,
        "sign": "处女座",
        "degree": 2.47,
        "retrograde": false
      }
    ],
    "aspects": [
      {
        "planet1": "太阳",
        "planet2": "月亮",
        "aspect": "四分相",
        "exactAngle": 90,
        "actualAngle": 98.87,
        "orb": 8.87,
        "nature": "紧张"
      }
    ],
    "moonPhase": {
      "phase": "上弦月",
      "illumination": 50,
      "age": 12.5,
      "emoji": "first_quarter"
    },
    "sunrise": "07:20",
    "sunset": "16:40"
  }
}
```

---

### 5. 历史记录 - GET /history

获取历史天象事件数据。

**请求**
```bash
curl -X GET \
  "https://nunotqruohnfoozorqiz.supabase.co/functions/v1/history?limit=10&page=1" \
  -H "Authorization: Bearer <SUPABASE_ANON_KEY>"
```

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| limit | number | 否 | 每页数量，默认10 |
| page | number | 否 | 页码，默认1 |

**响应**
```json
{
  "success": true,
  "data": {
    "events": [
      {
        "id": 1,
        "date": "2024-04-08",
        "title": "日全食",
        "type": "eclipse",
        "category": "日食",
        "description": "北美地区可见的日全食，从墨西哥到加拿大横跨整个北美大陆。",
        "significance": "日食象征着新的开始和转变，是进行重大改变的好时机。",
        "visibility": "北美洲",
        "magnitude": 1.0566,
        "duration": "4分28秒"
      }
    ],
    "pagination": {
      "total": 15,
      "page": 1,
      "limit": 10,
      "totalPages": 2
    }
  }
}
```

---

## 错误处理

所有错误响应遵循以下格式：

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述信息"
  }
}
```

### 常见错误码

| 错误码 | HTTP状态码 | 说明 |
|--------|-----------|------|
| CALCULATION_ERROR | 400 | 计算参数错误 |
| TIMELINE_ERROR | 400 | 时间线参数错误 |
| SKY_DATA_ERROR | 400 | 天象数据参数错误 |
| HISTORY_ERROR | 400 | 历史数据查询错误 |
| HEALTH_CHECK_ERROR | 500 | 健康检查失败 |

---

## CORS配置

所有端点已配置完整的CORS支持：

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Headers: authorization, x-client-info, apikey, content-type
Access-Control-Allow-Methods: POST, GET, OPTIONS, PUT, DELETE, PATCH
```

这确保前端应用可以从任何域名访问API。
