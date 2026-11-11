"""Matplotlib plots of 6S scan results."""

import numpy as np

from ._api import ModelError


def plot_spectrum(scan, *, ax=None, ylabel=None, **style):
    """Plot a spectral or sensor-band scan and return the Matplotlib axes."""
    x = scan.get("wavelengths_um", scan.get("centre_wavelengths_um"))
    return _plot_line(scan, x, "Wavelength (µm)", ax, ylabel, style)


def plot_principal_plane(scan, *, ax=None, ylabel=None, **style):
    """Plot a principal-plane scan against signed view zenith in degrees."""
    return _plot_line(
        scan,
        scan["signed_view_zenith_deg"],
        "Signed view zenith (°)",
        ax,
        ylabel,
        style,
    )


def _plot_line(scan, x, xlabel, ax, ylabel, style):
    y = np.asarray(scan.get("values", scan["apparent_reflectance"]))
    if x is None or y.shape != np.shape(x):
        raise ModelError(
            "invalid_parameter", "A scan plot needs one scalar per axis sample"
        )
    if ax is None:
        from matplotlib import pyplot as plt

        _, ax = plt.subplots()
    ax.plot(x, y, **style)
    ax.set(
        xlabel=xlabel,
        ylabel=ylabel
        or ("Selected output" if "values" in scan else "Apparent reflectance"),
    )
    return ax


def plot_angular_grid(scan, *, ax=None, colorbar_label=None, **style):
    """Plot an angular grid in polar coordinates; north is 0°."""
    azimuth = scan["azimuth_deg"]
    zenith = scan["zenith_deg"]
    values = np.asarray(scan.get("values", scan["apparent_reflectance"]))
    if min(len(azimuth), len(zenith)) < 2 or values.shape != (
        len(zenith),
        len(azimuth),
    ):
        raise ModelError(
            "invalid_parameter",
            "A polar plot needs a grid with one scalar per angle pair",
        )
    if ax is None:
        from matplotlib import pyplot as plt

        _, ax = plt.subplots(subplot_kw={"projection": "polar"})
    if ax.name != "polar":
        raise ModelError("invalid_parameter", "An angular grid needs polar axes")
    steps = np.diff(azimuth)
    if np.allclose(steps, steps[0]) and np.isclose(
        azimuth[-1] - azimuth[0] + steps[0], 360
    ):
        azimuth = np.r_[azimuth, azimuth[0] + 360]
        values = np.c_[values, values[:, 0]]
    contour = ax.contourf(np.deg2rad(azimuth), zenith, values, **style)
    ax.set_theta_zero_location("N")
    ax.set_theta_direction(-1)
    ax.figure.colorbar(
        contour,
        ax=ax,
        label=colorbar_label
        or ("Selected output" if "values" in scan else "Apparent reflectance"),
    )
    return ax
