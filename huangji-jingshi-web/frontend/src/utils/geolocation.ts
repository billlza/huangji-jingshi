/**
 * 地理位置工具函数
 * 根据用户所在地区智能选择最优API调用策略
 */

// 检测用户是否在中国大陆
let isInChina: boolean | null = null;
let detectionPromise: Promise<boolean> | null = null;

export async function detectChinaMainland(): Promise<boolean> {
  // 如果已经检测过，直接返回结果
  if (isInChina !== null) {
    return isInChina;
  }

  // 如果正在检测中，返回同一个Promise（避免重复检测）
  if (detectionPromise) {
    return detectionPromise;
  }

  detectionPromise = (async () => {
    try {
      // 方法1: 尝试访问国内特定的快速API（应该很快返回）
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 3000);
      
      const res = await fetch('http://ip-api.com/json/?fields=countryCode,status', {
        signal: controller.signal
      });
      
      clearTimeout(timeout);
      
      if (res.ok) {
        const data = await res.json();
        // CN = 中国大陆
        const result = data.status === 'success' && data.countryCode === 'CN';
        isInChina = result;
        console.log('🌍 地理位置检测:', result ? '中国大陆' : '海外');
        return result;
      }
    } catch (error) {
      console.log('地理位置检测失败，默认使用中转方案', error);
    }

    // 方法2: 通过时区推测（不够准确但可作为备用）
    try {
      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
      const chinaTimezones = ['Asia/Shanghai', 'Asia/Chongqing', 'Asia/Urumqi', 'Asia/Hong_Kong', 'Asia/Macau'];
      
      if (chinaTimezones.includes(tz)) {
        console.log('⏰ 通过时区推测: 可能在中国区域');
        isInChina = true;
        return true;
      }
    } catch (error) {
      console.log('时区检测失败', error);
    }

    // 默认假设在大陆（安全起见，使用中转方案）
    console.log('⚠️ 无法确定地理位置，默认使用中转方案');
    isInChina = true;
    return true;
  })();

  return detectionPromise;
}

/**
 * 逆地理编码：经纬度转地名
 */
export async function reverseGeocode(latitude: number, longitude: number): Promise<string> {
  const inChina = await detectChinaMainland();
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';

  // 策略1: 中国大陆用户 - 优先使用后端中转
  if (inChina) {
    console.log('📍 [大陆用户] 使用后端中转进行逆地理编码');
    
    // 尝试后端代理
    if (API_BASE) {
      try {
        const res = await fetch(
          `${API_BASE}/api/geocode/reverse?lat=${latitude}&lon=${longitude}`,
          { signal: AbortSignal.timeout(8000) }
        );
        
        if (res.ok) {
          const data = await res.json();
          if (data.location) {
            console.log('✅ 后端中转成功:', data.location);
            return data.location;
          }
        }
      } catch (error) {
        console.log('❌ 后端中转失败', error);
      }
    }
    
    // 降级: 尝试 BigDataCloud（大陆可访问）
    try {
      const res = await fetch(
        `https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${latitude}&longitude=${longitude}&localityLanguage=zh`,
        { signal: AbortSignal.timeout(5000) }
      );
      
      if (res.ok) {
        const data = await res.json();
        const location = data.city || data.locality || data.principalSubdivision || '未知地点';
        console.log('✅ BigDataCloud 成功:', location);
        return location;
      }
    } catch (error) {
      console.log('❌ BigDataCloud 失败', error);
    }
    
    return '';
  }

  // 策略2: 海外用户 - 直接调用国际API（更快）
  console.log('🌐 [海外用户] 直接调用国际API');
  
  // 优先 OpenStreetMap（海外速度快）
  try {
    const res = await fetch(
      `https://nominatim.openstreetmap.org/reverse?lat=${latitude}&lon=${longitude}&format=json&accept-language=zh-CN`,
      {
        signal: AbortSignal.timeout(5000),
        headers: { 'User-Agent': 'HuangjiJingshiWeb/1.0' }
      }
    );
    
    if (res.ok) {
      const data = await res.json();
      const address = data.address;
      const location = address.city || address.town || address.county || address.state || '未知地点';
      console.log('✅ OpenStreetMap 成功:', location);
      return location;
    }
  } catch (error) {
    console.log('❌ OpenStreetMap 失败', error);
  }
  
  // 降级: BigDataCloud
  try {
    const res = await fetch(
      `https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${latitude}&longitude=${longitude}&localityLanguage=zh`,
      { signal: AbortSignal.timeout(5000) }
    );
    
    if (res.ok) {
      const data = await res.json();
      const location = data.city || data.locality || data.principalSubdivision || '未知地点';
      console.log('✅ BigDataCloud 成功:', location);
      return location;
    }
  } catch (error) {
    console.log('❌ BigDataCloud 失败', error);
  }
  
  return '';
}

/**
 * 地理编码：地址转经纬度
 */
