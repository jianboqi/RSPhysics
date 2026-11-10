"""Compare original native, corrected native, and WASM smoke reports.

This initial suite covers two spectral modes and the original MODIS BRDF input
through both programs. It is not an exhaustive validation of 6SV2.1 options.
"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

from build import ROOT, verify_source

FLOAT = re.compile(r"(?<![\w.])[-+]?(?:\d+\.\d*|\.\d+)(?:[EeDd][-+]?\d+)?")

def report_numbers(report):
    angles, quantities = [], []
    for line in report.splitlines():
        values = [float(x.replace("D", "E").replace("d", "e")) for x in FLOAT.findall(line)]
        # mainlutaero.f:1554 prints four angular coordinates per LUT row.
        # These debug rows precede the framed scientific report.
        if len(values) == 4 and not FLOAT.sub("", line).strip():
            angles.extend(values)
        else:
            quantities.extend(values)
    return quantities, angles

def execute(binary_dir, inputs):
    result = {}
    for entry in ["sixsV2.1", "sixsV2.1-lutaero"]:
        executable = binary_dir.resolve()/(entry+(".exe" if os.name == "nt" else ""))
        for name, cards in inputs.items():
            with tempfile.TemporaryDirectory(prefix="sixsv21-verify-") as work:
                process = subprocess.run([str(executable)], input=cards.encode("ascii"), cwd=work,
                                         capture_output=True, timeout=120, check=True)
            report = process.stdout.decode("ascii")
            if "global gas. trans." not in report:
                raise RuntimeError(f"Incomplete {name}/{entry}: {process.stderr!r}")
            result[name+"/"+entry] = dict(report=report, stderr=process.stderr.decode("ascii"))
    return result

def compare(reference, actual):
    rows = {}
    if reference.keys() != actual.keys():
        raise AssertionError("Reference and actual cases differ")
    for name in reference:
        expected, expected_angles = report_numbers(reference[name]["report"])
        found, angles = report_numbers(actual[name]["report"])
        if len(expected) != len(found):
            raise AssertionError(f"{name}: numerical report field count {len(expected)} != {len(found)}")
        errors = [abs(x-y) for x,y in zip(expected,found)]
        maximum = max(errors, default=0.)
        # Five-decimal printed table fields can differ by one rounding unit.
        # Keep this smoke bound fixed; broader cases require quantity-specific
        # tolerances and higher-precision diagnostics rather than enlarging it.
        if maximum > 1.01e-5:
            raise AssertionError(f"{name}: maximum absolute report difference {maximum} > 1.01e-5")
        if len(angles) != len(expected_angles):
            raise AssertionError(f"{name}: angular coordinate count differs")
        angle_error = max((abs(x-y) for x,y in zip(angles,expected_angles)), default=0.)
        # Single-precision inverse-trigonometric LUT coordinates, in degrees.
        if angle_error > 1e-4:
            raise AssertionError(f"{name}: angular coordinate difference {angle_error} > 1e-4 deg")
        rows[name] = dict(numeric_fields=len(found), max_absolute_difference=maximum,
                          angular_coordinates=len(angles), max_angle_difference_deg=angle_error)
    return rows

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path)
    parser.add_argument("corrected", type=Path)
    parser.add_argument("--wasm-results", type=Path, required=True)
    parser.add_argument("--wasm-build", type=Path, default=ROOT/"target/sixs-wasm")
    parser.add_argument("--output", type=Path, default=ROOT/"target/sixs-smoke-validation.json")
    args = parser.parse_args()
    verify_source()
    inputs = json.loads((ROOT/"benchmarks/reference/6sv2.1/inputs.json").read_text())
    original = execute(args.original, inputs)
    corrected = execute(args.corrected, inputs)
    wasm = json.loads(args.wasm_results.read_text())
    report = dict(scope="Initial three-case / two-entrypoint build-portability smoke suite, not exhaustive model validation",
                  source_archive_sha256="42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1",
                  original_vs_corrected_native=compare(original,corrected),
                  original_vs_corrected_wasm=compare(original,wasm),
                  native_build=json.loads((args.corrected/"build-info.json").read_text()),
                  wasm_build=json.loads((args.wasm_build/"build-info.json").read_text()))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+"\n",encoding="utf8")
    print(json.dumps(report,indent=2))
