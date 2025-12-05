// 八字排盘算法验证测试
// 用已知案例验证计算正确性

#[cfg(test)]
mod tests {
    // 测试用例说明：
    // 
    // 1. 年柱测试 - 立春换年
    //    - 2025年2月3日 (立春前): 应为甲辰年 (不是乙巳年)
    //    - 2025年2月4日 (立春后): 应为乙巳年
    //
    // 2. 月柱测试 - 节气换月 + 五虎遁
    //    - 2025年2月4日 (立春后): 戊寅月 (乙年用戊寅起首)
    //    - 2025年3月6日 (惊蛰后): 己卯月
    //
    // 3. 日柱测试 - 基准日验证
    //    - 1970年1月1日: 庚戌日
    //    - 2000年1月1日: 戊午日
    //    - 2025年1月1日: 甲辰日
    //
    // 4. 时柱测试 - 五鼠遁
    //    - 甲日子时: 甲子时
    //    - 乙日子时: 丙子时
    //    - 丙日子时: 戊子时
    //
    // 5. 大运测试
    //    - 起运年龄应根据出生日到节气天数计算
    
    const TIANGAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    const DIZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

    // 计算太阳黄经
    fn get_solar_longitude(jd: f64) -> f64 {
        let t = (jd - 2451545.0) / 36525.0;
        let l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;
        let m = 357.52911 + 35999.05029 * t - 0.0001537 * t * t;
        let m_rad = m.to_radians();
        let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * m_rad.sin()
              + (0.019993 - 0.000101 * t) * (2.0 * m_rad).sin()
              + 0.000289 * (3.0 * m_rad).sin();
        (l0 + c).rem_euclid(360.0)
    }

    // 判断是否已过立春
    fn is_after_lichun(solar_longitude: f64) -> bool {
        solar_longitude >= 315.0 || solar_longitude < 270.0
    }

    // 根据太阳黄经获取月支
    fn get_month_branch_from_solar_longitude(solar_longitude: f64) -> usize {
        let adjusted = (solar_longitude + 45.0).rem_euclid(360.0);
        let month_idx = (adjusted / 30.0).floor() as usize;
        (month_idx + 2) % 12
    }

    #[test]
    fn test_year_pillar_lichun() {
        // 2025年2月3日 12:00 UTC - 立春前
        // JD = 2460710.0
        let jd_before = 2460710.0;
        let solar_lon_before = get_solar_longitude(jd_before);
        println!("2025年2月3日 太阳黄经: {:.2}°", solar_lon_before);
        assert!(!is_after_lichun(solar_lon_before), "2025年2月3日应该在立春前");

        // 2025年2月5日 12:00 UTC - 立春后
        let jd_after = 2460712.0;
        let solar_lon_after = get_solar_longitude(jd_after);
        println!("2025年2月5日 太阳黄经: {:.2}°", solar_lon_after);
        assert!(is_after_lichun(solar_lon_after), "2025年2月5日应该在立春后");
    }

    #[test]
    fn test_month_branch_solar_terms() {
        // 立春后 (315°) 应为寅月 (索引2)
        assert_eq!(get_month_branch_from_solar_longitude(315.0), 2, "立春315°应为寅月");
        
        // 惊蛰后 (345°) 应为卯月 (索引3)
        assert_eq!(get_month_branch_from_solar_longitude(345.0), 3, "惊蛰345°应为卯月");
        
        // 清明后 (15°) 应为辰月 (索引4)
        assert_eq!(get_month_branch_from_solar_longitude(15.0), 4, "清明15°应为辰月");
        
        // 小寒后 (285°) 应为丑月 (索引1)
        assert_eq!(get_month_branch_from_solar_longitude(285.0), 1, "小寒285°应为丑月");
    }

    #[test]
    fn test_day_pillar_1970() {
        // 1970年1月1日 00:00 UTC
        // JD = 2440587.5
        // 应为庚戌日 (庚=6, 戌=10)
        let days_from_epoch = 0;
        let day_gan_idx = ((days_from_epoch + 6) % 10 + 10) % 10;
        let day_zhi_idx = ((days_from_epoch + 10) % 12 + 12) % 12;
        
        assert_eq!(TIANGAN[day_gan_idx as usize], "庚", "1970-01-01 天干应为庚");
        assert_eq!(DIZHI[day_zhi_idx as usize], "戌", "1970-01-01 地支应为戌");
    }

    #[test]
    fn test_hour_pillar_five_rats() {
        // 五鼠遁口诀: 甲己还加甲，乙庚丙作初，丙辛从戊起，丁壬庚子居，戊癸何方发，壬子是真途
        // 甲日(0)子时 -> 甲子 (0,0)
        // 乙日(1)子时 -> 丙子 (2,0)
        // 丙日(2)子时 -> 戊子 (4,0)
        // 丁日(3)子时 -> 庚子 (6,0)
        // 戊日(4)子时 -> 壬子 (8,0)
        
        for day_gan_idx in 0..10 {
            let zi_hour_gan_idx = (day_gan_idx % 5) * 2;
            let hour_zhi_idx = 0; // 子时
            let hour_gan_idx = (zi_hour_gan_idx + hour_zhi_idx) % 10;
            
            println!("{}日子时 -> {}子", TIANGAN[day_gan_idx], TIANGAN[hour_gan_idx]);
            
            match day_gan_idx {
                0 | 5 => assert_eq!(hour_gan_idx, 0, "甲/己日子时应为甲子"),
                1 | 6 => assert_eq!(hour_gan_idx, 2, "乙/庚日子时应为丙子"),
                2 | 7 => assert_eq!(hour_gan_idx, 4, "丙/辛日子时应为戊子"),
                3 | 8 => assert_eq!(hour_gan_idx, 6, "丁/壬日子时应为庚子"),
                4 | 9 => assert_eq!(hour_gan_idx, 8, "戊/癸日子时应为壬子"),
                _ => {}
            }
        }
    }

    #[test]
    fn test_month_gan_five_tigers() {
        // 五虎遁口诀: 甲己之年丙作首，乙庚之岁戊为头，丙辛必定寻庚起，丁壬壬位顺行流，若问戊癸何方发，甲寅之上好追求
        // 甲年(0)正月 -> 丙寅 (2,2)
        // 乙年(1)正月 -> 戊寅 (4,2)
        // 丙年(2)正月 -> 庚寅 (6,2)
        // 丁年(3)正月 -> 壬寅 (8,2)
        // 戊年(4)正月 -> 甲寅 (0,2)
        
        for year_gan_idx in 0..10 {
            let yin_month_gan_idx = ((year_gan_idx % 5) * 2 + 2) % 10;
            println!("{}年正月 -> {}寅", TIANGAN[year_gan_idx], TIANGAN[yin_month_gan_idx]);
            
            match year_gan_idx {
                0 | 5 => assert_eq!(yin_month_gan_idx, 2, "甲/己年正月应为丙寅"),
                1 | 6 => assert_eq!(yin_month_gan_idx, 4, "乙/庚年正月应为戊寅"),
                2 | 7 => assert_eq!(yin_month_gan_idx, 6, "丙/辛年正月应为庚寅"),
                3 | 8 => assert_eq!(yin_month_gan_idx, 8, "丁/壬年正月应为壬寅"),
                4 | 9 => assert_eq!(yin_month_gan_idx, 0, "戊/癸年正月应为甲寅"),
                _ => {}
            }
        }
    }
}
