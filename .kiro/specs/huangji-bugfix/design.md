# Design Document

## Overview

本设计文档描述皇极经世「会/运/世/旬 + 卦象」计算系统的 Bug 修复方案。修复分为 P0（核心计算错误）和 P1（时区口径问题）两个优先级。

核心修复目标：
1. **acc ↔ year 对称转换**：确保 BCE/CE 边界正确，无公元 0 年
2. **Euclid 除法分段**：确保负数年份分段正确
3. **卦象映射修正**：修复否/泰等卦象的 upper/lower 对调问题
4. **时区统一**：前端显式时区，后端统一 hj_year 入口

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (React)                      │
│  ┌─────────────────┐    ┌─────────────────┐                 │
│  │   BaziForm      │    │   Timeline      │                 │
│  │ (时区选择+转换)  │    │  (区间展示)     │                 │
│  └────────┬────────┘    └────────┬────────┘                 │
│           │ UTC ISO + tzOffsetMinutes                        │
└───────────┼──────────────────────┼──────────────────────────┘
            │                      │
            ▼                      ▼
┌─────────────────────────────────────────────────────────────┐
│                     Backend (Rust/Axum)                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              datetime_to_hj_year()                   │    │
│  │  UTC + tz_offset + lon → rule_dt → hj_year          │    │
│  └────────────────────────┬────────────────────────────┘    │
│                           │                                  │
│  ┌────────────────────────▼────────────────────────────┐    │
│  │              huangji_core/algorithm.rs               │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │    │
│  │  │year_to_acc() │  │acc_to_year() │  │get_hj_info│  │    │
│  │  │(对称转换)    │  │(对称转换)    │  │(Euclid)   │  │    │
│  │  └──────────────┘  └──────────────┘  └───────────┘  │    │
│  │  ┌──────────────────────────────────────────────┐   │    │
│  │  │         HEXAGRAM_TABLE (修正后)               │   │    │
│  │  │  get_hexagram_name() / get_hexagram_struct() │   │    │
│  │  └──────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. acc ↔ year 转换模块

```rust
// 常量定义
const ACC_BC1: i32 = 67017;  // 1 BC 对应的 acc 值

/// 公元年 → 累积年（无 0 年）
/// year=-1 (1BC) → 67017
/// year=1  (1AD) → 67018
pub fn year_to_acc(year: i32) -> i32;

/// 累积年 → 公元年（无 0 年）
/// acc=67017 → year=-1 (1BC)
/// acc=67018 → year=1  (1AD)
pub fn acc_to_year(acc: i32) -> i32;
```

### 2. 分段计算模块

```rust
/// 分段核心函数：以 acc 为唯一输入域
/// 所有分段计算在 acc 域完成，避免 hj_year 口径污染
/// t = acc - EPOCH_ACC，然后 div_euclid/rem_euclid 做所有分段
fn segment(acc: i32) -> Segments;

/// 获取皇极经世信息（会/运/世/旬）
/// 内部先转 acc，再调用 segment()，最后展示层转回 hj_year label
pub fn get_hj_info(hj_year: i32) -> HuangjiInfo;

/// 获取时间线数据（包含各层级列表）
pub fn get_timeline_info(hj_year: i32) -> TimelineData;
```

**关键设计原则**：
- 分段核心逻辑只操作 acc 域：`t = acc - EPOCH_ACC`
- 只在展示层把 `start_acc/end_acc` 转成 hj_year 的 label
- 避免在分段里使用 `hj_year - EPOCH_YEAR` 这种差值，防止"无 0 年"反复掺沙子

### 3. 卦象映射模块

```rust
/// 八卦编码：0=坤, 1=震, 2=坎, 3=兑, 4=艮, 5=离, 6=巽, 7=乾
/// (upper, lower) → 卦名
pub fn get_hexagram_name(upper: u8, lower: u8) -> String;

/// 卦名 → (upper, lower)
pub fn get_hexagram_struct(name: &str) -> (u8, u8);
```

