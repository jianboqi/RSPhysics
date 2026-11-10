"""6SV2.1 through a direct Fortran C ABI; no 6S executable is launched.

All configuration is validated/rendered in the public Rust adapter. Parameters
use micrometres, degrees, km, hPa, g/cm2 and atm-cm, as named. The original
mixed-precision kernel, full formatted report and auxiliary files are retained.
"""

from __future__ import annotations
from copy import deepcopy
from concurrent.futures import ProcessPoolExecutor
import json
from collections.abc import Mapping
import numpy as np
from . import _native
from ._api import (
    ModelError,
    _call,
    _numpy_outputs,
    run as _run,
    run_batch as _run_batch,
)

MODEL_ID = "atmosphere.sixsv21"
REFRACTIVE_WAVELENGTHS_UM = np.array(
    [
        0.350,
        0.400,
        0.412,
        0.443,
        0.470,
        0.488,
        0.515,
        0.550,
        0.590,
        0.633,
        0.670,
        0.694,
        0.760,
        0.860,
        1.240,
        1.536,
        1.650,
        1.950,
        2.250,
        3.750,
    ]
)
BRDF_MODELS = {
    "measured": 0,
    "hapke": 1,
    "verstraete": 2,
    "roujean": 3,
    "walthall": 4,
    "minnaert": 5,
    "ocean": 6,
    "iaquinta_pinty": 7,
    "rahman": 8,
    "kuusk": 9,
    "modis": 10,
    "ross_li_maignan": 11,
}


def bands():
    """Original band IDs 2..199 and labels, extracted from the pinned source."""
    return json.loads(_call(_native.sixs_resources_json))["bands"]


def standard_levels(profile=6):
    """Copy a standard profile as (34,5) NumPy rows: km, hPa, K, g/m3, g/m3.

    The original final 99999-km sentinel row is retained. Profile IDs are 1..6.
    """
    try:
        return np.array(
            json.loads(_call(_native.sixs_resources_json))["profiles"][str(profile)][
                "levels"
            ],
            dtype=float,
        )
    except KeyError as error:
        raise ModelError(
            "invalid_parameter", "standard profile must be 1..6"
        ) from error


