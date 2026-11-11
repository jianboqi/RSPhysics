"""Convert local observations to 6S inputs without network access."""

import calendar
import csv
from datetime import datetime, timezone
import math
from pathlib import Path
import re

import numpy as np

from ._api import ModelError


def utc_datetime(value):
    """Parse an ISO 8601 string or datetime and require an explicit time zone."""
    if isinstance(value, str):
        try:
            value = datetime.fromisoformat(value)
        except ValueError as error:
            raise ModelError(
                "invalid_parameter", "Expected an ISO 8601 datetime"
            ) from error
    if not isinstance(value, datetime) or value.utcoffset() is None:
        raise ModelError("invalid_parameter", "Observation time requires a UTC offset")
    return value.astimezone(timezone.utc)


def solar_angles(latitude, longitude, utc):
    """Geometric solar position from NOAA GML's fractional-year equations."""
    if not (-90 <= latitude <= 90 and -180 <= longitude <= 180):
        raise ModelError(
            "invalid_parameter", "Latitude or longitude is outside its range"
        )
    minutes = utc.hour * 60 + utc.minute + utc.second / 60 + utc.microsecond / 60e6
    days = 365 + calendar.isleap(utc.year)
    gamma = math.tau / days * (utc.timetuple().tm_yday - 1 + minutes / 1440 - 0.5)
    c1, c2, c3 = (math.cos(k * gamma) for k in (1, 2, 3))
    s1, s2, s3 = (math.sin(k * gamma) for k in (1, 2, 3))
    eqtime = 229.18 * (
        0.000075 + 0.001868 * c1 - 0.032077 * s1 - 0.014615 * c2 - 0.040849 * s2
    )
    declination = (
        0.006918
        - 0.399912 * c1
        + 0.070257 * s1
        - 0.006758 * c2
        + 0.000907 * s2
        - 0.002697 * c3
        + 0.001480 * s3
    )
    hour_angle = math.radians((minutes + eqtime + 4 * longitude) % 1440 / 4 - 180)
    lat = math.radians(latitude)
    east = -math.cos(declination) * math.sin(hour_angle)
    north = math.cos(lat) * math.sin(declination) - math.sin(lat) * math.cos(
        declination
    ) * math.cos(hour_angle)
    up = math.sin(lat) * math.sin(declination) + math.cos(lat) * math.cos(
        declination
    ) * math.cos(hour_angle)
    return (
        math.degrees(math.atan2(math.hypot(east, north), up)),
        math.degrees(math.atan2(east, north)) % 360,
    )


def radiosonde_levels(levels, reference):
    """Use observations on the standard 6S altitude grid where they overlap."""
    levels = np.asarray(levels, dtype=float)
    if levels.ndim != 2 or levels.shape[1] not in (4, 5) or len(levels) < 2:
        raise ModelError(
            "invalid_parameter",
            "Radiosonde needs at least two [km,hPa,K,water g/m³,optional ozone g/m³] rows",
        )
    if (
        not np.isfinite(levels).all()
        or np.any(np.diff(levels[:, 0]) <= 0)
        or np.any(levels[:, 1] <= 0)
        or np.any(np.diff(levels[:, 1]) >= 0)
        or np.any(levels[:, 2] <= 0)
        or np.any(levels[:, 3:] < 0)
    ):
        raise ModelError(
            "invalid_parameter",
            "Radiosonde heights must increase, pressures decrease and observations be physical",
        )
    # Retain standard levels outside the measured height range.
    z = reference[:, 0]
    standard_pressure = reference[:, 1].copy()
    observed = (z >= levels[0, 0]) & (z <= levels[-1, 0]) & (z < 99999)
    if not np.any(observed):
        raise ModelError(
            "invalid_parameter", "Radiosonde must span at least one standard 6S level"
        )
    target = z[observed]
    reference[observed, 1] = np.exp(
        np.interp(target, levels[:, 0], np.log(levels[:, 1]))
    )
    for col in range(2, levels.shape[1]):
        reference[observed, col] = np.interp(target, levels[:, 0], levels[:, col])
    # A sounding from a different surface pressure can meet a standard-profile
    # level with lower pressure. Preserve the exterior pressure profile shape
    # while matching the nearest observed pressure at the transition.
    standard = np.flatnonzero(observed)
    for index, exterior in (
        (standard[0], z < target[0]),
        (standard[-1], z > target[-1]),
    ):
        if np.any(exterior):
            reference[exterior, 1] *= reference[index, 1] / standard_pressure[index]
    return reference


