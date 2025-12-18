# Requirements Document

## Introduction

本文档定义了皇极经世「会/运/世/旬 + 卦象」计算系统的 Bug 修复需求。当前系统存在三类核心问题：

1. **P0 - acc ↔ 公元年转换不对称**：BCE/CE 边界错位，区间长度异常（如 10801 年）
2. **P0 - Rust 负数整除/取模口径错误**：未使用 Euclid 除法，导致 BCE 分段全错
3. **P0 - 六十四卦表 upper/lower 解构对调**：上卦下卦整体反转
4. **P1 - 前后端时区口径不统一**：前端时间解析依赖浏览器时区，后端直接用 UTC 年份

## Glossary

- **acc (累积年)**：连续整数轴，用于内部计算，1 BC = 67017, 1 AD = 67018
- **hj_year (经世年)**：历史纪年，无公元 0 年，..., -2, -1, 1, 2, ...。分段计算必须使用 hj_year，禁止直接使用 UTC year
- **会**：10800 年周期
- **运**：360 年周期
- **世**：30 年周期
- **旬**：10 年周期
- **Euclid 除法**：数学上的欧几里得除法，对负数使用 floor 而非向 0 截断
- **半开区间**：[start, end)，start 属于本段，end 属于下一段
- **upper/lower**：卦象的上卦和下卦
- **trigram_code (八卦编码)**：0=坤, 1=震, 2=坎, 3=兑, 4=艮, 5=离, 6=巽, 7=乾（本项目采用的先天八卦二进制编码）
- **真太阳时**：根据经度校正后的本地时间，以所选时区中央经线为基准（central_meridian = 15° × tzOffsetHours）
- **公历岁首模式**：以公历 1 月 1 日为年份切换点
- **立春岁首模式**：以立春节气时刻为年份切换点
- **fixed-offset 时区**：固定偏移量时区（如 UTC+8），不含夏令时变化
- **tzOffsetMinutes**：时区偏移（分钟），东为正。UTC+8 = +480，UTC-5 = -300。注意：与 JS `Date.getTimezoneOffset()` 符号相反

## Requirements

### Requirement 1: acc ↔ year 对称转换

**User Story:** As a 开发者, I want acc 与 year 的转换函数完全对称, so that BCE/CE 边界不会错位且区间长度恒定。

#### Acceptance Criteria

1. WHEN year_to_acc 接收 year=-1 (1 BC) THEN the System SHALL 返回 acc=67017
2. WHEN year_to_acc 接收 year=1 (1 AD) THEN the System SHALL 返回 acc=67018
3. WHEN acc_to_year 接收 acc=67017 THEN the System SHALL 返回 year=-1 (1 BC)
4. WHEN acc_to_year 接收 acc=67018 THEN the System SHALL 返回 year=1 (1 AD)
5. WHEN 对任意 hj_year 执行 acc_to_year(year_to_acc(hj_year)) THEN the System SHALL 返回原始 hj_year 值（round-trip 一致性）
6. WHEN year=-1 和 year=1 转换为 acc THEN the System SHALL 确保 acc 值连续（差值为 1），无公元 0 年
7. WHEN year_to_acc 接收 year=0 THEN the System SHALL 返回 Result::Err（公元 0 年不存在），API 层应返回 400 错误

### Requirement 2: Euclid 除法分段计算

**User Story:** As a 用户, I want 会/运/世/旬分段在 BCE 区间也正确, so that 历史年份的周期计算不会跳段或错位。

注：分段函数入参必须使用 acc 或 hj_year（历史纪年），禁止直接使用 UTC year。

#### Acceptance Criteria

