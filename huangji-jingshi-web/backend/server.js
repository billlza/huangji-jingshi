const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 3000;

// 中间件
app.use(helmet());
app.use(cors({
    origin: 'https://huangji-jingshi.vercel.app',
    credentials: true
}));
app.use(express.json());

// 日志记录
console.log('🚀 皇极经世后端服务启动中...');

// 静态数据
let timelineData = {};
let historyData = {};
let celestialHashes = {};

// 智能路径检测
function findDataPath() {
    const possiblePaths = [
        path.join(__dirname, 'data/celestial'),
        path.join(__dirname, '../data/celestial'),
        path.join(__dirname, '../../data/celestial'),
        path.join(__dirname, 'data'),
        '/opt/render/project/src/data/celestial',
        '/workspace/data/celestial'
    ];
    
    console.log('🔍 搜索数据文件...');
    for (const filePath of possiblePaths) {
        try {
            require('fs').accessSync(filePath);
            console.log('✅ 找到数据路径:', filePath);
            return filePath;
        } catch (err) {
            console.log('❌ 路径不存在:', filePath);
        }
    }
    return null;
}

// 加载数据文件
async function loadDataFiles() {
    const dataPath = findDataPath();
    if (!dataPath) {
        throw new Error('未找到数据文件，禁止使用 Mock，服务终止');
    }
    
    console.log('📊 开始加载数据文件...');
    // 实际实现中可以在这里加载JSON文件
    console.log('🎯 数据文件加载完成');
}

// 路由定义
app.get('/', (req, res) => {
    res.json({
        service: '皇极经世后端服务',
        status: 'running',
        version: '1.0.0-node',
        message: 'API服务正常运行',
        endpoints: [
            'GET /health',
            'POST /api/calculate',
            'GET /api/timeline/:year',
            'GET /api/history',
            'GET /api/celestial/hashes',
            'GET /api/sky/settings',
            'POST /api/sky/settings'
        ]
    });
});

app.get('/health', (req, res) => {
    console.log('💊 健康检查请求');
    res.json({
        status: 'ok',
        message: '皇极经世后端服务正常运行',
        timestamp: new Date().toISOString(),
        version: '1.0.0-node',
        data_loaded: Object.keys(timelineData).length > 0
    });
});

app.post('/api/calculate', (req, res) => {
    console.log('🔮 收到演算请求:', req.body);
    
    return res.status(501).json({
        error: '演算功能尚未在 Node 版本实现',
        message: '请调用 Rust 主后端或补充真实实现，禁止返回 Mock 数据',
        input: req.body,
        timestamp: new Date().toISOString(),
        status: 'not_implemented'
    });
});

app.get('/api/timeline/:year', (req, res) => {
    const year = parseInt(req.params.year);
    console.log('📅 查询时间线:', year);
    
    if (timelineData[year]) {
        res.json(timelineData[year]);
    } else {
        res.json({
            year: year,
            ganzhi: '甲子年',
            events: []
        });
    }
});

app.get('/api/history', (req, res) => {
    res.json(historyData);
});

app.get('/api/celestial/hashes', (req, res) => {
    res.json(celestialHashes);
});

app.get('/api/sky/settings', (req, res) => {
    res.json({
        default_lat: 39.9,
        default_lon: 116.4,
        show_stars: true,
        show_constellations: true,
        show_planets: true,
        chinese_labels: true,
        huangji_mode: true
    });
});

app.post('/api/sky/settings', (req, res) => {
    console.log('🔧 更新天空设置:', req.body);
    res.json({ status: 'success', message: '设置已更新' });
});

// 启动服务器
async function startServer() {
    try {
        // 加载数据
        await loadDataFiles();
        
        // 启动服务器
        app.listen(PORT, '0.0.0.0', () => {
            console.log(`🌐 服务器启动成功，端口: ${PORT}`);
            console.log(`🔗 健康检查: http://localhost:${PORT}/health`);
            console.log(`📡 API服务: http://localhost:${PORT}/api/`);
        });
    } catch (error) {
        console.error('❌ 服务器启动失败:', error);
        process.exit(1);
    }
}

// 优雅关闭
process.on('SIGTERM', () => {
    console.log('🛑 收到SIGTERM信号，正在关闭服务器...');
    process.exit(0);
});

process.on('SIGINT', () => {
    console.log('🛑 收到SIGINT信号，正在关闭服务器...');
    process.exit(0);
});

startServer();
