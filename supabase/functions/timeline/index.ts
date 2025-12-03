// 时间线计算端点 - POST /timeline
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
        const { birthDate, duration } = await req.json();

        if (!birthDate) {
            throw new Error('birthDate is required');
        }

        const months = duration || 12;
        const timeline = generateTimeline(birthDate, months);

        const response = {
            success: true,
            data: {
                birthDate: birthDate,
                duration: months,
                timeline: timeline
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
                code: 'TIMELINE_ERROR',
                message: error.message
            }
        }), {
            status: 400,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }
});

// 天象事件类型
const cosmicEvents = [
    { type: '新月', impact: '新开始', area: '事业' },
    { type: '满月', impact: '收获期', area: '财运' },
    { type: '水星逆行', impact: '反思期', area: '沟通' },
    { type: '金星入相', impact: '桃花运', area: '感情' },
    { type: '火星冲日', impact: '能量高涨', area: '健康' },
    { type: '木星合相', impact: '好运期', area: '贵人' },
    { type: '土星回归', impact: '考验期', area: '责任' },
    { type: '日食', impact: '转折点', area: '人生' },
    { type: '月食', impact: '情绪释放', area: '内心' },
    { type: '行星换座', impact: '能量转换', area: '整体' }
];

// 月份运势等级
const fortuneLevels = ['极佳', '良好', '平稳', '需注意', '挑战'];

// 生成时间线
function generateTimeline(birthDate: string, months: number): Array<object> {
    const timeline = [];
    const startDate = new Date();
    
    for (let i = 0; i < months; i++) {
        const currentDate = new Date(startDate);
        currentDate.setMonth(currentDate.getMonth() + i);
        
        const monthEvents = generateMonthEvents(birthDate, currentDate, i);
        const fortune = calculateMonthFortune(birthDate, currentDate);
        
        timeline.push({
            month: formatMonth(currentDate),
            year: currentDate.getFullYear(),
            monthIndex: currentDate.getMonth() + 1,
            fortune: fortune,
            events: monthEvents,
            advice: generateAdvice(fortune, monthEvents)
        });
    }
    
    return timeline;
}

// 生成月度事件
function generateMonthEvents(birthDate: string, currentDate: Date, offset: number): Array<object> {
    const events = [];
    const hash = simpleHash(birthDate + currentDate.toISOString());
    const eventCount = 1 + (hash % 3);
    
    for (let i = 0; i < eventCount; i++) {
        const eventIndex = (hash + i * 7) % cosmicEvents.length;
        const event = cosmicEvents[eventIndex];
        
        // 生成事件日期 (在当月内)
        const eventDay = 1 + ((hash + i * 13) % 28);
        const eventDate = new Date(currentDate.getFullYear(), currentDate.getMonth(), eventDay);
        
        events.push({
            date: formatDate(eventDate),
            type: event.type,
            impact: event.impact,
            area: event.area,
            intensity: (hash % 5) + 1,
            description: generateEventDescription(event, offset)
        });
    }
    
    return events;
}

// 计算月度运势
function calculateMonthFortune(birthDate: string, currentDate: Date): object {
    const hash = simpleHash(birthDate + currentDate.getMonth().toString());
    
    return {
        overall: fortuneLevels[hash % 5],
        career: Math.floor((hash % 100) / 10) + 1,
        love: Math.floor(((hash * 3) % 100) / 10) + 1,
        wealth: Math.floor(((hash * 7) % 100) / 10) + 1,
        health: Math.floor(((hash * 11) % 100) / 10) + 1,
        luckyDays: generateLuckyDays(hash, currentDate),
        luckyColor: getLuckyColor(hash),
        luckyNumber: (hash % 9) + 1
    };
}

// 生成幸运日
function generateLuckyDays(hash: number, currentDate: Date): number[] {
    const days = [];
    const count = 2 + (hash % 3);
    for (let i = 0; i < count; i++) {
        days.push(1 + ((hash + i * 7) % 28));
    }
    return [...new Set(days)].sort((a, b) => a - b);
}

// 获取幸运颜色
function getLuckyColor(hash: number): string {
    const colors = ['红色', '橙色', '黄色', '绿色', '蓝色', '紫色', '白色', '金色', '银色'];
    return colors[hash % colors.length];
}

// 生成事件描述
function generateEventDescription(event: { type: string; impact: string; area: string }, offset: number): string {
    const descriptions: Record<string, string[]> = {
        '新月': ['适合开启新项目', '播种希望的种子', '制定新计划的好时机'],
        '满月': ['情绪可能波动', '适合完成未竟之事', '收获前期努力的成果'],
        '水星逆行': ['注意沟通误会', '适合回顾过去', '检查重要文件'],
        '金星入相': ['桃花运旺盛', '艺术灵感涌现', '财运有所提升'],
        '火星冲日': ['精力充沛', '适合运动健身', '避免冲动行事'],
        '木星合相': ['贵人相助', '学业进步', '视野开阔'],
        '土星回归': ['责任感加重', '需要脚踏实地', '适合长远规划'],
        '日食': ['重大转变期', '新的开始', '放下过去'],
        '月食': ['情绪释放', '内心觉醒', '潜意识浮现'],
        '行星换座': ['能量转换', '适应新节奏', '灵活应变']
    };
    
    const eventDescriptions = descriptions[event.type] || ['宇宙能量影响'];
    return eventDescriptions[offset % eventDescriptions.length];
}

// 生成建议
function generateAdvice(fortune: { overall: string }, events: Array<object>): string {
    const advices: Record<string, string> = {
        '极佳': '运势极佳，把握机遇，大胆前行。',
        '良好': '运势良好，稳步推进，适度冒险。',
        '平稳': '运势平稳，保持现状，静待时机。',
        '需注意': '需要注意细节，谨慎行事，避免大动作。',
        '挑战': '面临挑战，保持耐心，静观其变。'
    };
    
    return advices[fortune.overall] || '顺其自然，保持平常心。';
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

// 格式化月份
function formatMonth(date: Date): string {
    const months = ['一月', '二月', '三月', '四月', '五月', '六月', 
                   '七月', '八月', '九月', '十月', '十一月', '十二月'];
    return months[date.getMonth()];
}

// 格式化日期
function formatDate(date: Date): string {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
}
