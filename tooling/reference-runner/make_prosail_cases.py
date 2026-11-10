import json
from pathlib import Path
import random

ROOT = Path(__file__).resolve().parents[2]
BASE = dict(n=1.5, chlorophyll=40.0, carotenoids=8.0, anthocyanins=0.0, brown_pigments=0.0,
            water=0.01, dry_matter=0.008, incidence_angle_deg=40.0,
            leaf_area_index=3.0, mean_leaf_angle_deg=60.0, hotspot=0.1, solar_zenith_deg=30.0,
            view_zenith_deg=0.0, relative_azimuth_deg=60.0, soil_brightness=1.0, soil_dry_fraction=0.5)
cases = []


def add(name, **changes):
    cases.append(dict(name=name, parameters=BASE | changes, expected_source="author_r_fourSAIL"))


add("default")
add("sparse-canopy", leaf_area_index=0.05)
add("dense-canopy", leaf_area_index=8.0)
add("no-hotspot", hotspot=0.0)
add("pure-hotspot", solar_zenith_deg=30.0, view_zenith_deg=30.0, relative_azimuth_deg=0.0)
add("oblique-view", view_zenith_deg=70.0, relative_azimuth_deg=120.0)
add("low-sun", solar_zenith_deg=70.0)
add("planophile", mean_leaf_angle_deg=10.0)
add("erectophile", mean_leaf_angle_deg=80.0)
add("wet-soil", soil_dry_fraction=0.0)
add("dry-soil", soil_dry_fraction=1.0)
add("black-background", soil_brightness=0.0)
add("high-chlorophyll", chlorophyll=100.0, water=0.04)
add("dry-leaf", water=0.0, dry_matter=0.012)
rng = random.Random(20261110)
for i in range(10):
    add(f"sample-{i:02d}", n=rng.uniform(1,3), chlorophyll=rng.uniform(1,90), carotenoids=rng.uniform(1,25),
        anthocyanins=rng.uniform(0,10), brown_pigments=rng.uniform(0,1), water=rng.uniform(.003,.04),
        dry_matter=rng.uniform(.002,.02), leaf_area_index=rng.uniform(.1,8), mean_leaf_angle_deg=rng.uniform(10,80),
        hotspot=rng.uniform(.001,.5), solar_zenith_deg=rng.uniform(0,70), view_zenith_deg=rng.uniform(0,70),
        relative_azimuth_deg=rng.uniform(0,180), soil_brightness=rng.uniform(.2,1), soil_dry_fraction=rng.uniform(0,1))
cases.append(dict(name="bare-soil-analytic-limit", parameters=BASE | {"leaf_area_index": 0.0}, expected_source="analytic_bare_soil_limit_not_fourSAIL"))
(ROOT / "benchmarks/reference/prosail-d/inputs.json").write_text(json.dumps(cases, indent=2) + "\n", encoding="utf-8")
print(f"{len(cases)-1} author-R cases + 1 explicitly analytical LAI=0 boundary")
