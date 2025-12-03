const express = require('express');
const cors = require('cors');

const app = express();
const PORT = process.env.PORT || 3000;

// Enable CORS for all routes
app.use(cors());
app.use(express.json());

// 临时模拟数据
const MOCK_HISTORY = [
  {
    "date": "2025-12-03",
    "event": "天机演算系统启动成功",
    "type": "system",
    "timestamp": "2025-12-03T09:30:00Z"
  },
  {
    "date": "2025-12-01", 
    "event": "新的天象观测数据入库",
    "type": "observation",
    "timestamp": "2025-12-01T14:20:00Z"
  },
  {
    "date": "2025-11-28",
    "event": "天机算法优化完成",
    "type": "algorithm", 
    "timestamp": "2025-11-28T16:45:00Z"
  }
];

const MOCK_TIMELINE = {
  "events": [
    {
      "date": "2025-12-03",
      "phase": "新月",
      "significance": "重要决策时刻",
      "description": "今日新月，适合开始新的计划和项目"
    },
    {
      "date": "2025-12-10", 
      "phase": "上弦月",
      "significance": "行动时机",
      "description": "上弦月期间，适合推进既定的计划和目标"
    },
    {
      "date": "2025-12-17",
      "phase": "满月",
      "significance": "能量高峰",
      "description": "满月时刻，能量达到峰值，适合重要的沟通和表达"
    }
  ]
};

const MOCK_CELESTIAL_DATA = {
  "sun": {
    "longitude": 251.8,
    "declination": -22.4,
    "right_ascension": "16h 47m",
    "sign": "天蝎座"
  },
  "moon": {
    "longitude": 78.2,
    "declination": 15.7,
    "right_ascension": "5h 13m", 
    "phase": "上弦月"
  },
  "mercury": {
    "longitude": 284.3,
    "sign": "射手座",
    "retrograde": false
  },
  "venus": {
    "longitude": 310.1,
    "sign": "摩羯座",
    "phase": "蛾眉月"
  },
  "mars": {
    "longitude": 189.5,
    "sign": "处女座",
    "retrograde": false
  }
};

// 健康检查端点
app.get('/health', (req, res) => {
  res.json({
    status: "healthy",
    timestamp: new Date().toISOString(),
    service: "huangji-jingshi-backend",
    version: "1.0.0"
  });
});

// 演算端点
app.post('/calculate', async (req, res) => {
  try {
    const { birthDate, birthTime, location } = req.body;
    
    if (!birthDate || !birthTime || !location) {
      return res.status(400).json({
        error: "缺少必要参数：birthDate, birthTime, location"
      });
    }

    // 模拟计算过程
    const calculationId = `calc_${Date.now()}`;
    
    // 延迟模拟计算时间
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    const result = {
      calculation_id: calculationId,
      input: {
        birth_date: birthDate,
        birth_time: birthTime,
        location: location
      },
      calculation_result: {
        sun_sign: "天蝎座",
        moon_sign: "双子座",
        rising_sign: "处女座",
        elements: {
          fire: 25,
          earth: 30,
          air: 25,
          water: 20
        },
        houses: [
          { house: 1, sign: "处女座", planet: "水星" },
          { house: 2, sign: "天秤座", planet: "金星" },
          { house: 3, sign: "天蝎座", planet: "火星" }
        ]
      },
      timestamp: new Date().toISOString()
    };

    res.json(result);
  } catch (error) {
    console.error('计算错误:', error);
    res.status(500).json({
      error: "计算过程中发生错误",
      message: error.message
    });
  }
});

// 时间线端点
app.post('/timeline', async (req, res) => {
  try {
    const { birthDate, duration = 12 } = req.body;
    
    if (!birthDate) {
      return res.status(400).json({
        error: "缺少必要参数：birthDate"
      });
    }

    // 模拟计算过程
    await new Promise(resolve => setTimeout(resolve, 500));

    const result = {
      birth_date: birthDate,
      duration_months: duration,
      timeline: MOCK_TIMELINE.events.map(event => ({
        ...event,
        calculation_id: `calc_${Date.now()}`,
        phase: event.phase,
        significance: event.significance,
        description: event.description
      })),
      timestamp: new Date().toISOString()
    };

    res.json(result);
  } catch (error) {
    console.error('时间线计算错误:', error);
    res.status(500).json({
      error: "时间线计算过程中发生错误",
      message: error.message
    });
  }
});

// 天象数据端点
app.get('/sky', async (req, res) => {
  try {
    const { date, latitude, longitude } = req.query;
    
    // 模拟计算过程
    await new Promise(resolve => setTimeout(resolve, 300));

    const result = {
      query: {
        date: date || new Date().toISOString().split('T')[0],
        latitude: latitude || "39.9042",
        longitude: longitude || "116.4074"
      },
      celestial_data: MOCK_CELESTIAL_DATA,
      calculation_quality: "high",
      timestamp: new Date().toISOString()
    };

    res.json(result);
  } catch (error) {
    console.error('天象数据错误:', error);
    res.status(500).json({
      error: "获取天象数据时发生错误",
      message: error.message
    });
  }
});

// 历史记录端点 (新增)
app.get('/history', (req, res) => {
  res.json({
    history: MOCK_HISTORY,
    total_count: MOCK_HISTORY.length,
    timestamp: new Date().toISOString()
  });
});

// 错误处理中间件
app.use((err, req, res, next) => {
  console.error('服务器错误:', err);
  res.status(500).json({
    error: "内部服务器错误",
    message: err.message
  });
});

// 404处理
app.use((req, res) => {
  res.status(404).json({
    error: "端点不存在",
    available_endpoints: [
      "GET /health",
      "POST /calculate",
      "POST /timeline", 
      "GET /sky",
      "GET /history"
    ]
  });
});

// 启动服务器
app.listen(PORT, '0.0.0.0', () => {
  console.log(`黄极经世后端服务运行在端口 ${PORT}`);
  console.log(`健康检查: http://localhost:${PORT}/health`);
  console.log(`可用端点:`);
  console.log(`  - POST /calculate`);
  console.log(`  - POST /timeline`);
  console.log(`  - GET /sky`);
  console.log(`  - GET /history`);
});

module.exports = app;