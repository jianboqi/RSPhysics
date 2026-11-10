# Official spectral response asset preparation

`source-manifest.json` pins the original provider URLs, platform/version,
SHA-256, source units/parser and supported bands. Runtime registry and allowed
derived assets live in `bindings/python/python/rsphysics/data/sensors`.

- Copernicus S2 MSI v5: A/B/C instrument averages; original or processed response
  samples are **not distributed**. Explicit fetch/import installs them in the
  user's local cache. SentiWiki terms are linked in the infrastructure guide.
- USGS: Landsat 4/5 TM, 7 ETM+, 8/9 OLI and 8/9 TIRS. The TM/ETM+ XLSX files
  provide reflective subsets; no thermal curves are invented. USGS tables are
  provided as U.S. government scientific information; retain original source
  attribution, and do not infer a license for third-party site content.
- NASA OBPG: Terra/Aqua MODIS, 16 reflective responses with provider wavelength
  labels. They are not all MODIS bands or MCST single-detector curves. The
  unused MCST candidate downloads are not shipped.

The standard-library parser only reads XLSX XML or OBPG text; no macros/formulas
execute. Preparing assets converts um to nm where declared, clips negative
measured response noise to zero with counts/minima, sorts out-of-order source
rows with a flag, rejects duplicates, and trims only exact-zero tails retaining
one shoulder. L4 B5 contains out-of-order provider rows; L7 ETM+ and L8 OLI
contain 5 and 43 negative samples respectively. No response resampling or
Gaussian approximation is performed. Numerical integration stays in Rust.

To regenerate the full pinned registry from the official originals:

```sh
python tooling/sensor-data/fetch.py --include-esa
python tooling/sensor-data/prepare.py
python tooling/sensor-data/verify.py
```

Downloads are isolated under `.reference-cache/sensors/upstream`. The script
fails if upstream bytes differ. It writes redistributable profiles into package
resources and ESA profiles only into ignored `target/sensor-runtime-cache`.
Do not copy the ESA profiles or original workbooks into package data. Normal
runtime loading and offline tests do not require the reference-download cache.
The runtime ESA cache is a separate user-managed cache for offline use.

See [Python infrastructure](../../docs/python-infrastructure.md) for API,
scientific conventions, cache paths, provider citations and validation limits.

Remove temporary source downloads with `python tooling/reference-runner/clean.py --model sensors --remove`. This preserves user runtime caches and bundled distributable resources.
