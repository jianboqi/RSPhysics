"""Audit the installed Python wheel against durable author-R golden spectra."""
import hashlib
import json
from pathlib import Path
import platform
import sys
import argparse

import numpy as np
import rsphysics as rsp

ROOT = Path(__file__).resolve().parents[2]
SUITE = ROOT / "benchmarks/reference/prospect-d/expected.json"


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model',choices=['prospect-d','prosail-d'],default='prospect-d')
    args=parser.parse_args()
    global SUITE
    SUITE=ROOT / f'benchmarks/reference/{args.model}/expected.json'
    suite = json.loads(SUITE.read_text(encoding="utf-8"))
    canopy=args.model=='prosail-d'
    names = rsp.vegetation.PROSAIL_D_PARAMETERS if canopy else rsp.vegetation.PROSPECT_D_PARAMETERS
    single_fn=rsp.vegetation.prosail_d if canopy else rsp.vegetation.prospect_d
    batch_fn=rsp.vegetation.prosail_d_batch if canopy else rsp.vegetation.prospect_d_batch
    matrix = np.array([[case["parameters"][name] for name in names] for case in suite["cases"]])
    batch = batch_fn(matrix)["result"]
    report = dict(suite=suite["suite"], upstream_commit=suite.get("canopy_upstream_commit",suite.get("upstream_commit")),
                  oracle_sha256=hashlib.sha256(SUITE.read_bytes()).hexdigest(),
                  model_implementation_version=rsp.describe('vegetation.'+args.model)["metadata"]["implementation_version"],
                  system=platform.platform(), python=sys.version.split()[0], numpy=np.__version__,
                  reference_interpreter=suite["interpreter"], reference_expint=suite["expint_version"],
                  tolerance=suite["tolerance"], samples_compared=0, max_absolute_error=0.0,
                  max_normalized_error=0.0, cases=[])
    for i, case in enumerate(suite["cases"]):
        single = single_fn(**case["parameters"])["result"]
        metrics = {"name": case["name"]}
        if canopy:metrics['expected_source']=case['expected_source']
        for key in (("brf", "hemispherical_reflectance") if canopy else ("reflectance", "transmittance")):
            expected = np.asarray(case[key])
            actual = batch[key][i]
            np.testing.assert_array_equal(actual, single[key])
            error = np.abs(actual - expected)
            allowed = suite["tolerance"]["absolute"] + suite["tolerance"]["relative"] * np.abs(expected)
            if not np.all(error <= allowed):
                raise ValueError(f"reference comparison failed: {case['name']} {key}")
            maximum = float(error.max())
            metrics[key + "_max_absolute_error"] = maximum
            report["max_absolute_error"] = max(report["max_absolute_error"], maximum)
            report["max_normalized_error"] = max(report["max_normalized_error"], float((error / allowed).max()))
            report["samples_compared"] += actual.size
        report["cases"].append(metrics)
    report["status"] = "passed"
    destination = ROOT / f"benchmarks/reference/{args.model}/validation-report.json"
    destination.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"{report['samples_compared']} samples passed; max abs={report['max_absolute_error']:.6e}, max normalized={report['max_normalized_error']:.6f}")


if __name__ == "__main__":
    main()
