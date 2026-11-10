"""Check manifests using the actual Rust parser and compare them with Cargo/catalog.

Python 3.11+ standard library only. Run from any directory. Does not execute models.
"""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def cargo_json(*args: str):
    result = subprocess.run(
        ["cargo", *args], cwd=ROOT, text=True, encoding="utf-8", capture_output=True,
        check=True,
    )
    return json.loads(result.stdout)


def check() -> None:
    metadata = cargo_json("metadata", "--format-version", "1", "--no-deps", "--locked")
    members = set(metadata["workspace_members"])
    packages = {Path(p["manifest_path"]).resolve().parent: p
                for p in metadata["packages"] if p["id"] in members}
    manifests = sorted((ROOT / "crates" / "models").rglob("model.toml"))
    manifests += sorted((ROOT / "templates").rglob("model.toml"))
    official = {}
    declared = {}
    ids = set()
    for path in manifests:
        model = cargo_json("run", "--quiet", "--locked", "-p", "rsphysics-cli", "--",
                           "manifest", "validate", str(path))
        package = packages.get(path.parent.resolve())
        if package is None:
            raise ValueError(f"not a workspace member: {path.relative_to(ROOT)}")
        if (model["source_crate"], model["implementation_version"]) != (package["name"], package["version"]):
            raise ValueError(f"crate name/version mismatch: {path.relative_to(ROOT)}")
        if model["id"] in ids:
            raise ValueError(f"duplicate model ID: {model['id']}")
        ids.add(model["id"])
        if path.is_relative_to(ROOT / "crates" / "models"):
            if any(word in model["license"].upper() for word in ("UNSET", "TO_BE", "REPLACE")):
                raise ValueError(f"unresolved license: {path.relative_to(ROOT)}")
            official[model["id"]] = path.relative_to(ROOT).as_posix()
            declared[model["id"]] = model
        print(f"OK {model['id']} ({path.relative_to(ROOT).as_posix()})")
    catalog_path = ROOT / "data" / "model-catalog" / "model-catalog.toml"
    with catalog_path.open("rb") as stream:
        catalog = tomllib.load(stream)
    if set(catalog) != {"models"}:
        raise ValueError("catalog only accepts a models array")
    intended = {}
    for entry in catalog["models"]:
        if set(entry) != {"id", "manifest"} or entry["id"] in intended:
            raise ValueError("invalid or duplicate catalog entry")
        intended[entry["id"]] = entry["manifest"]
    if intended != official:
        raise ValueError("catalog inventory differs from scientific model manifests")
    compiled = cargo_json("run", "--quiet", "--locked", "-p", "rsphysics-cli", "--", "models", "list")
    if {entry["metadata"]["id"] for entry in compiled} != set(intended):
        raise ValueError("compiled catalog differs from inventory; registration is incomplete")
    for entry in compiled:
        if entry["metadata"] != declared[entry["metadata"]["id"]]:
            raise ValueError("compiled model metadata differs from its source manifest; rebuild or fix registration")
    print(f"OK {len(manifests)} manifest(s), {len(official)} scientific model(s)")


if __name__ == "__main__":
    try:
        check()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            print(error.stderr, file=sys.stderr)
        raise SystemExit(1)
