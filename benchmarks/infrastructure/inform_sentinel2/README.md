# INFORM crown LAI from Sentinel-2B

This benchmark builds an INFORM lookup table (LUT) and retrieves single-crown leaf area index on the native Sentinel-2B L2A 20 m grid. Single-crown LAI is leaf area per unit of projected crown area. The benchmark uses Python bindings from this checkout, GDAL 3.8 or later with NumPy support, and Matplotlib for the optional preview.

## Inputs

Prepare a local Sentinel-2B L2A tile with `product-metadata.xml` (product XML), `granule-metadata.xml` (tile XML), and GeoTIFFs `original/{B03,B04,B05,B07,B8A,B11,SCL}.tif`. Export the bands and SCL from the product preserving their native grids and digital numbers. B03 and B04 are averaged over aligned 2 × 2 cells. The remaining bands and SCL are at 20 m. Surface reflectance is calculated using the scale and offsets from the product XML. [Sentinel-2 products](https://sentiwiki.copernicus.eu/web/s2-products)

Supply the ESA WorldCover 2021 v200 map tiles that cover the Sentinel tile. The example for T50TMK uses `N39E114` and `N39E117`. The workflow reprojects WorldCover using the modal class and retrieves LAI where SCL indicates vegetation and WorldCover indicates tree cover. [ESA WorldCover data access](https://esa-worldcover.org/en/data-access)

## Run

Run from the repository root. Fetch the pinned Sentinel-2B spectral response profile once, or use `--sensor-cache` with an existing profile cache.

```console
python -c "from rsphysics import sensors; sensors.fetch('sentinel-2b.msi')"
python benchmarks/infrastructure/inform_sentinel2/run.py lut --scene /data/S2B_L2A --output target/inform-s2 --workers 4
python benchmarks/infrastructure/inform_sentinel2/run.py scene --scene /data/S2B_L2A --output target/inform-s2 --workers 16 --worldcover /data/N39E114_Map.tif /data/N39E117_Map.tif
python benchmarks/infrastructure/inform_sentinel2/verify.py --scene /data/S2B_L2A --output target/inform-s2
python benchmarks/infrastructure/inform_sentinel2/preview.py --output target/inform-s2
```

The 8,192 LUT rows use Latin hypercube samples in eight leaf chlorophyll and soil spectrum groups. The benchmark uses a crown LAI interval of 0.5–8.0 m²/m², scene-mean sun and viewing angles, and a fixed diffuse fraction of 0.1. Terrain illumination is uncorrected. Tree cover changes after the WorldCover reference year can affect the mask.

## Outputs

- `inform_lai_20m.tif` stores best-fit crown LAI, mean crown LAI, population standard deviation of crown LAI, and best spectral reflectance RMSE in bands 1 through 4, respectively. The LAI statistics use the 16 lowest-RMSE candidates, with equal weights and `ddof=0`. This spread is conditional on the LUT sampling and priors. Ground accuracy requires separate validation. The Float32 NoData value is `-9999`.
- `inform_quality_20m.tif` contains a byte mask. Bit definitions are stored in the GeoTIFF and in `scene_summary.json`.
- `scene_summary.json` and `lut_timing.json` record scene statistics and stage timings. `verification.json` records raster checks and independent exhaustive LUT checks for sampled pixels. `inform_lai_quality_overview.png` is the optional overview.

| Bit value | Condition | LAI value |
| ---: | --- | --- |
| 0 | Retrieved without flags | Retained |
| 1 | SCL 0 or 1, or missing source data in any band | NoData |
| 2 | SCL class other than vegetation | NoData |
| 4 | WorldCover class other than tree cover | NoData |
| 8 | Best spectral RMSE exceeds 0.04 reflectance | Retained |
| 16 | Best LAI is within 5% of either LUT bound | Retained |
| 32 | Candidate LAI standard deviation exceeds 1.5 m²/m² | Retained |
| 64 | WorldCover has no data | NoData |
| 128 | A selected reflectance is outside [0, 1] | NoData |

The September 23, 2024 T50TMK run processed 121 windows of up to 512 × 512 pixels across 30,140,100 pixels. It retrieved 12,566,859 pixels, including 8,160,189 without flags. Image reading, retrieval and GeoTIFF writes took 74.9 seconds with 16 search workers on the test machine. LUT construction took 8.7 seconds with 4 workers. Total local processing including setup and mask alignment took 84.5 seconds. The total excludes downloads, preview rendering, and verification.