export async function geocode(address: string): Promise<{
  latitude: number;
  longitude: number;
  address: string;
} | null> {
  const inChina = await detectChinaMainland();
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';

  // 策略1: 中国大陆用户 - 优先使用后端中转
  if (inChina) {
    console.log('📍 [大陆用户] 使用后端中转进行地理编码');
    
    if (API_BASE) {
      try {
        const res = await fetch(
          `${API_BASE}/api/geocode?address=${encodeURIComponent(address)}`,
          { signal: AbortSignal.timeout(10000) }
        );
        
        if (res.ok) {
          const data = await res.json();
          if (data.latitude && data.longitude) {
            console.log('✅ 后端中转地理编码成功:', data.address);
            return {
              latitude: data.latitude,
              longitude: data.longitude,
              address: data.address || address
            };
          } else if (data.error) {
            console.log('❌ 后端中转地理编码失败:', data.error);
            return null;
          }
        }
      } catch (error) {
        console.log('❌ 后端中转地理编码失败', error);
      }
    }
    
    return null;
  }

  // 策略2: 海外用户 - 直接调用国际API
  console.log('🌐 [海外用户] 直接调用国际地理编码API');
  
  // 优先 OpenStreetMap（支持中文地址）
  try {
    const res = await fetch(
      `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(address)}&format=json&limit=1&accept-language=zh-CN`,
      {
        signal: AbortSignal.timeout(8000),
        headers: { 'User-Agent': 'HuangjiJingshiWeb/1.0' }
      }
    );
    
    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data) && data.length > 0) {
        const first = data[0];
        const lat = parseFloat(first.lat);
        const lon = parseFloat(first.lon);
        if (!isNaN(lat) && !isNaN(lon)) {
          console.log('✅ OpenStreetMap 地理编码成功:', first.display_name);
          return {
            latitude: lat,
            longitude: lon,
            address: first.display_name || address
          };
        }
      }
    }
  } catch (error) {
    console.log('❌ OpenStreetMap 地理编码失败', error);
  }
  
  // 降级: BigDataCloud
  try {
    const res = await fetch(
      `https://api.bigdatacloud.net/data/forward-geocode-client?query=${encodeURIComponent(address)}&localityLanguage=zh`,
      { signal: AbortSignal.timeout(8000) }
    );
    
    if (res.ok) {
      const data = await res.json();
      if (data.results && Array.isArray(data.results) && data.results.length > 0) {
        const first = data.results[0];
        if (first.latitude && first.longitude) {
          console.log('✅ BigDataCloud 地理编码成功:', first.formatted);
          return {
            latitude: first.latitude,
            longitude: first.longitude,
            address: first.formatted || address
          };
        }
      }
    }
  } catch (error) {
    console.log('❌ BigDataCloud 地理编码失败', error);
  }
  
  return null;
}

/**
 * IP地理定位
 */
export async function getIPLocation(): Promise<{
  latitude: number;
  longitude: number;
  city: string;
  region: string;
} | null> {
  const inChina = await detectChinaMainland();
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';

  // 策略1: 中国大陆用户 - 使用后端中转
  if (inChina) {
    console.log('📍 [大陆用户] 使用后端中转进行IP定位');
    
    if (API_BASE) {
      try {
        const res = await fetch(`${API_BASE}/api/geoip`, { 
          signal: AbortSignal.timeout(8000) 
        });
        
        if (res.ok) {
          const data = await res.json();
          console.log('✅ 后端中转IP定位成功:', data.city);
          return {
            latitude: data.latitude,
            longitude: data.longitude,
            city: data.city,
            region: data.region
          };
        }
      } catch (error) {
        console.log('❌ 后端中转IP定位失败', error);
      }
    }
    
    return null;
  }

  // 策略2: 海外用户 - 直接调用国际API
  console.log('🌐 [海外用户] 直接调用国际IP定位API');
  
  // 优先 ipapi.co（海外速度快，数据准确）
  try {
    const res = await fetch('https://ipapi.co/json/', { 
      signal: AbortSignal.timeout(5000) 
    });
    
    if (res.ok) {
      const data = await res.json();
      console.log('✅ ipapi.co 成功:', data.city);
      return {
        latitude: data.latitude,
        longitude: data.longitude,
        city: data.city,
        region: data.region
      };
    }
  } catch (error) {
    console.log('❌ ipapi.co 失败', error);
  }
  
  // 降级: ip-api.com
  try {
    const res = await fetch('http://ip-api.com/json/?lang=zh-CN', { 
      signal: AbortSignal.timeout(5000) 
    });
    
    if (res.ok) {
      const data = await res.json();
      if (data.status === 'success') {
        console.log('✅ ip-api.com 成功:', data.city);
        return {
          latitude: data.lat,
          longitude: data.lon,
          city: data.city,
          region: data.regionName
        };
      }
    }
  } catch (error) {
    console.log('❌ ip-api.com 失败', error);
  }
  
  return null;
}