### 4. 前端时区转换

```typescript
interface BaziParams {
  datetime: string;      // UTC ISO8601
  tzOffsetMinutes: number; // 时区偏移（分钟）
  lat: number;
  lon: number;
  gender: 'male' | 'female' | 'other';
}

// 本地时间 + 时区 → UTC ISO
function convertLocalToUTC(localDateTime: string, tzOffsetMinutes: number): string;
```

### 5. 后端时间规则入口

```rust
/// UTC + 时区偏移 + 经度 → 规则时间
/// 真太阳时校正：以所选时区中央经线为基准
/// 
/// 注意：必须使用浮点除法，支持 +5:30（印度）等非整小时时区
/// let tz_offset_hours: f64 = tz_offset_minutes as f64 / 60.0;
/// let central_meridian: f64 = 15.0 * tz_offset_hours;
/// let delta_minutes: f64 = 4.0 * (lon - central_meridian);
fn to_rule_datetime(
    utc: DateTime<Utc>, 
    tz_offset_minutes: i32, 
    lon: f64
) -> DateTime<FixedOffset>;

/// 规则时间 → 经世年（支持公历岁首/立春岁首）
fn datetime_to_hj_year(
    rule_dt: DateTime<FixedOffset>,
    use_lichun: bool
) -> i32;
```

**tzOffsetMinutes 符号约定**（必须在代码中注释）：
```rust
// tzOffsetMinutes: 东为正、西为负
// UTC+8 => +480
// UTC+9 => +540
// UTC-5 => -300
// 注意：与 JS Date.getTimezoneOffset() 符号相反！
```

### 6. 距下一周期计算

```rust
/// 计算距下一个周期边界的年数（使用 acc 差值，保证非负）
fn years_to_next_boundary(current_acc: i32, next_boundary_acc: i32) -> i32 {
    (next_boundary_acc - current_acc).max(0)
}
```

## Data Models

### PeriodInfo（周期信息）

```rust
pub struct PeriodInfo {
    pub name: String,       // 名称（如 "子会"、"己运"）
    pub start_year: i32,    // 起始年（展示用，hj_year）
    pub end_year: i32,      // 结束年（闭区间展示用）
    pub start_acc: i32,     // 起始累积年（内部计算用）
    pub end_acc: i32,       // 结束累积年（半开区间）
    pub index: u32,         // 当前索引（1-based）
    pub max_index: u32,     // 最大索引
}
```

### HuangjiInfo（皇极信息）

```rust
pub struct HuangjiInfo {
    pub yuan: PeriodInfo,
    pub hui: PeriodInfo,
    pub yun: PeriodInfo,
    pub shi: PeriodInfo,
    pub xun: PeriodInfo,
    pub year_gua: String,
    pub hexagram: HexagramInfo,  // 新增：卦象详情
}

pub struct HexagramInfo {
    pub upper: u8,          // 上卦编码
    pub lower: u8,          // 下卦编码
    pub name: String,       // 卦名
    pub upper_name: String, // 上卦名（如 "乾"）
    pub lower_name: String, // 下卦名（如 "坤"）
}
```

### HEXAGRAM_TABLE（修正后的卦象表）

**关键修正：否/泰 的卦名需要对调**

当前代码 `algorithm.rs` 中的错误：
```rust
// 乾宫八卦（当前代码 - 错误）
(7, 0, "泰"),  // 错：乾上坤下应该是"否"

// 坤宫八卦（当前代码 - 错误）
(0, 7, "否"),  // 错：坤上乾下应该是"泰"
```

修复方案（两刀都要落下）：
1. **修正卦象表中的卦名**：
```rust
// 乾宫八卦（修正后）
(7, 0, "否"),  // 正确：天地否 = 乾上坤下

// 坤宫八卦（修正后）
(0, 7, "泰"),  // 正确：地天泰 = 坤上乾下
```

