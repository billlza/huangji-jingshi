import tzLookup from 'tz-lookup';
import { DateTime } from 'luxon';

export type TimezoneResult = {
  offsetHours: number;
  zoneName: string | null;
  source: 'local' | 'remote';
};

export function computeLocalOffset(date: Date, lat: number, lon: number): TimezoneResult | null {
  try {
    const zone = tzLookup(lat, lon);
    const dt = DateTime.fromJSDate(date, { zone });
    const offsetHours = dt.offset / 60;
    return { offsetHours, zoneName: zone, source: 'local' };
  } catch {
    return null;
  }
}

async function fetchRemoteOffset(date: Date, lat: number, lon: number): Promise<TimezoneResult | null> {
  try {
    const base = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || '';
    const SUPABASE_ANON_KEY = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_SUPABASE_ANON_KEY || '';
    const q = new URLSearchParams({
      datetime: date.toISOString(),
      lat: String(lat),
      lon: String(lon),
    });
    const resp = await fetch(`${base}/functions/v1/calculate?${q.toString()}`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
        'Content-Type': 'application/json'
      }
    });
    if (!resp.ok) return null;
    const json = await resp.json();
    const offsetSeconds = typeof json.gmtOffset === 'number' ? json.gmtOffset : (json.offset_seconds ?? 0);
    const zoneName = json.zoneName ?? json.zone_name ?? null;
    if (typeof offsetSeconds !== 'number') return null;
    return { offsetHours: offsetSeconds / 3600, zoneName, source: 'remote' };
  } catch {
    return null;
  }
}

export async function resolveTimezoneOffset(date: Date, lat: number, lon: number, preferRemote = false): Promise<TimezoneResult> {
  if (preferRemote) {
    const remote = await fetchRemoteOffset(date, lat, lon);
    if (remote) return remote;
    const local = computeLocalOffset(date, lat, lon);
    if (local) return local;
    return { offsetHours: 0, zoneName: null, source: 'local' };
  }
  const local = computeLocalOffset(date, lat, lon);
  if (local) return local;
  const remote = await fetchRemoteOffset(date, lat, lon);
  if (remote) return remote;
  return { offsetHours: 0, zoneName: null, source: 'local' };
}

export function resolveTimezoneOffsetSync(date: Date, lat: number, lon: number): TimezoneResult {
  const local = computeLocalOffset(date, lat, lon);
  if (local) return local;
  return { offsetHours: 0, zoneName: null, source: 'local' };
}
