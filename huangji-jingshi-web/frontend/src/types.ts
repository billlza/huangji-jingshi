export interface LunarInfo {
  lunar_year: string;
  lunar_month: string;
  lunar_day: string;
  ganzhi_year: string;
  ganzhi_month: string;
  ganzhi_day: string;
  ganzhi_hour: string;
  zodiac: string;
  solar_term: string | null;
  twelve_officer: string;
  aus_directions: string;
  yi: string[];
  ji: string[];
}

export interface PeriodInfo {
  name: string;
  start_year: number;
  end_year: number;
  index: number;
  max_index: number;
}

export interface HuangjiInfo {
  yuan: PeriodInfo;
  hui: PeriodInfo;
  yun: PeriodInfo;
  shi: PeriodInfo;
  xun: PeriodInfo;
  year_gua: string;
}

export interface TimelineData {
  current: HuangjiInfo;
  yuan_list: PeriodInfo[];
  hui_list: PeriodInfo[];
  yun_list: PeriodInfo[];
  shi_list: PeriodInfo[];
  xun_list: PeriodInfo[];
}

export interface FortuneResponse {
  yuan: string;
  hui: string;
  yun: string;
  shi: string;
  xun: string;
  nian_ganzhi: string;
  hexagram_major: string;
  hexagram_code?: number[]; // Array of 6 bits
  flying_star?: string;
  note: string;
  lunar?: LunarInfo;
  period_info?: HuangjiInfo;
  next_yun_start_year?: number;
  next_shi_start_year?: number;
  next_xun_start_year?: number;
  mapping_record?: {
    gregorian_year: number;
    ganzhi: string;
    nian_hexagram: string;
    dynasty: string;
    person: string;
    yuan_raw: string;
    hui_raw: string;
    yun_raw: string;
    shi_raw: string;
    xun_raw: string;
  };
}

export interface SkyResponse {
  bodies: Array<{
    name: string;
    alt_deg: number;
    az_deg: number;
    distance_au?: number;
    ra_deg?: number;
    dec_deg?: number;
  }>;
  note: string;
  jd?: number;
  lst_deg?: number;
  gmst_deg?: number;
  delta_t_sec?: number;
}

export interface CombinedResponse {
  fortune: FortuneResponse;
  sky: SkyResponse;
}