2. **确保解构顺序正确**：
```rust
// 正确的解构方式
for &(u, l, name) in &HEXAGRAM_TABLE {
    if u == upper && l == lower { ... }
}
// 不要写成 for (l, u, n) - 这会导致上下卦对调！
```

完整的八卦编码映射：
| 编码 | 卦名 | 二进制 | 爻象 |
|------|------|--------|------|
| 0    | 坤   | 000    | ☷    |
| 1    | 震   | 001    | ☳    |
| 2    | 坎   | 010    | ☵    |
| 3    | 兑   | 011    | ☱    |
| 4    | 艮   | 100    | ☶    |
| 5    | 离   | 101    | ☲    |
| 6    | 巽   | 110    | ☴    |
| 7    | 乾   | 111    | ☰    |



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: acc ↔ year Round-trip 一致性

*For any* 非零整数年份 year（year ≠ 0），执行 `acc_to_year(year_to_acc(year))` 应返回原始 year 值。

**Validates: Requirements 1.5**

### Property 2: 区间长度恒定（acc 域）

*For any* 年份（包括 BCE），计算得到的周期区间长度在 acc 域应恒定：
- 会：`hui_end_acc - hui_start_acc == 10800`
- 运：`yun_end_acc - yun_start_acc == 360`
- 世：`shi_end_acc - shi_start_acc == 30`
- 旬：`xun_end_acc - xun_start_acc == 10`

**Validates: Requirements 2.5, 2.6, 2.7, 2.8**

### Property 3: 半开区间包含性

*For any* acc 值，计算得到的区间应满足 `segment_start_acc <= acc < segment_end_acc`，即当前 acc 值应落在其所属区间的半开区间内。这是一个关键的属性测试，能抓住 div/mod 口径错误、end inclusive 写错等问题。

**Validates: Requirements 2.9, 7.5**

### Property 4: 分段单调连续性（边界跃迁）

*For any* 连续的 acc 值对 (acc, acc+1)：
- 若两者在同一段：节点索引不变
- 若跨边界：节点索引恰好 +1
- 不会出现跳段（+2 或更多）或回退（-1）

**Validates: Requirements 2.10, 7.6**

### Property 5: 卦象映射双向一致性

*For any* 有效的卦名 name（64卦之一），执行 `get_hexagram_name(get_hexagram_struct(name))` 应返回原始 name。
*For any* 有效的 (upper, lower) 组合（0-7 范围），执行 `get_hexagram_struct(get_hexagram_name(upper, lower))` 应返回原始 (upper, lower)。

**Validates: Requirements 3.9, 3.10**

### Property 6: 时区转换与浏览器无关

*For any* 本地时间字符串和用户选择的时区偏移，转换得到的 UTC 时间应与浏览器当前时区无关。即：相同的输入（本地时间 + 选择的时区）在任何浏览器时区下应产生相同的 UTC 输出。

**Validates: Requirements 4.5**

### Property 7: 距下一周期非负

*For any* 年份，计算得到的"距下一世"和"距下一旬"的年数应为非负整数（>= 0）。

**Validates: Requirements 6.3, 6.4**

## Error Handling

### 1. 无效年份处理

- **year = 0**：由于历史纪年无公元 0 年，`year_to_acc(0)` 应返回错误
- 推荐实现：使用 Result 类型
  ```rust
  pub fn year_to_acc(year: i32) -> Result<i32, &'static str> {
      if year == 0 {
          return Err("公元0年不存在");
      }
      // ... 正常转换逻辑
  }
  ```
- API 层：捕获错误并返回 400 Bad Request
- 注意：不推荐使用 `assert!`，因为会在 release 模式下导致进程 panic

### 2. 无效卦象编码

- **upper/lower > 7**：超出八卦编码范围
- 处理：`get_hexagram_name` 返回 "未知"，`get_hexagram_struct` 返回 (0, 0)

### 3. 时区偏移范围

