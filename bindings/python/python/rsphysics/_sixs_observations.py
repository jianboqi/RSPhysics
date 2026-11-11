"""Read observations and convert them to 6S inputs."""

import calendar
import csv
from datetime import datetime, timezone
from html import unescape
import math
from pathlib import Path
import re
from urllib.parse import urlsplit
from urllib.request import urlopen

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
    """Read a Wyoming fixed-width or CSV sounding table."""
    return _wyoming_text(Path(path).read_text(encoding="utf-8-sig"))


def wyoming_url_levels(url, timeout):
    """Read a single sounding from a Wyoming TEXT:LIST or TEXT:CSV URL."""
    if (
        urlsplit(url).scheme not in {"http", "https"}
        or not math.isfinite(timeout)
        or timeout <= 0
    ):
        raise ModelError(
            "invalid_parameter", "Expected an HTTP(S) URL and a positive timeout"
        )
    limit = 4 * 1024**2
    try:
        with urlopen(url, timeout=timeout) as response:
            raw = response.read(limit + 1)
    except OSError as error:
        raise ModelError(
            "missing_data", f"Could not retrieve sounding: {error}"
        ) from error
    if len(raw) > limit:
        raise ModelError("invalid_parameter", "Sounding response exceeds 4 MiB")
    text = raw.decode("utf-8-sig", errors="replace")
    tables = re.findall(r"<pre\b[^>]*>(.*?)</pre\s*>", text, re.I | re.S)
    soundings = [
        table for table in tables if re.search(r"\bPRES\s+HGHT\s+TEMP\b", table)
    ]
    if len(soundings) > 1:
        raise ModelError("invalid_parameter", "Select one sounding in the Wyoming URL")
    return _wyoming_text(unescape(soundings[0]) if soundings else text)


def _wyoming_text(text):
    lines = text.splitlines()
    csv_columns = (
        "pressure",
        "geopotential_height",
        "temperature",
        "dew_point_temperature",
    )
    for index, line in enumerate(lines):
        if "," not in line:
            continue
        headers = [
            field.strip().lower().replace(" ", "_")
            for field in next(csv.reader([line]))
        ]
        columns = [
            next(
                (
                    i
                    for i, name in enumerate(headers)
                    if name == key or name.startswith(key + "_")
                ),
                None,
            )
            for key in csv_columns
        ]
        if None not in columns:
            rows = []
            for row in csv.reader(lines[index + 1 :]):
                try:
                    rows.append([float(row[i]) for i in columns])
                except (ValueError, IndexError):
                    continue
            return _dewpoint_levels(*np.asarray(rows, dtype=float).reshape(-1, 4).T)
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


def bas_levels(path):
    """Read legacy BAS text with pressure, height, temperature and dew point."""
    try:
        pressure, altitude, temperature, dewpoint = np.loadtxt(
            path, skiprows=1, usecols=(2, 3, 4, 6), ndmin=2, unpack=True
        )
    except ValueError as error:
        raise ModelError("invalid_parameter", "Invalid BAS sounding table") from error
    return _dewpoint_levels(pressure, altitude, temperature, dewpoint)


def _dewpoint_levels(pressure, altitude, temperature, dewpoint):
    temperature = temperature + 273.15
    valid = np.isfinite([pressure, altitude, temperature, dewpoint]).all(axis=0)
    valid &= (pressure > 0) & (temperature > 0) & (dewpoint > -150)
    pressure, altitude, temperature, dewpoint = (
        column[valid] for column in (pressure, altitude, temperature, dewpoint)
    )
    # Saturation vapour pressure at the dew point, converted from hPa to Pa.
    vapour_pa = 611.2 * np.exp(17.67 * dewpoint / (243.5 + dewpoint))
    valid = vapour_pa < 100 * pressure
    return np.column_stack(
        [
            altitude[valid] / 1000,
            pressure[valid],
            temperature[valid],
            1000 * vapour_pa[valid] / (461.5 * temperature[valid]),
        ]
    )


