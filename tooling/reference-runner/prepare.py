"""Recreate the disposable official-reference environment in one directory."""
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / ".reference-cache/prospect-d"


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    for name in ("package.json", "package-lock.json"):
        shutil.copyfile(ROOT / "tooling/reference-runner" / name, CACHE / name)
    environment = os.environ.copy()
    environment["npm_config_cache"] = str(CACHE / "npm-cache")
    npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
    if npm is None:
        raise RuntimeError("Node.js/npm is needed only to regenerate original-reference spectra")
    subprocess.run([npm, "ci", "--prefix", str(CACHE), "--ignore-scripts", "--no-audit", "--no-fund"], cwd=ROOT, env=environment, check=True)
    subprocess.run([sys.executable, str(ROOT / "benchmarks/reference/prospect-d/fetch_upstream.py")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, str(ROOT / "benchmarks/reference/prospect-d/verify_upstream.py"), "--require-cache"], cwd=ROOT, check=True)
    print("Reference environment ready; run: node tooling/reference-runner/generate.mjs")


if __name__ == "__main__":
    main()