- **tzOffsetMinutes** 应在 -720 到 +840 范围内（UTC-12 到 UTC+14）
- 超出范围应返回错误或使用默认值 480（UTC+8）

### 4. 日期解析失败

- 前端：显示错误提示，不发送请求
- 后端：返回 400 Bad Request，包含错误信息

## Testing Strategy

### 双重测试方法

本项目采用单元测试 + 属性测试的双重策略：

1. **单元测试**：验证具体的边界条件和关键示例
2. **属性测试**：验证普遍性质，覆盖大量随机输入

### 属性测试框架

使用 Rust 的 `proptest` 库进行属性测试：

```toml
[dev-dependencies]
proptest = "1.4"
```

### 测试配置

- 每个属性测试运行 **100+ 次迭代**
- 测试范围覆盖 BCE 和 CE 年份
- 特别关注边界：-1, 1, 周期边界年份

### 测试文件结构

```
huangji_core/
├── src/
│   └── algorithm.rs
└── tests/
    ├── acc_year_tests.rs      # acc/year 转换测试
    ├── segment_tests.rs       # 分段计算测试
    ├── hexagram_tests.rs      # 卦象映射测试
    └── property_tests.rs      # 属性测试（proptest）
```

### 关键测试用例

#### 1. acc/year 对称性（黄金用例）

```rust
#[test]
fn test_year_acc_golden_cases() {
    assert_eq!(year_to_acc(-1), 67017);  // 1 BC
    assert_eq!(year_to_acc(1), 67018);   // 1 AD
    assert_eq!(acc_to_year(67017), -1);
    assert_eq!(acc_to_year(67018), 1);
    assert_eq!(year_to_acc(1) - year_to_acc(-1), 1);  // 连续性
}
```

#### 2. 卦象映射（关键修复验证）

```rust
#[test]
fn test_hexagram_pi_tai() {
    // 否 = 天地否 = 乾上坤下
    assert_eq!(get_hexagram_name(7, 0), "否");
    assert_eq!(get_hexagram_struct("否"), (7, 0));
    
    // 泰 = 地天泰 = 坤上乾下
    assert_eq!(get_hexagram_name(0, 7), "泰");
    assert_eq!(get_hexagram_struct("泰"), (0, 7));
}
```

#### 3. 边界年份（2044 问题）

```rust
#[test]
fn test_boundary_2044() {
    let info_2043 = get_hj_info(2043);
    let info_2044 = get_hj_info(2044);
    
    assert_eq!(info_2044.shi.start_year, 2044);
    assert_eq!(info_2043.shi.end_year, 2043);
    assert!(info_2044.shi.index > info_2043.shi.index || 
            info_2044.shi.start_year > info_2043.shi.start_year);
}
```

### 属性测试示例

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Feature: huangji-bugfix, Property 1: acc ↔ year Round-trip 一致性**
    #[test]
    fn prop_year_acc_roundtrip(year in -10000i32..10000i32) {
        prop_assume!(year != 0);  // 排除公元0年
        let acc = year_to_acc(year);
        let year2 = acc_to_year(acc);
        prop_assert_eq!(year, year2);
    }
    
    /// **Feature: huangji-bugfix, Property 2: 区间长度恒定**
    #[test]
    fn prop_segment_length_constant(year in -10000i32..10000i32) {
        prop_assume!(year != 0);
        let info = get_hj_info(year);
        let hui_len = year_to_acc(info.hui.end_year) - year_to_acc(info.hui.start_year) + 1;
        prop_assert_eq!(hui_len, 10800);
    }
    
    /// **Feature: huangji-bugfix, Property 5: 卦象映射双向一致性**
    #[test]
    fn prop_hexagram_bijection(upper in 0u8..8u8, lower in 0u8..8u8) {
        let name = get_hexagram_name(upper, lower);
        if name != "未知" {
            let (u2, l2) = get_hexagram_struct(&name);
            prop_assert_eq!((upper, lower), (u2, l2));
        }
    }
}
```
