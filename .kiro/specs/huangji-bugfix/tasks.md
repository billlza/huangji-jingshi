# Implementation Plan

- [x] 0. 添加 proptest 依赖和测试文件结构
  - [x] 0.1 在 Cargo.toml 添加 proptest 依赖
    - `proptest = "1.4"` 在 dev-dependencies
    - _Requirements: 7.1, 7.2, 7.3, 7.5, 7.6, 7.7_
  - [x] 0.2 创建属性测试文件结构
    - 创建 `tests/property_tests.rs`
    - 配置 100+ 迭代次数
    - _Requirements: 7.1, 7.2, 7.3, 7.5, 7.6, 7.7_

- [x] 1. P0: 修复 acc ↔ year 对称转换
  - [x] 1.1 修改 year_to_acc 返回 Result 类型
    - 将 `pub fn year_to_acc(year: i32) -> i32` 改为 `pub fn year_to_acc(year: i32) -> Result<i32, &'static str>`
    - year=0 时返回 `Err("公元0年不存在")`
    - 更新所有调用点处理 Result
    - _Requirements: 1.7_
  - [x] 1.2 编写 acc/year 转换属性测试
    - **Property 1: acc ↔ year Round-trip 一致性**
    - **Validates: Requirements 1.5**
  - [x] 1.3 编写 acc/year 黄金用例单元测试
    - 测试 year=-1 → acc=67017, year=1 → acc=67018
    - 测试 acc 连续性（差值为 1）
    - 测试 year=0 返回 Err
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.6, 1.7_

- [x] 2. P0: 修复卦象表 upper/lower 映射
  - [x] 2.1 修正 HEXAGRAM_TABLE 中否/泰的卦名
    - 将 `(7, 0, "泰")` 改为 `(7, 0, "否")`（乾宫八卦）
    - 将 `(0, 7, "否")` 改为 `(0, 7, "泰")`（坤宫八卦）
    - _Requirements: 3.3, 3.4, 3.7, 3.8_
  - [x] 2.2 验证并修复 get_hexagram_name 和 get_hexagram_struct 解构顺序
    - 确保两个函数都使用 `for &(u, l, name)` 而非 `for (l, u, n)`
    - 确保两个函数使用同一张表、同一套 (u,l,name) 口径
    - _Requirements: 3.9, 3.10_
  - [x] 2.3 编写卦象映射属性测试
    - **Property 5: 卦象映射双向一致性**
    - **Validates: Requirements 3.9, 3.10**
  - [x] 2.4 编写卦象黄金用例单元测试
    - 测试乾(7,7)、坤(0,0)、否(7,0)、泰(0,7)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8_

- [x] 3. Checkpoint - 确保 P0 核心修复测试通过
  - CI 绿灯后进入下一阶段；若失败，优先定位失败测试对应的 Requirement 编号并修复。

- [x] 4. P0: 验证并优化 Euclid 除法分段计算
  - [x] 4.1 重构分段计算以 acc 为唯一输入域
    - 确认 `get_hj_info` 和 `get_timeline_info` 中使用 `t = acc - EPOCH_ACC`
    - 确保所有 div_euclid/rem_euclid 操作在 acc 域完成
    - 只在展示层把 start_acc/end_acc 转成 hj_year label
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - [x] 4.2 编写区间长度恒定属性测试
    - **Property 2: 区间长度恒定（acc 域）**
    - **Validates: Requirements 2.5, 2.6, 2.7, 2.8**
  - [x] 4.3 编写半开区间包含性属性测试
    - **Property 3: 半开区间包含性**
    - **Validates: Requirements 2.9**
  - [x] 4.4 编写边界跃迁属性测试
    - **Property 4: 分段单调连续性（边界跃迁）**
    - **Validates: Requirements 2.10**

- [x] 5. P0: 修复边界年份问题
  - [x] 5.1 实现 years_to_next_boundary 函数
    - 使用 acc 差值计算：`max(0, next_boundary_acc - current_acc)`
    - _Requirements: 6.3, 6.4_
  - [x] 5.2 编写距下一周期非负属性测试
    - **Property 7: 距下一周期非负**
    - **Validates: Requirements 6.3, 6.4**
  - [x] 5.3 编写边界年份单元测试（公历岁首模式）
    - 测试条件：mode=GregorianNewYear, true_solar_time=false, tzOffsetMinutes=+480
    - 测试 2044-01-01 属于新世（2044-2073）
    - 测试 2043-12-31 属于上一世（2014-2043）
    - _Requirements: 6.1, 6.2_

- [x] 6. Checkpoint - 确保所有 P0 修复测试通过
  - CI 绿灯后进入下一阶段；若失败，优先定位失败测试对应的 Requirement 编号并修复。

- [x] 7. P1: 前端时区处理修复
  - [x] 7.1 修改 convertLocalToUTC 函数
    - 显式使用用户选择的时区偏移，不依赖浏览器时区
    - 使用 `Date.UTC()` 构造 UTC 时间
    - 添加符号约定注释：`// tzOffsetMinutes: 东为正 UTC+8=+480, 西为负 UTC-5=-300`
    - _Requirements: 4.1, 4.2, 4.4, 4.5_
  - [x] 7.2 确保 payload 包含 tzOffsetMinutes
    - 在 BaziParams 接口中添加 tzOffsetMinutes
    - 在请求中传递时区偏移
    - 添加注释警告：不要使用 getTimezoneOffset() 直接赋值（符号相反）
    - _Requirements: 4.3_
  - [x] 7.3 编写时区转换单元测试
    - 测试 UTC+8 (+480) 输入 "2025-12-18T21:48" → "2025-12-18T13:48:00.000Z"
    - 测试 UTC+9 (+540) 输入 "2025-12-18T21:48" → "2025-12-18T12:48:00.000Z"
    - _Requirements: 4.1, 4.2_

- [x] 8. P1: 后端时间规则统一
  - [x] 8.1 实现 to_rule_datetime 函数
    - UTC + 时区偏移 → 本地时间
    - 真太阳时校正（使用浮点除法支持非整小时时区如 +5:30）：
      - `tz_offset_hours = tz_offset_minutes as f64 / 60.0`
      - `central_meridian = 15.0 * tz_offset_hours`
      - `delta_minutes = 4.0 * (lon - central_meridian)`
    - _Requirements: 5.1, 5.3_
  - [x] 8.2 实现 datetime_to_hj_year 函数
    - 支持公历岁首模式（默认）
    - 预留立春岁首模式接口
    - _Requirements: 5.2, 5.4_
  - [x] 8.3 修改 API 端点使用统一入口
    - 确保 `/api/timeline` 和 `/api/bazi` 使用 hj_year 而非 UTC year
    - year=0 时返回 400 Bad Request
    - _Requirements: 5.2_

- [x] 9. Checkpoint - 确保所有 P1 修复测试通过
  - CI 绿灯后进入下一阶段；若失败，优先定位失败测试对应的 Requirement 编号并修复。

- [x] 10. Final Checkpoint - 确保所有测试通过
  - CI 绿灯；若失败，优先定位失败测试对应的 Requirement 编号并修复。
