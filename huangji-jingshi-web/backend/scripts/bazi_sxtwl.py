#!/usr/bin/env python3
import json
import math
import sys
from datetime import datetime, timedelta, timezone
from importlib import metadata as importlib_metadata

try:
    from zoneinfo import ZoneInfo
except Exception:  # pragma: no cover
    ZoneInfo = None


def emit(payload):
    sys.stdout.write(json.dumps(payload, ensure_ascii=False))
    sys.stdout.flush()


def emit_error(code, message):
    emit({"ok": False, "error_code": code, "message": message})


def parse_datetime_utc(raw):
    text = raw.strip()
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    dt = datetime.fromisoformat(text)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


def equation_of_time_spencer(day_of_year):
    b = math.radians(360.0 * (day_of_year - 81.0) / 365.0)
    return 9.87 * math.sin(2.0 * b) - 7.67 * math.sin(b + math.radians(78.7))


def resolve_standard_local(dt_utc, timezone_name, tz_offset_minutes):
    if timezone_name:
        if ZoneInfo is None:
            raise RuntimeError("zoneinfo_not_available")
        try:
            return dt_utc.astimezone(ZoneInfo(timezone_name))
        except Exception as exc:
            raise RuntimeError(f"invalid_timezone:{timezone_name}:{exc}") from exc

    return dt_utc.astimezone(timezone(timedelta(minutes=tz_offset_minutes)))


def true_solar_datetime(local_standard_dt, longitude, standard_meridian):
    day_of_year = local_standard_dt.timetuple().tm_yday
    eot_minutes = equation_of_time_spencer(day_of_year)
    longitude_correction_minutes = (longitude - standard_meridian) * 4.0
    total_minutes = longitude_correction_minutes + eot_minutes
    return local_standard_dt + timedelta(minutes=total_minutes)


def hour_to_zhi_index(hour_float):
    hour_float = hour_float % 24.0
    if hour_float >= 23.0 or hour_float < 1.0:
        return 0
    return int(math.floor((hour_float + 1.0) / 2.0)) % 12


def get_sxtwl_version():
    try:
        return importlib_metadata.version("sxtwl")
    except Exception:
        return None


def run():
    if "--health" in sys.argv:
        try:
            import sxtwl  # type: ignore

            emit(
                {
                    "ok": True,
                    "engine": "sxtwl",
                    "version": get_sxtwl_version(),
                }
            )
        except ModuleNotFoundError:
            emit_error("module_not_available", "python module 'sxtwl' is not installed")
        except Exception as exc:
            emit_error("runtime_error", f"sxtwl health check failed: {exc}")
        return

    try:
        payload = json.load(sys.stdin)
    except Exception as exc:
        emit_error("invalid_input", f"invalid stdin json: {exc}")
        return

    try:
        import sxtwl  # type: ignore
    except ModuleNotFoundError:
        emit_error("module_not_available", "python module 'sxtwl' is not installed")
        return
    except Exception as exc:
        emit_error("runtime_error", f"failed to import sxtwl: {exc}")
        return

    try:
        dt_utc = parse_datetime_utc(payload["datetime_utc"])
        timezone_name = payload.get("timezone")
        tz_offset_minutes = int(payload.get("tz_offset_minutes", 480))
        longitude = float(payload.get("longitude", 116.4))
        time_basis = (payload.get("time_basis") or "standard").strip().lower()
        day_rollover = (payload.get("day_rollover") or "zi_chu_23").strip().lower()

        local_standard = resolve_standard_local(dt_utc, timezone_name, tz_offset_minutes)
        standard_meridian = (tz_offset_minutes / 60.0) * 15.0
        if time_basis == "true_solar":
            basis_dt = true_solar_datetime(local_standard, longitude, standard_meridian)
        else:
            basis_dt = local_standard

        hour_float = (
            basis_dt.hour
            + basis_dt.minute / 60.0
            + basis_dt.second / 3600.0
            + basis_dt.microsecond / 3_600_000_000.0
        )
        is_late_zi = day_rollover == "zi_chu_23" and hour_float >= 23.0

        year_month_date = basis_dt.date()
        day_date = year_month_date + timedelta(days=1) if is_late_zi else year_month_date

        ym_day = sxtwl.fromSolar(
            year_month_date.year, year_month_date.month, year_month_date.day
        )
        day_day = sxtwl.fromSolar(day_date.year, day_date.month, day_date.day)

        year_gz = ym_day.getYearGZ(True)
        month_gz = ym_day.getMonthGZ()
        day_gz = day_day.getDayGZ()

        hour_int = int(math.floor(hour_float)) % 24
        # We resolve day rollover explicitly via day_rollover/day_date above,
        # so disable sxtwl's internal early/late-zi split to avoid double-shifting.
        hour_gz = sxtwl.getShiGz(day_gz.tg, hour_int, False)

        solar_term_name = None
        try:
            jq = ym_day.getJieQi()
            if jq >= 0 and hasattr(sxtwl, "jqmc"):
                solar_term_name = sxtwl.jqmc[jq]
        except Exception:
            solar_term_name = None

        emit(
            {
                "ok": True,
                "engine": "sxtwl",
                "engine_version": get_sxtwl_version(),
                "year": {"tg": int(year_gz.tg), "dz": int(year_gz.dz)},
                "month": {"tg": int(month_gz.tg), "dz": int(month_gz.dz)},
                "day": {"tg": int(day_gz.tg), "dz": int(day_gz.dz)},
                "hour": {"tg": int(hour_gz.tg), "dz": int(hour_gz.dz)},
                "solar_term": solar_term_name,
                "solar_longitude": None,
                "true_solar_hour": hour_float if time_basis == "true_solar" else None,
                "is_late_zi": bool(is_late_zi),
            }
        )
    except ValueError as exc:
        emit_error("out_of_supported_range", str(exc))
    except Exception as exc:
        emit_error("runtime_error", f"sxtwl execution failed: {exc}")


if __name__ == "__main__":
    run()