1. WHEN 计算任意 hj_year 的会区间 THEN the System SHALL 使用 div_euclid 和 rem_euclid 进行分段
2. WHEN 计算任意 hj_year 的运区间 THEN the System SHALL 使用 div_euclid 和 rem_euclid 进行分段
3. WHEN 计算任意 hj_year 的世区间 THEN the System SHALL 使用 div_euclid 和 rem_euclid 进行分段
4. WHEN 计算任意 hj_year 的旬区间 THEN the System SHALL 使用 div_euclid 和 rem_euclid 进行分段
5. WHEN 计算任意 hj_year（包括 BCE）的会区间 THEN the System SHALL 满足 (hui_end_acc - hui_start_acc) == 10800（在 acc 域计算长度）
6. WHEN 计算任意 hj_year（包括 BCE）的运区间 THEN the System SHALL 满足 (yun_end_acc - yun_start_acc) == 360（在 acc 域计算长度）
7. WHEN 计算任意 hj_year（包括 BCE）的世区间 THEN the System SHALL 满足 (shi_end_acc - shi_start_acc) == 30（在 acc 域计算长度）
8. WHEN 计算任意 hj_year（包括 BCE）的旬区间 THEN the System SHALL 满足 (xun_end_acc - xun_start_acc) == 10（在 acc 域计算长度）
9. WHEN 对任意 acc 值 THEN the System SHALL 满足 segment_start_acc <= acc < segment_end_acc（半开区间包含性）
10. WHEN acc 值增加 1 THEN the System SHALL 保证节点索引要么不变，要么在边界处恰好 +1（单调连续性，不跳段）

### Requirement 3: 卦象 upper/lower 映射正确性

**User Story:** As a 用户, I want 卦象的上卦下卦显示正确, so that 卦名与卦象结构一一对应。

#### Acceptance Criteria

注：根据《周易》定义，否=天地否=乾上坤下，泰=地天泰=坤上乾下。trigram_code: 0=坤, 7=乾。

1. WHEN get_hexagram_name 接收 (upper=7, lower=7) THEN the System SHALL 返回 "乾"（乾为天，上下皆乾）
2. WHEN get_hexagram_name 接收 (upper=0, lower=0) THEN the System SHALL 返回 "坤"（坤为地，上下皆坤）
3. WHEN get_hexagram_name 接收 (upper=7, lower=0) THEN the System SHALL 返回 "否"（天地否，乾上坤下）
4. WHEN get_hexagram_name 接收 (upper=0, lower=7) THEN the System SHALL 返回 "泰"（地天泰，坤上乾下）
5. WHEN get_hexagram_struct 接收 "乾" THEN the System SHALL 返回 (upper=7, lower=7)
6. WHEN get_hexagram_struct 接收 "坤" THEN the System SHALL 返回 (upper=0, lower=0)
7. WHEN get_hexagram_struct 接收 "否" THEN the System SHALL 返回 (upper=7, lower=0)
8. WHEN get_hexagram_struct 接收 "泰" THEN the System SHALL 返回 (upper=0, lower=7)
9. WHEN 对任意卦名执行 get_hexagram_name(get_hexagram_struct(name)) THEN the System SHALL 返回原始卦名（round-trip 一致性）
10. WHEN 对任意 (upper, lower) 执行 get_hexagram_struct(get_hexagram_name(upper, lower)) THEN the System SHALL 返回原始 (upper, lower)（round-trip 一致性）

### Requirement 4: 前端时区处理

**User Story:** As a 用户, I want 选择北京时间后输入的时间被正确解析为北京时间, so that 不会因浏览器时区不同而产生偏差。

注：当前只保证 fixed-offset 时区（如 UTC+8），不处理夏令时（DST）。未来如需支持欧美时区，应传 timeZoneId（IANA 格式）并由后端时区库处理。

#### Acceptance Criteria

