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

export type TimelineLevel = 'yuan' | 'hui' | 'yun' | 'shi' | 'xun';

export interface AuthorityEvidenceRef {
  label: string;
  url: string;
}

export interface AuthorityCoverage {
  min_year: number;
  max_year: number;
  covered: boolean;
}

export interface AuthorityMeta {
  requested_source: 'algorithm' | 'table';
  resolved_source: 'algorithm' | 'table';
  table_coverage?: AuthorityCoverage | null;
  fallback_reason?: string | null;
  authority_level: 'canonical' | 'derived';
  evidence_refs: AuthorityEvidenceRef[];
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
  calc_meta?: {
    mode: 'algorithm' | 'table' | 'compare';
    primary: 'algorithm' | 'table';
    year_start: 'lichun' | 'gregorian';
    hj_year: number;
  };
  variants?: {
    algorithm: FortuneVariant;
    table_raw: FortuneVariant;
    table_normalized: FortuneVariant;
  };
  diff?: {
    hexagram_major_diff: boolean;
    yun_diff: boolean;
    shi_diff: boolean;
    xun_diff: boolean;
    note: string;
  };
  authority?: AuthorityMeta;
}

export interface FortuneVariant {
  source: string;
  available: boolean;
  yuan: string;
  hui: string;
  yun: string;
  shi: string;
  xun: string;
  nian_ganzhi: string;
  hexagram_major: string;
  note: string;
  mapping_record?: FortuneResponse['mapping_record'] | null;
  mapping_record_normalized?: {
    gregorian_year: number;
    ganzhi: string;
    nian_hexagram_raw: string;
    nian_hexagram?: string | null;
    dynasty: string;
    person: string;
    yuan_raw: string;
    hui_raw: string;
    yun_raw: string;
    shi_raw: string;
    xun_raw: string;
    hui_name?: string | null;
    yun_name?: string | null;
    shi_name?: string | null;
    xun_name?: string | null;
  } | null;
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

export interface TimelineResponse extends TimelineData {
  year: number;
  calc_meta?: FortuneResponse['calc_meta'];
  variants?: FortuneResponse['variants'];
  diff?: FortuneResponse['diff'];
  mapping_record?: FortuneResponse['mapping_record'];
  authority?: AuthorityMeta;
  timeline_meta?: {
    primary_source: 'algorithm' | 'table';
    secondary_source?: 'algorithm' | 'table' | null;
  };
  timeline_variants?: {
    algorithm: TimelineData;
    table: TimelineData | null;
  };
}

export type BaziSource = 'auto' | 'sxtwl' | 'huangji_core';
export type BaziTimeBasis = 'standard' | 'true_solar';
export type BaziDayRollover = 'zi_chu_23' | 'zi_zheng_00';

export interface BaziAuthorityEvidenceRef {
  label: string;
  url_or_id: string;
  version?: string | null;
}

export interface BaziRuleProfile {
  time_basis: BaziTimeBasis | string;
  day_rollover: BaziDayRollover | string;
  timezone?: string | null;
  tz_offset_minutes: number;
  longitude: number;
}

export interface BaziAuthorityMeta {
  requested_source: BaziSource | string;
  resolved_source: Exclude<BaziSource, 'auto'> | string;
  fallback_reason?: string | null;
  authority_level: 'canonical' | 'derived' | string;
  rule_profile: BaziRuleProfile;
  evidence_refs: BaziAuthorityEvidenceRef[];
}

export interface BaziVariantPayload {
  available: boolean;
  reason?: string | null;
  payload?: BaziResponse | null;
}

export interface BaziHiddenStem {
  gan: string;
  gan_wuxing: string;
  ten_god: string;
  type: string;
  energy: number;
}

export interface BaziPillar {
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
  zhi_animal: string;
  nayin: string;
  gan_ten_god?: string;
  hidden_stems?: BaziHiddenStem[];
}

export interface BaziDayunCycle {
  cycle: number;
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
  start_age: number;
  end_age: number;
  year_range: string;
}

export interface BaziXiaoyunCycle {
  age: number;
  year: number;
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
}

export interface BaziLiunianYear {
  year: number;
  age: number;
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
  zodiac: string;
}

export interface BaziResponse {
  year_pillar: BaziPillar;
  month_pillar: BaziPillar;
  day_pillar: BaziPillar;
  hour_pillar: BaziPillar;
  wuxing_analysis: {
    day_master: string;
    day_master_gan?: string;
    day_master_strength: 'strong' | 'weak' | 'balanced';
    wuxing_counts: Record<string, number>;
    missing_wuxing: string[];
  };
  ten_gods_summary?: {
    year_gan: string;
    month_gan: string;
    day_gan: string;
    hour_gan: string;
  };
  dayun?: BaziDayunCycle[];
  xiaoyun?: BaziXiaoyunCycle;
  liunian?: BaziLiunianYear[];
  gender: 'male' | 'female' | 'other' | string;
  birth_year?: number;
  solar_term?: string;
  zodiac?: string;
  authority?: BaziAuthorityMeta;
  variants?: {
    sxtwl: BaziVariantPayload;
    huangji_core: BaziVariantPayload;
  };
}
