"""Render a labelled overview of the INFORM retrieval GeoTIFFs."""

import argparse
from pathlib import Path

import matplotlib
import numpy as np
from osgeo import gdal

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.colors import BoundaryNorm, ListedColormap
from matplotlib.patches import Patch


def preview(output):
    gdal.UseExceptions()
    lai = gdal.Open(str(output / "inform_lai_20m.tif"))
    quality = gdal.Open(str(output / "inform_quality_20m.tif"))
    x, sx, _, y, _, sy = lai.GetGeoTransform()
    extent = (x / 1000, (x + sx * lai.RasterXSize) / 1000,
              (y + sy * lai.RasterYSize) / 1000, y / 1000)
    shape = {"buf_xsize": min(lai.RasterXSize, 1100), "buf_ysize": min(lai.RasterYSize, 1100)}
    layers = lai.ReadAsArray(**shape)
    flags = quality.ReadAsArray(**shape)
    nodata = lai.GetRasterBand(1).GetNoDataValue()

    fig, axes = plt.subplots(2, 2, figsize=(12, 11), layout="constrained")
    fig.suptitle(f"INFORM single-crown LAI | 20 m\n{lai.GetMetadataItem('SCENE')}", fontsize=14)
    for ax, data, title, label, cmap, vmax in zip(
        axes.flat, layers[:3],
        ("Best-fit crown LAI", "Mean crown LAI among the 16 best candidates", "Candidate crown LAI standard deviation"),
        ("LAI (m²/m²)", "LAI (m²/m²)", "Population standard deviation (m²/m²)"),
        ("viridis", "viridis", "magma"), (8., 8., 2.5),
    ):
        colormap = plt.colormaps[cmap].with_extremes(bad="#edf0ee")
        image = ax.imshow(np.ma.masked_equal(data, nodata), origin="upper", extent=extent,
                          cmap=colormap, vmin=0, vmax=vmax, interpolation="nearest")
        ax.set_title(title, fontsize=11)
        fig.colorbar(image, ax=ax, shrink=0.76, label=label, extend="max" if cmap == "magma" else "neither")

    colors = ["#edf0ee", "#287751", "#ecbd57", "#7595bb", "#c35343"]
    labels = ["Excluded by input masks", "Retrieved, no flags", "LAI near prior boundary",
              "Wide candidate spread", "Poor spectral fit"]
    categories = np.ones_like(flags)
    categories[(flags & 16) != 0] = 2
    categories[(flags & 32) != 0] = 3
    categories[(flags & 8) != 0] = 4
    categories[layers[0] == nodata] = 0
    ax = axes.flat[-1]
    ax.imshow(categories, origin="upper", extent=extent, cmap=ListedColormap(colors),
              norm=BoundaryNorm(np.arange(-0.5, 5.5), len(colors)), interpolation="nearest")
    ax.set_title("Quality flags (highest-priority flag shown)", fontsize=11)
    ax.legend(handles=[Patch(facecolor=color, label=label) for color, label in zip(colors, labels)],
              loc="lower left", fontsize=8, framealpha=0.94)

    for ax in axes.flat:
        ax.set_xlabel("Easting (km)")
        ax.set_ylabel("Northing (km)")
    fig.savefig(output / "inform_lai_quality_overview.png", dpi=145)
    plt.close(fig)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    preview(parser.parse_args().output)
