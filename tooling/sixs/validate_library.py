"""Validate direct C ABI libraries against the compressed original-Fortran suite."""

import argparse
import ctypes
import json
import math
from pathlib import Path
import re
import tempfile
import zlib
from build import ROOT


def numbers(text):
    result = []
    for token in text.split():
        try:
            result.append(float(token.strip("*,").replace("D", "E").replace("d", "e")))
        except ValueError:
            pass
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library-dir", type=Path, required=True)
    args = parser.parse_args()
    suite = json.loads(
        zlib.decompress(
            (ROOT / "benchmarks/reference/6sv2.1/full-reference.json.zlib").read_bytes()
        )
    )
    import sys

    extension = (
        ".dll"
        if sys.platform == "win32"
        else ".dylib" if sys.platform == "darwin" else ".so"
    )
    libraries = {}
    for program in ["sixsV2.1", "sixsV2.1-lutaero"]:
        library = ctypes.CDLL(str(args.library_dir.resolve() / (program + extension)))
        library.rsp_sixs_run.argtypes = [ctypes.c_char_p]
        library.rsp_sixs_run.restype = ctypes.c_int
        libraries[program] = library
    samples = 0
    for case in suite["cases"]:
        with tempfile.TemporaryDirectory(prefix="sixs-direct-") as directory:
            work = Path(directory)
            for name, content in case["parameters"].get("files", {}).items():
                (work / name).write_text(content, encoding="ascii")
            (work / "input.cards").write_text(case["input_cards"], encoding="ascii")
            program = (
                "sixsV2.1-lutaero"
                if case["parameters"].get("program") == "lutaero"
                else "sixsV2.1"
            )
            status = libraries[program].rsp_sixs_run(str(work).encode())
            assert status == 0, (case["name"], status)
            report = (work / "report.txt").read_text()
            quantities = {}
            for line in report.splitlines():
                text = line.strip().strip("*").strip()
                if ":" in text:
                    key, values = text.split(":", 1)
                    values = numbers(values)
                    if values:
                        quantities[
                            re.sub(r"[^a-z0-9]+", "_", key.strip().lower()).strip("_")
                        ] = values
                if "apparent reflectance" in text and len(numbers(text)) >= 2:
                    actual = numbers(text)
                    for key, value, tolerance in zip(
                        ["apparent_reflectance", "apparent_radiance_w_m2_sr_um"],
                        actual[:2],
                        [2e-6, 0.0011],
                    ):
                        expected = case["expected"][key]
                        assert math.isfinite(value) and abs(
                            value - expected
                        ) <= tolerance + 3e-6 * abs(expected), (
                            case["name"],
                            key,
                            value,
                            expected,
                        )
            assert set(quantities) == set(case["expected"]["quantities"]), case["name"]
            for key, values in case["expected"]["quantities"].items():
                assert len(quantities[key]) == len(values), (case["name"], key)
                for x, y in zip(quantities[key], values):
                    if y is not None:
                        assert math.isfinite(x) and abs(x - y) <= 2e-5 + 3e-5 * abs(
                            y
                        ), (case["name"], key, x, y)
                        samples += 1
            for name, expected in case["files"].items():
                actual = (work / name).read_text()
                a, b = numbers(actual), numbers(expected)
                assert len(a) == len(b), (case["name"], name)
                for x, y in zip(a, b):
                    if math.isfinite(y):
                        assert math.isfinite(x) and abs(x - y) <= 2e-5 + 3e-5 * abs(
                            y
                        ), (case["name"], name, x, y)
                        samples += 1
        print(case["name"], flush=True)
    print(
        f"{len(suite['cases'])} direct-library cases passed; {samples} numeric report/file values checked."
    )


if __name__ == "__main__":
    main()
