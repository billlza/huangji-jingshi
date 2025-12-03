// 天机演算计算端点 - POST /calculate
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
        const { birthDate, birthTime, location } = await req.json();

        if (!birthDate) {
            throw new Error('birthDate is required');
        }

        // 解析出生日期
        const date = new Date(birthDate);
        const month = date.getMonth() + 1;
        const day = date.getDate();

        // 根据日期计算太阳星座
        const sunSign = getSunSign(month, day);
        
        // 模拟月亮星座 (基于日期的hash)
        const moonSign = getMoonSign(birthDate, birthTime);
        
        // 模拟上升星座 (基于时间和位置)
        const ascendantSign = getAscendantSign(birthTime, location);

        // 元素分布计算
        const elements = calculateElements(sunSign, moonSign, ascendantSign);

        // 宫位信息
        const houses = generateHouses(sunSign, moonSign, ascendantSign);

        const response = {
            success: true,
            data: {
                sunSign: {
                    name: sunSign,
                    degree: Math.floor(Math.random() * 30),
                    element: getElement(sunSign),
                    quality: getQuality(sunSign)
                },
                moonSign: {
                    name: moonSign,
                    degree: Math.floor(Math.random() * 30),
                    element: getElement(moonSign),
                    quality: getQuality(moonSign)
                },
                ascendant: {
                    name: ascendantSign,
                    degree: Math.floor(Math.random() * 30),
                    element: getElement(ascendantSign),
                    quality: getQuality(ascendantSign)
                },
                elements: elements,
                houses: houses,
                birthInfo: {
                    date: birthDate,
                    time: birthTime || '12:00',
                    location: location || '未知'
                }
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
                code: 'CALCULATION_ERROR',
                message: error.message
            }
        }), {
            status: 400,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }
});

// 星座列表
const zodiacSigns = [
    '白羊座', '金牛座', '双子座', '巨蟹座',
    '狮子座', '处女座', '天秤座', '天蝎座',
    '射手座', '摩羯座', '水瓶座', '双鱼座'
];

// 根据月日获取太阳星座
function getSunSign(month: number, day: number): string {
    const dates = [
        [1, 20], [2, 19], [3, 21], [4, 20], [5, 21], [6, 21],
        [7, 23], [8, 23], [9, 23], [10, 23], [11, 22], [12, 22]
    ];
    
    for (let i = 0; i < 12; i++) {
        if (month === dates[i][0] && day < dates[i][1]) {
            return zodiacSigns[(i + 11) % 12];
        }
        if (month === dates[i][0] && day >= dates[i][1]) {
            return zodiacSigns[i];
        }
    }
    return zodiacSigns[9]; // 默认摩羯座
}

// 模拟月亮星座
function getMoonSign(birthDate: string, birthTime: string): string {
    const hash = simpleHash(birthDate + (birthTime || ''));
    return zodiacSigns[hash % 12];
}

// 模拟上升星座
function getAscendantSign(birthTime: string, location: string): string {
    const hash = simpleHash((birthTime || '12:00') + (location || ''));
    return zodiacSigns[hash % 12];
}

// 简单hash函数
function simpleHash(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash;
    }
    return Math.abs(hash);
}

// 获取星座对应的元素
function getElement(sign: string): string {
    const elements: Record<string, string> = {
        '白羊座': '火', '狮子座': '火', '射手座': '火',
        '金牛座': '土', '处女座': '土', '摩羯座': '土',
        '双子座': '风', '天秤座': '风', '水瓶座': '风',
        '巨蟹座': '水', '天蝎座': '水', '双鱼座': '水'
    };
    return elements[sign] || '未知';
}

// 获取星座对应的属性
function getQuality(sign: string): string {
    const qualities: Record<string, string> = {
        '白羊座': '本位', '巨蟹座': '本位', '天秤座': '本位', '摩羯座': '本位',
        '金牛座': '固定', '狮子座': '固定', '天蝎座': '固定', '水瓶座': '固定',
        '双子座': '变动', '处女座': '变动', '射手座': '变动', '双鱼座': '变动'
    };
    return qualities[sign] || '未知';
}

// 计算元素分布
function calculateElements(sun: string, moon: string, asc: string): Record<string, number> {
    const elements: Record<string, number> = { '火': 0, '土': 0, '风': 0, '水': 0 };
    
    elements[getElement(sun)] += 3;
    elements[getElement(moon)] += 2;
    elements[getElement(asc)] += 1;
    
    // 添加一些随机分布
    const keys = Object.keys(elements);
    for (const key of keys) {
        elements[key] += Math.floor(Math.random() * 2);
    }
    
    return elements;
}

// 生成宫位信息
function generateHouses(sun: string, moon: string, asc: string): Array<{house: number, sign: string, planets: string[]}> {
    const houses = [];
    const ascIndex = zodiacSigns.indexOf(asc);
    const planets = ['太阳', '月亮', '水星', '金星', '火星', '木星', '土星', '天王星', '海王星', '冥王星'];
    
    for (let i = 1; i <= 12; i++) {
        const signIndex = (ascIndex + i - 1) % 12;
        const housePlanets: string[] = [];
        
        // 随机分配行星到宫位
        if (i === 1) housePlanets.push('上升点');
        if (zodiacSigns[signIndex] === sun) housePlanets.push('太阳');
        if (zodiacSigns[signIndex] === moon) housePlanets.push('月亮');
        
        // 随机添加其他行星
        for (const planet of planets.slice(2)) {
            if (Math.random() < 0.15) {
                housePlanets.push(planet);
            }
        }
        
        houses.push({
            house: i,
            sign: zodiacSigns[signIndex],
            planets: housePlanets
        });
    }
    
    return houses;
}