def _plain(value):
    if isinstance(value, SixS):
        return value.to_dict()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, Mapping):
        return {key: _plain(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_plain(item) for item in value]
    return value


def prepare(configuration=None):
    """Validate/default configuration and export the exact original input cards."""
    try:
        payload = json.dumps(_plain(configuration or {}), allow_nan=False)
    except (TypeError, ValueError) as error:
        raise ModelError("invalid_parameter", str(error)) from error
    return json.loads(_call(_native.sixs_prepare_json, payload))


def run(configuration=None, **sections):
    """Run one direct library calculation; return the standard result/manifest.

    Sections may be passed as mappings, e.g. ``run(spectrum=Spectrum.band(43))``.
    No compiler or separate 6S installation is required by a prebuilt wheel.
    """
    parameters = _plain(configuration or {})
    if not isinstance(parameters, dict):
        raise ModelError("invalid_parameter", "configuration must be a mapping or SixS")
    parameters.update(_plain(sections))
    return _run(MODEL_ID, parameters)


def run_cards(input_cards, *, files=None, program="standard"):
    """Run original input cards and named auxiliary files through the same C ABI.

    Advanced input preserves original options; it bypasses structured physical
    validation. ``files`` maps simple ASCII filenames to original ASCII contents.
    """
    return run(input_cards=input_cards, files=files or {}, program=program)


def parse_report(report, *, files=None):
    """Parse an existing original report without executing the model."""
    return _numpy_outputs(
        json.loads(
            _call(_native.sixs_parse_report_json, report, json.dumps(files or {}))
        )
    )


def _batch_item(parameters):
    try:
        return run(parameters), None
    except ModelError as error:
        # Exception reconstruction must not depend on ModelError's constructor
        # signature in spawned workers.
        return None, (error.kind, str(error))


def run_batch(configurations, *, workers=1):
    """Ordered, fail-fast batch. Parallel workers each call the direct library.

    For ``workers>1`` use Python's usual ``if __name__ == '__main__'`` guard
    in scripts. No 6S executable is invoked. Native COMMON state is reset and
    calls within a process are serialized; worker processes permit parallelism.
    """
    if isinstance(workers, bool) or not isinstance(workers, int) or workers < 1:
        raise ModelError("invalid_parameter", "workers must be a positive integer")
    rows = [_plain(row) for row in configurations]
    if workers == 1:
        return _run_batch(MODEL_ID, rows)
    for index, row in enumerate(rows):
        try:
            prepare(row)
        except ModelError as error:
            raise ModelError(error.kind, f"batch row {index}: {error}") from error
    results = []
    with ProcessPoolExecutor(max_workers=workers) as pool:
        iterator = pool.map(_batch_item, rows)
        for index in range(len(rows)):
            try:
                result, failure = next(iterator)
                if failure is not None:
                    raise ModelError(failure[0], failure[1])
                results.append(result)
            except Exception as error:
                raise ModelError(
                    getattr(error, "kind", "internal"), f"batch row {index}: {error}"
                ) from error
    return results


def spectrum(wavelengths_um, configuration=None, *, workers=1):
    """Monochromatic spectral sweep; return NumPy signals and full ordered runs."""
    wavelengths = np.asarray(wavelengths_um, dtype=float)
    if wavelengths.ndim != 1 or not np.isfinite(wavelengths).all():
        raise ModelError(
            "invalid_parameter", "wavelengths_um must be a finite one-dimensional array"
        )
    rows = []
    for wavelength in wavelengths:
        row = deepcopy(_plain(configuration or {}))
        row["spectrum"] = Spectrum.monochromatic(float(wavelength))
        rows.append(row)
    runs = run_batch(rows, workers=workers)
    return {
        "wavelengths_um": wavelengths.copy(),
        "apparent_reflectance": np.array(
            [r["result"]["apparent_reflectance"] for r in runs]
        ),
        "apparent_radiance_w_m2_sr_um": np.array(
            [r["result"]["apparent_radiance_w_m2_sr_um"] for r in runs]
        ),
        "runs": runs,
    }


class SixS:
    """Reusable independent configuration; each run resets the original kernel."""

    def __init__(self, configuration=None, **sections):
        self.configuration = deepcopy(_plain(configuration or {}))
        self.configuration.update(_plain(sections))

    def to_dict(self):
        return deepcopy(self.configuration)

    def prepare(self):
        return prepare(self.configuration)

    def input_cards(self):
        return self.prepare()["prepared"]["input_cards"]

    def run(self, **sections):
        return run(self.configuration, **sections)


class Geometry:
    @staticmethod
    def user(
        solar_zenith_deg=30.0,
        solar_azimuth_deg=0.0,
        view_zenith_deg=20.0,
        view_azimuth_deg=90.0,
        month=7,
        day=1,
    ):
        return dict(
            mode=0,
            solar_zenith_deg=solar_zenith_deg,
            solar_azimuth_deg=solar_azimuth_deg,
            view_zenith_deg=view_zenith_deg,
            view_azimuth_deg=view_azimuth_deg,
            month=month,
            day=day,
        )

    @staticmethod
    def satellite(mode, **parameters):
        return dict(mode=mode, **parameters)


class Atmosphere:
    @staticmethod
    def standard(profile=2):
        return {"profile": profile}

    @staticmethod
    def water_ozone(water_vapour_g_cm2, ozone_atm_cm):
        return dict(
            profile=8, water_vapour_g_cm2=water_vapour_g_cm2, ozone_atm_cm=ozone_atm_cm
        )

    @staticmethod
    def radiosonde(levels):
        return dict(profile=7, levels=_plain(levels))


class Aerosol:
    @staticmethod
    def predefined(model=1, *, aot550=0.2, visibility_km=None):
        return dict(model=model, aot550=aot550, visibility_km=visibility_km)

    @staticmethod
    def mixture(components, **loading):
        return dict(model=4, components=_plain(components), **loading)

    @staticmethod
    def layered(layers):
        return dict(model=-1, layers=_plain(layers))

    @staticmethod
    def lognormal(modes, *, radius_min_um=0.01, radius_max_um=1.0, **options):
        return dict(
            model=8,
            modes=_plain(modes),
            radius_min_um=radius_min_um,
            radius_max_um=radius_max_um,
            **options,
        )

    @staticmethod
    def gamma(parameters, *, radius_min_um=0.01, radius_max_um=1.0, **options):
        return dict(
            model=9,
            gamma=_plain(parameters),
            radius_min_um=radius_min_um,
            radius_max_um=radius_max_um,
            **options,
        )

    @staticmethod
    def junge(exponent=3.5, *, radius_min_um=0.01, radius_max_um=1.0, **options):
        return dict(
            model=10,
            junge_exponent=exponent,
            radius_min_um=radius_min_um,
            radius_max_um=radius_max_um,
            **options,
        )

    @staticmethod
    def sunphotometer(distribution, **options):
        return dict(model=11, sunphotometer=_plain(distribution), **options)

    @staticmethod
    def from_mie(filename, **loading):
        return dict(model=12, mie_file=filename, **loading)

    @staticmethod
    def mode(
        mean_radius_um=0.1,
        sigma=1.5,
        fraction=1.0,
        *,
        refractive_real=None,
        refractive_imag=None,
    ):
        return dict(
            mean_radius_um=mean_radius_um,
            sigma=sigma,
            fraction=fraction,
            refractive_real=_plain(
                refractive_real if refractive_real is not None else [1.5] * 20
            ),
            refractive_imag=_plain(
                refractive_imag if refractive_imag is not None else [0.01] * 20
            ),
        )


class Altitudes:
    @staticmethod
    def satellite(*, target_altitude_km=0.0, target_pressure_hpa=None):
        return dict(
            target_altitude_km=target_altitude_km,
            target_pressure_hpa=target_pressure_hpa,
            sensor_height_km=None,
        )

    @staticmethod
    def ground(**target):
        return dict(sensor_height_km=0.0, **target)

    @staticmethod
    def aircraft(
        height_km,
        *,
        path_water_vapour_g_cm2=-1.0,
        path_ozone_atm_cm=-1.0,
        path_aot550=-1.0,
        **target,
    ):
        return dict(
            sensor_height_km=height_km,
            path_water_vapour_g_cm2=path_water_vapour_g_cm2,
            path_ozone_atm_cm=path_ozone_atm_cm,
            path_aot550=path_aot550,
            **target,
        )


class Spectrum:
    @staticmethod
    def monochromatic(wavelength_um=0.55):
        return dict(mode="monochromatic", wavelength_um=wavelength_um)

    @staticmethod
    def band(band):
        return dict(mode="band", band=band)

    @staticmethod
    def uniform(lower_um, upper_um):
        return dict(mode="uniform", lower_um=lower_um, upper_um=upper_um)

    @staticmethod
    def step(lower_um, upper_um):
        return dict(mode="step", lower_um=lower_um, upper_um=upper_um)

    @staticmethod
    def response(lower_um, upper_um, response):
        return dict(
            mode="response",
            lower_um=lower_um,
            upper_um=upper_um,
            response=_plain(response),
        )


class Surface:
    @staticmethod
    def lambertian(reflectance=0.2):
        return dict(mode="lambertian", target=dict(spectrum=0, value=reflectance))

    @staticmethod
    def predefined(spectrum=1):
        return dict(mode="lambertian", target=dict(spectrum=spectrum))

    @staticmethod
    def spectrum(lower_um, upper_um, reflectance):
        return dict(
            mode="lambertian",
            target=dict(
                spectrum=-1,
                lower_um=lower_um,
                upper_um=upper_um,
                values=_plain(reflectance),
            ),
        )

    @staticmethod
    def heterogeneous(target, background, *, radius_km=1.0):
        def select(value):
            return (
                dict(spectrum=0, value=value) if np.isscalar(value) else _plain(value)
            )

        return dict(
            mode="heterogeneous",
            target=select(target),
            background=select(background),
            radius_km=radius_km,
        )

    @staticmethod
    def brdf(model, parameters):
        return dict(
            mode="brdf",
            brdf_model=BRDF_MODELS[model] if isinstance(model, str) else model,
            parameters=_plain(parameters),
        )

    @staticmethod
    def measured(sun_table, view_table, *, albedo, directional_reflectance):
        return dict(
            mode="brdf",
            brdf_model=0,
            sun_table=_plain(sun_table),
            view_table=_plain(view_table),
            albedo=albedo,
            directional_reflectance=directional_reflectance,
        )

    @staticmethod
    def hapke(single_scattering_albedo, asymmetry, hotspot_amplitude, hotspot_width):
        return Surface.brdf(
            "hapke",
            [single_scattering_albedo, asymmetry, hotspot_amplitude, hotspot_width],
        )

    @staticmethod
    def modis(isotropic, volumetric, geometric):
        return Surface.brdf("modis", [isotropic, volumetric, geometric])

    @staticmethod
    def ocean(
        wind_speed_m_s=5.0, wind_azimuth_deg=0.0, salinity_ppt=34.3, pigment_mg_m3=0.1
    ):
        return Surface.brdf(
            "ocean", [wind_speed_m_s, wind_azimuth_deg, salinity_ppt, pigment_mg_m3]
        )


class Correction:
    @staticmethod
    def none():
        return dict(mode="none")

    @staticmethod
    def from_radiance(value):
        return dict(mode="radiance", value=value)

    @staticmethod
    def from_reflectance(value):
        return dict(mode="reflectance", value=value)


class Polarization:
    @staticmethod
    def stokes(q, u):
        return dict(mode="stokes", q=q, u=u)

    @staticmethod
    def vegetation(fraction):
        return dict(mode="vegetation", vegetation_fraction=fraction)

    @staticmethod
    def ocean(wind_speed_m_s=5.0, wind_azimuth_deg=0.0):
        return dict(
            mode="ocean",
            wind_speed_m_s=wind_speed_m_s,
            wind_azimuth_deg=wind_azimuth_deg,
        )
