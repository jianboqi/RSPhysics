"""Run offline source, resource, and catalog integrity checks used by CI."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
CHECKS = [
    "benchmarks/reference/foursail-tir/verify.py",
    'benchmarks/reference/verify_extended.py',
    'benchmarks/reference/fluspect-cx/verify.py',
    'benchmarks/reference/i2em/verify.py',
    'benchmarks/reference/sort/verify.py',
    'tooling/check-models/check_models.py',
    'tooling/sensor-data/verify.py',
    'tooling/smac-data/verify.py',
    'benchmarks/reference/basic-physics/verify.py',
    'benchmarks/reference/prospect-d/verify_upstream.py',
    'benchmarks/reference/prosail-d/verify_upstream.py',
    'benchmarks/reference/ross-li/verify_upstream.py',
    'benchmarks/reference/microwave/verify_upstream.py',
]

if __name__ == '__main__':
    for check in CHECKS:
        print(f'Checking {check}', flush=True)
        subprocess.run([sys.executable, str(ROOT / check)], cwd=ROOT, check=True)
