"""Generate reference reports only with original Fortran executables.

Executable use is confined to reference validation, never product execution.
"""

import argparse, concurrent.futures, hashlib, importlib.util, json, subprocess, tempfile, zlib, os, re, math
from pathlib import Path
from build import ROOT
from rsphysics import sixs

spec = importlib.util.spec_from_file_location(
    "sixs_cases", ROOT / "benchmarks/reference/6sv2.1/cases.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--corrected", type=Path, required=True)
    a = p.parse_args()

    def one(case):
        parameters = case["parameters"]
        prepared = sixs.prepare(parameters)["prepared"]
        name = case["name"]
        binary = a.corrected.resolve() / (
            prepared["program"] + (".exe" if os.name == "nt" else "")
        )
        with tempfile.TemporaryDirectory(prefix="sixs-oracle-") as d:
            for filename, content in prepared["files"].items():
                (Path(d) / filename).write_text(content, encoding="ascii")
            result = subprocess.run(
                [str(binary)],
                input=prepared["input_cards"].encode(),
                cwd=d,
                capture_output=True,
                timeout=90,
            )
            report = result.stdout.decode("ascii")
            files = {f.name: f.read_text() for f in Path(d).iterdir() if f.is_file()}
        if result.returncode or "apparent reflectance" not in report:
            raise RuntimeError(
                f"{name}: {result.returncode} {result.stderr!r} {report[-500:]}"
            )
        if " NaN" in next(
            line for line in report.splitlines() if "apparent reflectance" in line
        ):
            raise RuntimeError(name + " non-finite original signal")
        quantities = {}
        apparent = []

        def numbers(text):
            values = []
            for word in text.split():
                try:
                    values.append(
                        float(word.strip("*,").replace("D", "E").replace("d", "e"))
                    )
                except ValueError:
                    pass
            return [v if math.isfinite(v) else None for v in values]

        for line in report.splitlines():
            text = line.strip().strip("*").strip()
            if "apparent reflectance" in text and len(numbers(text)) >= 2:
                apparent = numbers(text)
            if ":" in text:
                label, values = text.split(":", 1)
                v = numbers(values)
                if v:
                    quantities[
                        re.sub(r"[^a-z0-9]+", "_", label.strip().lower()).strip("_")
                    ] = v
        print(name, flush=True)
        return dict(
            **case,
            input_cards=prepared["input_cards"],
            report=report,
            files=files,
            expected={
                "apparent_reflectance": apparent[0],
                "apparent_radiance_w_m2_sr_um": apparent[1],
                "quantities": quantities,
            },
            oracle="compatibility-corrected-original",
        )

    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        rows = list(pool.map(one, module.cases()))
    for model in [8, 9, 10, 11]:
        saved = next(c for c in rows if c["name"] == f"mie-{model}")["files"][
            "aerosol.mie"
        ]
        for program in ["standard", "lutaero"]:
            rows.append(
                one(
                    {
                        "name": f"mie-reuse-{model}-{program}",
                        "parameters": {
                            "aerosol": {"model": 12, "mie_file": "saved.mie"},
                            "files": {"saved.mie": saved},
                            "program": program,
                        },
                    }
                )
            )
    output = ROOT / "benchmarks/reference/6sv2.1/full-reference.json.zlib"
    payload = json.dumps(
        {
            "suite": "sixsv21-direct-library-v1",
            "source_archive_sha256": "42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1",
            "oracle_build": json.loads((a.corrected / "build-info.json").read_text()),
            "cases": rows,
        },
        separators=(",", ":"),
    ).encode()
    output.write_bytes(zlib.compress(payload, 9))
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    (output.parent / "manifest.json").write_text(
        json.dumps(
            {
                "suite": "sixsv21-direct-library-v1",
                "cases": len(rows),
                "fixture": output.name,
                "sha256": digest,
            },
            indent=2,
        )
        + "\n"
    )
    print(
        len(rows),
        "cases",
        output.stat().st_size,
        "bytes",
        "sha256",
        digest,
    )
