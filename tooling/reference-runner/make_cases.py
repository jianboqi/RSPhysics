"""Deterministic inputs only. Expected spectra are generated exclusively by upstream R."""
import json
from pathlib import Path
import random

DEFAULT = dict(n=1.5, chlorophyll=40.0, carotenoids=8.0, anthocyanins=0.0,
               brown_pigments=0.0, water=0.01, dry_matter=0.008, incidence_angle_deg=40.0)
CASES = []


def case(name, **overrides):
    CASES.append(dict(name=name, parameters=DEFAULT | overrides))


case("default")
case("single-layer", n=1.0)
case("thick-leaf", n=5.0)
case("senescent", chlorophyll=3.0, carotenoids=1.5, anthocyanins=4.0, brown_pigments=1.0, water=0.003)
case("wet-leaf", water=0.05, dry_matter=0.02)
case("dry-leaf", water=0.0, dry_matter=0.012)
case("pigment-free", chlorophyll=0.0, carotenoids=0.0, anthocyanins=0.0)
case("zero-absorption", chlorophyll=0.0, carotenoids=0.0, water=0.0, dry_matter=0.0)
case("near-zero-absorption", chlorophyll=1e-7, carotenoids=1e-8, water=1e-10, dry_matter=1e-10)
case("grazing-cone", incidence_angle_deg=90.0)
case("narrow-cone", incidence_angle_deg=5.0)
case("minimum-cone", incidence_angle_deg=1.0)
case("intermediate-cone", incidence_angle_deg=60.0)
case("high-absorption", chlorophyll=200.0, carotenoids=100.0, anthocyanins=100.0, brown_pigments=5.0, water=0.1, dry_matter=0.05)
rng = random.Random(20261109)
for i in range(18):
    case(f"sample-{i:02d}", n=rng.uniform(1.0, 4.0), chlorophyll=rng.uniform(0, 100),
         carotenoids=rng.uniform(0, 30), anthocyanins=rng.uniform(0, 20), brown_pigments=rng.uniform(0, 1),
         water=rng.uniform(0.0001, 0.05), dry_matter=rng.uniform(0.0001, 0.025), incidence_angle_deg=rng.uniform(5, 90))

if __name__ == "__main__":
    root = Path(__file__).resolve().parents[2]
    destination = root / "benchmarks/reference/prospect-d/inputs.json"
    destination.write_text(json.dumps(CASES, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {len(CASES)} deterministic reference inputs")
