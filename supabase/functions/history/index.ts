// 历史记录端点 - GET /history
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
        const limit = parseInt(url.searchParams.get('limit') || '10');
        const page = parseInt(url.searchParams.get('page') || '1');

        // 生成模拟的历史事件数据
        const historyEvents = generateHistoryEvents();
        
        // 分页处理
        const startIndex = (page - 1) * limit;
        const endIndex = startIndex + limit;
        const paginatedEvents = historyEvents.slice(startIndex, endIndex);

        const response = {
            success: true,
            data: {
                events: paginatedEvents,
                pagination: {
                    total: historyEvents.length,
                    page: page,
                    limit: limit,
                    totalPages: Math.ceil(historyEvents.length / limit)
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
                code: 'HISTORY_ERROR',
                message: error.message
            }
        }), {
            status: 400,
            headers: { ...corsHeaders, 'Content-Type': 'application/json' }
        });
    }
});

// 历史天象事件数据
function generateHistoryEvents(): Array<object> {
    const events = [
        {
            id: 1,
            date: '2024-04-08',
            title: '日全食',
            type: 'eclipse',
            category: '日食',
            description: '北美地区可见的日全食，从墨西哥到加拿大横跨整个北美大陆。',
            significance: '日食象征着新的开始和转变，是进行重大改变的好时机。',
            visibility: '北美洲',
            magnitude: 1.0566,
            duration: '4分28秒'
        },
        {
            id: 2,
            date: '2024-03-25',
            title: '半影月食',
            type: 'eclipse',
            category: '月食',
            description: '天秤座满月伴随半影月食，影响情感关系和平衡。',
            significance: '月食期间适合释放旧有模式，清理情绪负担。',
            visibility: '美洲、欧洲、非洲',
            magnitude: 0.9577,
            duration: '4小时39分钟'
        },
        {
            id: 3,
            date: '2024-04-01',
            title: '水星逆行开始',
            type: 'retrograde',
            category: '逆行',
            description: '水星在白羊座逆行，影响沟通、交通和电子设备。',
            significance: '水逆期间适合回顾过去，避免签署重要合同。',
            visibility: '全球',
            startDate: '2024-04-01',
            endDate: '2024-04-25'
        },
        {
            id: 4,
            date: '2024-05-23',
            title: '木星进入双子座',
            type: 'ingress',
            category: '行星换座',
            description: '木星离开金牛座进入双子座，为期一年的学习扩展期。',
            significance: '木星在双子座带来智识上的成长和沟通机会的扩展。',
            visibility: '全球',
            duration: '约1年'
        },
        {
            id: 5,
            date: '2024-08-19',
            title: '水瓶座满月',
            type: 'full_moon',
            category: '满月',
            description: '水瓶座超级满月，激发创新思维和集体意识。',
            significance: '适合群体活动和推进社会改革项目。',
            visibility: '全球',
            illumination: 100
        },
        {
            id: 6,
            date: '2024-10-02',
            title: '日环食',
            type: 'eclipse',
            category: '日食',
            description: '天秤座日环食，强调人际关系和伙伴议题。',
            significance: '日环食期间关系中的重要主题会浮现。',
            visibility: '南美洲、南太平洋',
            magnitude: 0.9326,
            duration: '7分25秒'
        },
        {
            id: 7,
            date: '2024-11-26',
            title: '冥王星逆行结束',
            type: 'retrograde_end',
            category: '逆行结束',
            description: '冥王星在水瓶座顺行，开启新的权力动态周期。',
            significance: '转化能量向前推进，适合实施深层改变。',
            visibility: '全球'
        },
        {
            id: 8,
            date: '2025-01-03',
            title: '火星逆行开始',
            type: 'retrograde',
            category: '逆行',
            description: '火星在狮子座逆行，影响行动力和创意表达。',
            significance: '火逆期间适合重新评估目标和行动策略。',
            visibility: '全球',
            startDate: '2025-01-03',
            endDate: '2025-02-24'
        },
        {
            id: 9,
            date: '2025-03-14',
            title: '月全食',
            type: 'eclipse',
            category: '月食',
            description: '处女座满月伴随月全食，关注健康和日常习惯。',
            significance: '适合调整生活方式和工作模式。',
            visibility: '美洲、欧洲、非洲',
            magnitude: 1.1784,
            duration: '1小时5分钟'
        },
        {
            id: 10,
            date: '2025-03-29',
            title: '日偏食',
            type: 'eclipse',
            category: '日食',
            description: '白羊座新月伴随日偏食，新起点的能量。',
            significance: '适合设定新目标，开启新项目。',
            visibility: '欧洲、北非、俄罗斯',
            magnitude: 0.9377
        },
        {
            id: 11,
            date: '2025-05-12',
            title: '木星进入巨蟹座',
            type: 'ingress',
            category: '行星换座',
            description: '木星进入巨蟹座，家庭和情感领域受到祝福。',
            significance: '木星擢升在巨蟹座，带来情感安全感和家庭扩展。',
            visibility: '全球',
            duration: '约1年'
        },
        {
            id: 12,
            date: '2025-09-07',
            title: '月全食',
            type: 'eclipse',
            category: '月食',
            description: '双鱼座超级满月伴随月全食，灵性觉醒的高潮。',
            significance: '适合冥想、艺术创作和灵性实践。',
            visibility: '欧洲、非洲、亚洲、澳洲',
            magnitude: 1.3620,
            duration: '1小时22分钟'
        },
        {
            id: 13,
            date: '2025-09-21',
            title: '日环食',
            type: 'eclipse',
            category: '日食',
            description: '处女座日环食，强调服务和实用主义。',
            significance: '适合改善工作效率和健康习惯。',
            visibility: '南太平洋、南美洲南部',
            magnitude: 0.9550
        },
        {
            id: 14,
            date: '2025-11-09',
            title: '土星进入白羊座',
            type: 'ingress',
            category: '行星换座',
            description: '土星离开双鱼座进入白羊座，开启新的责任周期。',
            significance: '土星在白羊座要求勇敢面对挑战，承担领导责任。',
            visibility: '全球',
            duration: '约2.5年'
        },
        {
            id: 15,
            date: '2025-12-25',
            title: '火星逆行开始',
            type: 'retrograde',
            category: '逆行',
            description: '火星在狮子座逆行，反思个人意志和创意方向。',
            significance: '适合回顾过去的行动，调整未来策略。',
            visibility: '全球',
            startDate: '2025-12-25',
            endDate: '2026-02-13'
        }
    ];
    
    return events;
}
