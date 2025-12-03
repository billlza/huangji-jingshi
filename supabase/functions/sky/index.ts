// 天象数据端点 - GET /sky
Deno.serve(async (req) => {
    const corsHeaders = {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Headers': 'authorization, x-client-info, apikey, content-type',
        'Access-Control-Allow-Methods': 'POST, GET, OPTIONS, PUT, DELETE, PATCH',
        'Access-Control-Max-Age': '86400',
        'Access-Control-Allow-Credentials': 'false'
    };

    if (req.method === 'OPTIONS') {
        return new Response(null, { status: 200, headers: corsHeaders });
    }

    try {
        const url = new URL(req.url);
        const dateParam = url.searchParams.get('date') || new Date().toISOString().split('T')[0];
        const latitude = parseFloat(url.searchParams.get('latitude') || '39.9042');
        const longitude = parseFloat(url.searchParams.get('longitude') || '116.4074');

        const skyData = calculateSkyData(dateParam, latitude, longitude);

        const response = {
            success: true,
            data: {
                date: dateParam,
                location: {
                    latitude: latitude,
                    longitude: longitude
                },
                planets: skyData.planets,
                aspects: skyData.aspects,
                moonPhase: skyData.moonPhase,
                sunrise: skyData.sunrise,
                sunset: skyData.sunset
            }
        };

        return new Response(JSON.stringify(response), {
            status: 200,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    } catch (error) {
        return new Response(JSON.stringify({
            success: false,
            error: {
                code: 'SKY_DATA_ERROR',
                message: error.message
            }
        }), {
            status: 400,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }
});

// 行星数据
interface PlanetData {
    name: string;
    symbol: string;
    longitude: number;
    sign: string;
    degree: number;
    retrograde: boolean;
}

// 黄道十二宫
const zodiacSigns = [
    '白羊座', '金牛座', '双子座', '巨蟹座',
    '狮子座', '处女座', '天秤座', '天蝎座',
    '射手座', '摩羯座', '水瓶座', '双鱼座'
];

// 计算天象数据
function calculateSkyData(dateStr: string, latitude: number, longitude: number): {
    planets: PlanetData[];
    aspects: Array<object>;
    moonPhase: object;
    sunrise: string;
    sunset: string;
} {
    const date = new Date(dateStr);
    const dayOfYear = getDayOfYear(date);
    
    // 基于日期生成行星位置
    const planets = generatePlanetPositions(dayOfYear);
    
    // 计算行星相位
    const aspects = calculateAspects(planets);
    
    // 月相计算
    const moonPhase = calculateMoonPhase(date);
    
    // 日出日落时间 (模拟)
    const { sunrise, sunset } = calculateSunTimes(date, latitude);
    
    return {
        planets,
        aspects,
        moonPhase,
        sunrise,
        sunset
    };
}

// 生成行星位置
function generatePlanetPositions(dayOfYear: number): PlanetData[] {
    const planets = [
        { name: '太阳', symbol: 'Sun', baseSpeed: 0.9856 },
        { name: '月亮', symbol: 'Moon', baseSpeed: 13.176 },
        { name: '水星', symbol: 'Mercury', baseSpeed: 1.383 },
        { name: '金星', symbol: 'Venus', baseSpeed: 1.2 },
        { name: '火星', symbol: 'Mars', baseSpeed: 0.524 },
        { name: '木星', symbol: 'Jupiter', baseSpeed: 0.083 },
        { name: '土星', symbol: 'Saturn', baseSpeed: 0.033 },
        { name: '天王星', symbol: 'Uranus', baseSpeed: 0.012 },
        { name: '海王星', symbol: 'Neptune', baseSpeed: 0.006 },
        { name: '冥王星', symbol: 'Pluto', baseSpeed: 0.004 }
    ];
    
    return planets.map((planet, index) => {
        // 使用基础速度和日期计算黄道经度
        const baseLongitude = (dayOfYear * planet.baseSpeed + index * 30) % 360;
        
        // 添加一些变化
        const variation = Math.sin(dayOfYear * 0.017 + index) * 5;
        const longitude = (baseLongitude + variation + 360) % 360;
        
        // 计算所在星座和度数
        const signIndex = Math.floor(longitude / 30);
        const degree = longitude % 30;
        
        // 逆行判断 (简化逻辑)
        const retrograde = planet.name !== '太阳' && planet.name !== '月亮' && 
                          Math.sin(dayOfYear * 0.02 + index * 0.5) > 0.7;
        
        return {
            name: planet.name,
            symbol: planet.symbol,
            longitude: Math.round(longitude * 100) / 100,
            sign: zodiacSigns[signIndex],
            degree: Math.round(degree * 100) / 100,
            retrograde: retrograde
        };
    });
}

// 计算行星相位
function calculateAspects(planets: PlanetData[]): Array<object> {
    const aspects = [];
    const aspectTypes = [
        { name: '合相', angle: 0, orb: 10, nature: '强化' },
        { name: '六分相', angle: 60, orb: 6, nature: '和谐' },
        { name: '四分相', angle: 90, orb: 8, nature: '紧张' },
        { name: '三分相', angle: 120, orb: 8, nature: '和谐' },
        { name: '对分相', angle: 180, orb: 10, nature: '对立' }
    ];
    
    // 检查主要行星之间的相位
    const mainPlanets = planets.slice(0, 5);
    
    for (let i = 0; i < mainPlanets.length; i++) {
        for (let j = i + 1; j < mainPlanets.length; j++) {
            const diff = Math.abs(mainPlanets[i].longitude - mainPlanets[j].longitude);
            const angle = diff > 180 ? 360 - diff : diff;
            
            for (const aspectType of aspectTypes) {
                if (Math.abs(angle - aspectType.angle) <= aspectType.orb) {
                    aspects.push({
                        planet1: mainPlanets[i].name,
                        planet2: mainPlanets[j].name,
                        aspect: aspectType.name,
                        exactAngle: aspectType.angle,
                        actualAngle: Math.round(angle * 100) / 100,
                        orb: Math.round(Math.abs(angle - aspectType.angle) * 100) / 100,
                        nature: aspectType.nature
                    });
                    break;
                }
            }
        }
    }
    
    return aspects;
}

// 计算月相
function calculateMoonPhase(date: Date): object {
    // 简化的月相计算
    const synMonth = 29.530588853;
    const baseDate = new Date(2000, 0, 6, 18, 14, 0); // 已知新月时间
    const diff = (date.getTime() - baseDate.getTime()) / (1000 * 60 * 60 * 24);
    const phase = ((diff % synMonth) / synMonth) * 100;
    
    let phaseName: string;
    let illumination: number;
    
    if (phase < 1.85) {
        phaseName = '新月';
        illumination = 0;
    } else if (phase < 25) {
        phaseName = '蛾眉月';
        illumination = phase * 2;
    } else if (phase < 50) {
        phaseName = '上弦月';
        illumination = 50;
    } else if (phase < 51.85) {
        phaseName = '盈凸月';
        illumination = 50 + phase;
    } else if (phase < 53.7) {
        phaseName = '满月';
        illumination = 100;
    } else if (phase < 75) {
        phaseName = '亏凸月';
        illumination = 100 - (phase - 50) * 2;
    } else {
        phaseName = '下弦月';
        illumination = 50 - (phase - 75) * 2;
    }
    
    return {
        phase: phaseName,
        illumination: Math.round(Math.max(0, Math.min(100, illumination))),
        age: Math.round((diff % synMonth) * 10) / 10,
        emoji: getMoonEmoji(phaseName)
    };
}

// 获取月相表情
function getMoonEmoji(phase: string): string {
    const emojis: Record<string, string> = {
        '新月': 'new_moon',
        '蛾眉月': 'waxing_crescent',
        '上弦月': 'first_quarter',
        '盈凸月': 'waxing_gibbous',
        '满月': 'full_moon',
        '亏凸月': 'waning_gibbous',
        '下弦月': 'last_quarter',
        '残月': 'waning_crescent'
    };
    return emojis[phase] || 'moon';
}

// 计算日出日落时间
function calculateSunTimes(date: Date, latitude: number): { sunrise: string; sunset: string } {
    // 简化计算 - 基于纬度和季节
    const dayOfYear = getDayOfYear(date);
    
    // 计算日照时长变化
    const declination = 23.45 * Math.sin((360 / 365) * (dayOfYear - 81) * Math.PI / 180);
    const hourAngle = Math.acos(-Math.tan(latitude * Math.PI / 180) * Math.tan(declination * Math.PI / 180));
    const daylightHours = 2 * hourAngle * 180 / (15 * Math.PI);
    
    // 计算日出日落时间 (假设正午为12:00)
    const sunriseHour = 12 - daylightHours / 2;
    const sunsetHour = 12 + daylightHours / 2;
    
    return {
        sunrise: formatTime(sunriseHour),
        sunset: formatTime(sunsetHour)
    };
}

// 获取年中的第几天
function getDayOfYear(date: Date): number {
    const start = new Date(date.getFullYear(), 0, 0);
    const diff = date.getTime() - start.getTime();
    return Math.floor(diff / (1000 * 60 * 60 * 24));
}

// 格式化时间
function formatTime(hours: number): string {
    const h = Math.floor(hours);
    const m = Math.round((hours - h) * 60);
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
}