1. WHEN 用户选择 UTC+8 时区并输入 "2025-12-18T21:48" THEN the System SHALL 生成 UTC 时间 "2025-12-18T13:48:00.000Z"
2. WHEN 用户选择 UTC+9 时区并输入 "2025-12-18T21:48" THEN the System SHALL 生成 UTC 时间 "2025-12-18T12:48:00.000Z"
3. WHEN 前端发送请求 THEN the System SHALL 在 payload 中包含 tzOffsetMinutes 参数
4. WHEN 解析本地时间字符串 THEN the System SHALL 显式使用用户选择的时区偏移，而非依赖浏览器时区
5. WHEN 用户在任意浏览器时区下选择 UTC+8 并输入相同时间 THEN the System SHALL 生成相同的 UTC 时间（与浏览器时区无关）

### Requirement 5: 后端时间规则统一

**User Story:** As a 系统架构师, I want 后端使用统一的 hj_year 计算入口, so that 所有分段计算基于相同的时间规则。

#### Acceptance Criteria

1. WHEN 后端接收 UTC 时间和时区偏移 THEN the System SHALL 先转换为规则时间（rule_dt）
2. WHEN 计算会/运/世/旬分段 THEN the System SHALL 使用 hj_year 而非 UTC year
3. WHEN 支持真太阳时模式 THEN the System SHALL 根据经度进行时间校正，以所选时区中央经线为基准（central_meridian = 15° × tzOffsetHours，delta_minutes = 4 × (lon - central_meridian)）
4. WHEN 支持立春岁首模式 THEN the System SHALL 根据立春时刻判断年份归属

### Requirement 6: 边界年份正确性

**User Story:** As a 用户, I want 边界年份（如 2044-01-01）显示正确的世/旬归属, so that 不会出现 "Next Shi -1 yrs" 这种异常。

#### Acceptance Criteria（公历岁首模式）

1. WHEN 使用公历岁首模式查询 2044-01-01 THEN the System SHALL 显示该日期属于新的世（2044-2073）
2. WHEN 使用公历岁首模式查询 2043-12-31 THEN the System SHALL 显示该日期属于上一个世（2014-2043）
3. WHEN 计算 "距下一世" 的年数 THEN the System SHALL 使用 acc 差值计算：years_to_next = max(0, next_boundary_acc - current_acc)，返回非负整数
4. WHEN 计算 "距下一旬" 的年数 THEN the System SHALL 使用 acc 差值计算：years_to_next = max(0, next_boundary_acc - current_acc)，返回非负整数

#### Acceptance Criteria（立春岁首模式）

5. WHEN 使用立春岁首模式查询 2044-01-01（立春前） THEN the System SHALL 显示该日期仍属于上一经世年（2043），归属上一个世
6. WHEN 使用立春岁首模式查询 2044 年立春时刻之后 THEN the System SHALL 显示该日期属于新的经世年（2044），归属新的世
7. WHEN 岁首模式切换 THEN the System SHALL 根据当前模式正确计算年份归属

### Requirement 7: 测试覆盖

**User Story:** As a 开发者, I want 有完整的单元测试和属性测试, so that 修复后的行为被固化，避免未来回归。

#### Acceptance Criteria

1. WHEN 运行 acc/year 转换测试 THEN the System SHALL 验证 round-trip 一致性（Property 1）
2. WHEN 运行分段长度测试 THEN the System SHALL 验证所有周期长度恒定（在 acc 域）（Property 2）
3. WHEN 运行卦象映射测试 THEN the System SHALL 验证 name ↔ (upper, lower) 双向一致（Property 5）
4. WHEN 运行 BCE 连续性测试 THEN the System SHALL 验证 -2, -1, 1 的 acc 值连续递增
5. WHEN 运行包含性属性测试 THEN the System SHALL 对随机 acc 验证 segment_start_acc <= acc < segment_end_acc（Property 3）
6. WHEN 运行边界跃迁属性测试 THEN the System SHALL 对随机 acc 验证：若 acc 与 acc+1 在同一段则 index 不变，若跨边界则 index 恰好 +1（Property 4）
7. WHEN 运行距下一周期测试 THEN the System SHALL 验证返回值非负（Property 7）
