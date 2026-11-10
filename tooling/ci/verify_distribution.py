"""Validate release versions and distribution metadata before publishing."""

import argparse
from email.parser import BytesParser
import os
from pathlib import Path
import tarfile
import tomllib
import zipfile


def check_metadata(data, filename, expected_version):
    metadata = BytesParser().parsebytes(data)
    if metadata["Name"] != "rsphysics" or metadata["Version"] != expected_version:
        raise ValueError(f"Unexpected name/version in {filename}")
    if not metadata.get("License-Expression") or not metadata.get_all("License-File"):
        raise ValueError(f"Missing component license metadata in {filename}")
    if not any(item.lower().startswith("numpy") for item in metadata.get_all("Requires-Dist", [])):
        raise ValueError(f"Missing NumPy dependency in {filename}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dist", type=Path)
    parser.add_argument("--expected-wheels", type=int)
    parser.add_argument("--tag", default=os.environ.get("RSP_RELEASE_TAG", ""))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    version = tomllib.loads((root / "bindings/python/pyproject.toml").read_text(encoding="utf-8"))["project"]["version"]
    rust_version = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    if version != rust_version:
        raise ValueError(f"Python version {version} differs from Rust version {rust_version}")
    if args.tag and args.tag != f"v{version}":
        raise ValueError(f"Release tag must be v{version}, received {args.tag}")
    if args.dist:
        wheels = sorted(args.dist.glob("*.whl"))
        archives = sorted(args.dist.glob("*.tar.gz"))
        if len(archives) != 1 or not wheels:
            raise ValueError("Expected one source archive and at least one wheel")
        if args.expected_wheels is not None and len(wheels) != args.expected_wheels:
            raise ValueError(f"Expected {args.expected_wheels} wheels, found {len(wheels)}")
        for wheel in wheels:
            with zipfile.ZipFile(wheel) as archive:
                paths = [name for name in archive.namelist() if name.endswith(".dist-info/METADATA")]
                if len(paths) != 1:
                    raise ValueError(f"Expected one METADATA record in {wheel}")
                check_metadata(archive.read(paths[0]), wheel.name, version)
        with tarfile.open(archives[0]) as archive:
            paths = [item for item in archive.getmembers() if item.name.count("/") == 1 and item.name.endswith("/PKG-INFO")]
            if len(paths) != 1:
                raise ValueError(f"Expected one root PKG-INFO in {archives[0]}")
            check_metadata(archive.extractfile(paths[0]).read(), archives[0].name, version)
        print(f"Verified {len(wheels)} wheels and one source archive")
    print(f"RSPhysics {version}: version metadata verified")
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a", encoding="utf-8") as stream:
            stream.write(f"version={version}\n")


if __name__ == "__main__":
    main()
