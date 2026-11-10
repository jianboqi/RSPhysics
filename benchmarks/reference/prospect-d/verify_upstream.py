"""Offline verification of the pinned PROSPECT reference materials."""
import hashlib
import json
from pathlib import Path
import argparse

ROOT = Path(__file__).resolve().parent
CACHE = ROOT.parents[2] / ".reference-cache" / "prospect-d"
COMMIT = "4649d5006ddf6bce609951765a88ba2498fcd8ab"


def verify(require_cache=False):
    provenance = json.loads((ROOT / "upstream-provenance.json").read_text(encoding="utf-8"))
    if provenance["commit"] != COMMIT or provenance["tag"] != "v1.7.0":
        raise ValueError("reference version changed without updating the verification contract")
    for name, record in provenance["files"].items():
        source = (CACHE / "upstream" / name).resolve()
        if not source.is_relative_to((CACHE / "upstream").resolve()):
            raise ValueError("reference file escaped upstream directory")
        if not source.exists() and not require_cache:
            continue
        data = source.read_bytes()
        if len(data) != record["bytes"] or hashlib.sha256(data).hexdigest() != record["sha256"]:
            raise ValueError(f"reference material changed: {name}")
        expected = f"https://raw.githubusercontent.com/jbferet/prospect/{COMMIT}/{name}"
        if record["url"] != expected:
            raise ValueError(f"reference URL differs from pinned version: {name}")
    description = CACHE / "upstream" / "DESCRIPTION"
    if description.exists():
        text = description.read_text(encoding="utf-8")
        if "Version: 1.7.0" not in text or "License: MIT" not in text:
            raise ValueError("upstream package identity/license declaration does not match intake")
    coefficients = ROOT.parents[2] / "crates/models/vegetation/prospect-d/data/coefficients.tsv"
    expected_hash = provenance["files"]["data-raw/dataSpec_PRO.txt"]["sha256"]
    if hashlib.sha256(coefficients.read_bytes()).hexdigest() != expected_hash:
        raise ValueError("runtime coefficient bytes differ from pinned upstream")
    reference = json.loads((ROOT / "expected.json").read_text(encoding="utf-8"))
    if reference["coefficients_sha256"] != expected_hash or reference["upstream_commit"] != COMMIT:
        raise ValueError("regression fixture provenance mismatch")
    inputs = json.loads((ROOT / "inputs.json").read_text(encoding="utf-8"))
    if [{"name": case["name"], "parameters": case["parameters"]} for case in reference["cases"]] != inputs:
        raise ValueError("reference cases differ from input specification")
    if reference["wavelengths_nm"] != list(range(400, 2501)) or len(reference["cases"]) != 32:
        raise ValueError("reference suite axis/case count mismatch")
    import math
    for case in reference["cases"]:
        for quantity in ("reflectance", "transmittance"):
            if len(case[quantity]) != 2101 or not all(math.isfinite(x) for x in case[quantity]):
                raise ValueError("invalid reference spectrum")
    print("OK pinned provenance, runtime coefficients and 32 offline reference spectra (cache optional)")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--require-cache", action="store_true")
    verify(parser.parse_args().require_cache)