def wyoming_levels(path):
    """Read the University of Wyoming fixed-width TEXT:LIST sounding table."""
    lines = Path(path).read_text(encoding="utf-8-sig").splitlines()
    required = ("PRES", "HGHT", "TEMP", "MIXR")
    for index, line in enumerate(lines):
        headers = [line[i : i + 7].strip() for i in range(0, len(line), 7)]
        if all(name in headers for name in required):
            break
    else:
        raise ModelError(
            "invalid_parameter", "Wyoming sounding table header is missing"
        )
    rows = []
    for line in lines[index + 1 :]:
        if line.strip().startswith("Station"):
            break
        try:
            pressure, altitude, temperature, mixing_ratio = (
                float(line[j * 7 : (j + 1) * 7])
                for j in (headers.index(name) for name in required)
            )
        except ValueError:
            continue
        temperature += 273.15
        mixing_ratio /= 1000  # kg vapour per kg dry air
        if pressure <= 0 or temperature <= 0 or mixing_ratio < 0:
            continue
        vapour_pressure_pa = 100 * pressure * mixing_ratio / (0.62198 + mixing_ratio)
        water_g_m3 = 1000 * vapour_pressure_pa / (461.5 * temperature)
        if not rows or (altitude / 1000 > rows[-1][0] and pressure < rows[-1][1]):
            rows.append([altitude / 1000, pressure, temperature, water_g_m3])
    return rows


def _aot_at_550(row):
    aod = {}
    for key, value in row.items():
        match = re.fullmatch(
            r"A(?:OD|OT)[_ -]?(\d+)(?:nm)?(?:[-_]Total)?", key or "", re.I
        )
        if match:
            try:
                optical_depth = float(value)
            except (TypeError, ValueError):
                continue
            if math.isfinite(optical_depth) and optical_depth >= 0:
                aod[int(match[1])] = optical_depth
    if 550 in aod:
        return aod[550]
    lower = max((w for w in aod if 0 < w < 550), default=None)
    upper = min((w for w in aod if 550 < w <= 870), default=None)
    if lower is None or upper is None:
        return None
    x, y = aod[lower], aod[upper]
    weight = math.log(550 / lower) / math.log(upper / lower)
    if x == 0 or y == 0:
        return x + weight * (y - x)
    return math.exp(math.log(x) + weight * (math.log(y) - math.log(x)))


def aeronet_aot(path, when, max_age):
    """Nearest valid AOD observation from an AERONET all-points CSV."""
    with Path(path).open(encoding="utf-8-sig", newline="") as stream:
        for line in stream:
            if "Date(dd:mm:yyyy)" in line and "Time(hh:mm:ss)" in line:
                fields = [name.strip() for name in next(csv.reader([line]))]
                break
        else:
            raise ModelError(
                "invalid_parameter", "AERONET date and time columns are missing"
            )
        nearest = None
        for row in csv.DictReader(stream, fieldnames=fields):
            try:
                observed = datetime.strptime(
                    f"{row['Date(dd:mm:yyyy)']} {row['Time(hh:mm:ss)']}",
                    "%d:%m:%Y %H:%M:%S",
                ).replace(tzinfo=timezone.utc)
            except ValueError:
                continue
            delta = abs(observed - when)
            if max_age is not None and delta > max_age:
                continue
            if nearest is not None and delta >= nearest[0]:
                continue
            aot = _aot_at_550(row)
            if aot is not None:
                nearest = delta, aot
    if nearest is None:
        raise ModelError(
            "missing_data",
            "No valid AERONET 550 nm or bracketing AOD measurement within the requested time window",
        )
    return nearest[1]
