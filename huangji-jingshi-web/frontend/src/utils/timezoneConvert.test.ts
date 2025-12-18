/**
 * 时区转换单元测试
 * 
 * **Validates: Requirements 4.1, 4.2**
 */

import { describe, it, expect } from 'vitest';
import { convertLocalToUTC, getTimezoneOffsetMinutes } from './timezoneConvert';

describe('convertLocalToUTC', () => {
  /**
   * 测试 UTC+8 (+480) 输入 "2025-12-18T21:48" → "2025-12-18T13:48:00.000Z"
   * **Validates: Requirements 4.1**
   */
  it('should convert UTC+8 local time to correct UTC time', () => {
    const localDateTime = '2025-12-18T21:48';
    const tzOffsetMinutes = 480; // UTC+8
    
    const result = convertLocalToUTC(localDateTime, tzOffsetMinutes);
    
    expect(result).toBe('2025-12-18T13:48:00.000Z');
  });

  /**
   * 测试 UTC+9 (+540) 输入 "2025-12-18T21:48" → "2025-12-18T12:48:00.000Z"
   * **Validates: Requirements 4.2**
   */
  it('should convert UTC+9 local time to correct UTC time', () => {
    const localDateTime = '2025-12-18T21:48';
    const tzOffsetMinutes = 540; // UTC+9
    
    const result = convertLocalToUTC(localDateTime, tzOffsetMinutes);
    
    expect(result).toBe('2025-12-18T12:48:00.000Z');
  });

  /**
   * 测试 UTC-5 (-300) 西半球时区
   */
  it('should convert UTC-5 local time to correct UTC time', () => {
    const localDateTime = '2025-12-18T08:00';
    const tzOffsetMinutes = -300; // UTC-5
    
    const result = convertLocalToUTC(localDateTime, tzOffsetMinutes);
    
    // 08:00 - (-300min) = 08:00 + 5h = 13:00 UTC
    expect(result).toBe('2025-12-18T13:00:00.000Z');
  });

  /**
   * 测试日期跨越（本地时间转换后跨到下一天）
   */
  it('should handle date crossing when converting to UTC', () => {
    const localDateTime = '2025-12-18T02:00';
    const tzOffsetMinutes = 480; // UTC+8
    
    const result = convertLocalToUTC(localDateTime, tzOffsetMinutes);
    
    // 02:00 - 8h = -6h = 前一天 18:00 UTC
    expect(result).toBe('2025-12-17T18:00:00.000Z');
  });

  /**
   * 测试日期跨越（西半球时区，本地时间转换后跨到下一天）
   */
  it('should handle date crossing for western timezones', () => {
    const localDateTime = '2025-12-18T20:00';
    const tzOffsetMinutes = -300; // UTC-5
    
    const result = convertLocalToUTC(localDateTime, tzOffsetMinutes);
    
    // 20:00 - (-5h) = 20:00 + 5h = 25:00 = 下一天 01:00 UTC
    expect(result).toBe('2025-12-19T01:00:00.000Z');
  });

  /**
   * 测试 UTC+0 时区
   */
  it('should handle UTC+0 timezone correctly', () => {
    const localDateTime = '2025-12-18T15:30';
    const tzOffsetMinutes = 0; // UTC+0
    
    const result = convertLocalToUTC(localDateTime, tzOffsetMinutes);
    
    expect(result).toBe('2025-12-18T15:30:00.000Z');
  });

  /**
   * 测试空输入返回当前时间
   */
  it('should return current time for empty input', () => {
    const result = convertLocalToUTC('', 480);
    
    // 应该返回一个有效的 ISO 字符串
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/);
  });
});

describe('getTimezoneOffsetMinutes', () => {
  it('should return correct offset for Asia/Shanghai', () => {
    expect(getTimezoneOffsetMinutes('Asia/Shanghai')).toBe(480);
  });

  it('should return correct offset for Asia/Tokyo', () => {
    expect(getTimezoneOffsetMinutes('Asia/Tokyo')).toBe(540);
  });

  it('should return correct offset for America/New_York', () => {
    expect(getTimezoneOffsetMinutes('America/New_York')).toBe(-300);
  });

  it('should return correct offset for America/Los_Angeles', () => {
    expect(getTimezoneOffsetMinutes('America/Los_Angeles')).toBe(-480);
  });

  it('should return correct offset for Europe/London', () => {
    expect(getTimezoneOffsetMinutes('Europe/London')).toBe(0);
  });

  it('should return default offset (480) for unknown timezone', () => {
    expect(getTimezoneOffsetMinutes('Unknown/Timezone')).toBe(480);
  });
});