def aster_samples(path):
    """Read ASTER/ECOSTRESS reflectance as ascending µm and fraction arrays."""
    metadata = {}
    rows = []
    for line in (
        Path(path).read_text(encoding="utf-8-sig", errors="replace").splitlines()
    ):
        if ":" in line and not rows:
            name, value = line.split(":", 1)
            metadata[name.strip().lower()] = value.strip().lower()
            continue
        if not line.strip():
            continue
        columns = line.split()
        if len(columns) == 2:
            try:
                rows.append(tuple(map(float, columns)))
                continue
            except ValueError:
                pass
        if rows:
            raise ModelError("invalid_parameter", "Unexpected data in ASTER spectrum")
    if len(rows) < 2:
        raise ModelError(
            "invalid_parameter", "ASTER spectrum needs at least two samples"
        )
    x_units, y_units = (metadata.get(key, "") for key in ("x units", "y units"))
    if x_units == "nm" or any(
        unit in x_units for unit in ("nanometer", "nanometre", "(nm)")
    ):
        wavelength_factor = 0.001
    elif any(
        unit in x_units
        for unit in ("micrometer", "micrometre", "micron", "µm", "μm", "(um)")
    ):
        wavelength_factor = 1
    else:
        raise ModelError("invalid_parameter", "ASTER wavelength units must be µm or nm")
    if "reflectance" not in y_units and "reflectivity" not in y_units:
        raise ModelError("invalid_parameter", "ASTER spectrum must contain reflectance")
    if "percent" in y_units or "%" in y_units:
        reflectance_factor = 0.01
    elif any(
        unit in y_units for unit in ("fraction", "unitless", "dimensionless", "0-1")
    ):
        reflectance_factor = 1
    else:
        raise ModelError(
            "invalid_parameter", "ASTER reflectance units must be percent or fraction"
        )
    x, y = np.asarray(sorted(rows)).T
    if not np.isfinite([x, y]).all() or np.any(x <= 0) or np.any(np.diff(x) <= 0):
        raise ModelError(
            "invalid_parameter",
            "ASTER wavelengths must be positive and unique with finite reflectances",
        )
    return x * wavelength_factor, y * reflectance_factor


def _aot_at_550(row):
    channels = {"direct": {}, "coincident": {}, "extinction": {}}
    for key, value in row.items():
        key = key or ""
        match = re.fullmatch(r"A(?:OD|OT)[_ -]?(\d+)(?:nm)?(?:[-_]Total)?", key, re.I)
        source = "direct"
        if not match:
            v3 = re.fullmatch(
                r"AOD_(Coincident_Input|Extinction-Total)\[(\d+)nm\]", key, re.I
            )
            if not v3:
                continue
            source = (
                "coincident" if v3[1].lower() == "coincident_input" else "extinction"
            )
            wavelength = int(v3[2])
        else:
            wavelength = int(match[1])
        if wavelength > 0:
            try:
                optical_depth = float(value)
            except (TypeError, ValueError):
                continue
            if math.isfinite(optical_depth) and optical_depth >= 0:
                channels[source][wavelength] = optical_depth
    for aod in channels.values():
        aot = _interpolate_aot(aod)
        if aot is not None:
            return aot
    return None


def _interpolate_aot(aod):
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


def _aeronet_closest(path, when, max_age, parse, description):
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
            value = parse(row)
            if value is not None:
                nearest = delta, value
    if nearest is None:
        raise ModelError(
            "missing_data",
            f"No valid AERONET {description} within the requested time window",
        )
    return nearest[1]


def aeronet_aot(path, when, max_age):
    """Nearest valid AOD observation from an AERONET all-points CSV."""
    return _aeronet_closest(
        path, when, max_age, _aot_at_550, "550 nm or bracketing AOD measurement"
    )


def aeronet_inversion(path, when, max_age, wavelengths_um):
    """Read a complete V3 ALL or V2 Combined aerosol retrieval."""

    def parse(row):
        aot = _aot_at_550(row)
        if aot is None or aot <= 0:
            return None
        real, imaginary, sizes = {}, {}, {}
        for key, value in row.items():
            if not key:
                continue
            v3 = re.fullmatch(
                r"Refractive_Index-(Real|Imaginary)_Part\[(\d+)nm\]", key, re.I
            )
            v2 = re.fullmatch(r"REF([RI])\((\d+)\)", key, re.I)
            radius = float(key) if re.fullmatch(r"\d+\.\d+", key) else None
            size_bin = radius is not None and 0.0001 <= radius <= 100
            try:
                sample = float(value)
            except (TypeError, ValueError):
                if size_bin:
                    return None
                continue
            if not math.isfinite(sample) or sample < 0:
                if size_bin:
                    return None
                continue
            if v3 or v2:
                index = v3 or v2
                is_real = index[1].lower() in {"real", "r"}
                if (1 if is_real else 0) <= sample <= 5:
                    (real if is_real else imaginary)[int(index[2]) / 1000] = sample
                continue
            if size_bin:
                sizes[radius] = sample
        if min(len(real), len(imaginary), len(sizes)) < 2 or not any(sizes.values()):
            return None

        # Extrapolate each refractive component with its nearest measured value.
        def interpolate(values):
            x, y = zip(*sorted(values.items()))
            return np.interp(wavelengths_um, x, y).tolist()

        return aot, sorted(sizes.items()), interpolate(real), interpolate(imaginary)

    return _aeronet_closest(path, when, max_age, parse, "complete aerosol inversion")
