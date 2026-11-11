"""Verify the preserved scientific source and complete direct-library fixture."""

import hashlib
import importlib.util
import json
from pathlib import Path
import tomllib
import zlib

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "sixs_source", ROOT / "crates/models/atmosphere/sixs/fortran/build_tools/build.py"
)
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)
source.verify_source()
manifest = json.loads((HERE / "manifest.json").read_text())
data = (HERE / manifest["fixture"]).read_bytes()
assert hashlib.sha256(data).hexdigest() == manifest["sha256"]
suite = json.loads(zlib.decompress(data))
cases = suite["cases"]
assert len(cases) == manifest["cases"] == 330
assert len({c["name"] for c in cases}) == 330
assert {
    c["parameters"]["spectrum"]["band"] for c in cases if c["name"].startswith("band-")
} == set(range(2, 200))
assert all(c["oracle"] == "compatibility-corrected-original" for c in cases)
assert suite["oracle_build"]["compatibility_patches"] == source.COMPATIBILITY_PATCHES
metadata = tomllib.loads(
    (ROOT / "crates/models/atmosphere/sixs/model.toml").read_text(encoding="utf-8")
)
resource = ROOT / "crates/models/atmosphere/sixs/data/resources.json"
assert (
    metadata["data"]["hashes"]["standard_profiles_and_band_labels"]
    == "sha256:" + hashlib.sha256(resource.read_bytes()).hexdigest()
)
print(
    "6SV2.1: 163 preserved files, standard profiles, 198 bands, and 330 reference cases verified."
)
