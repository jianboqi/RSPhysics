"""Time original 6SV2.1 subprocesses in isolated working directories.

Container startup, if any, is outside the timing region. Each sample includes
the executable launch, computation, and stdout capture. This measures end-to-end
throughput, not a profile of the Fortran numerical kernel.
"""
from __future__ import annotations
import argparse
import concurrent.futures
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]

def benchmark(binary_dir, inputs, repeat=7, count=32, workers=8):
    binary_dir = binary_dir.resolve()
    suffix = ".exe" if os.name == "nt" else ""
    def run(job):
        name, entry = job
        with tempfile.TemporaryDirectory(prefix="sixsv21-benchmark-") as directory:
            start = time.perf_counter()
            process = subprocess.run([str(binary_dir/(entry+suffix))], input=inputs[name].encode("ascii"),
                                     cwd=directory, capture_output=True, timeout=120)
            elapsed = time.perf_counter()-start
            if process.returncode or b"global gas. trans." not in process.stdout:
                raise RuntimeError(f"{name}/{entry} failed: {process.stderr!r}; {process.stdout[-1000:]!r}")
            return elapsed, len(process.stdout)
    records = {}
    for entry in ["sixsV2.1", "sixsV2.1-lutaero"]:
        for name in inputs:
            run((name, entry))
            runs = [run((name, entry)) for _ in range(repeat)]
            times = [t for t, _ in runs]
            records[name+"/"+entry] = dict(median_s=statistics.median(times), min_s=min(times), max_s=max(times),
                                         samples_s=times, report_bytes=runs[0][1])
            print(name, entry, records[name+"/"+entry]["median_s"], flush=True)
    batch_case = next(iter(inputs))
    jobs = [(batch_case, "sixsV2.1")]*count
    start = time.perf_counter()
    for job in jobs:
        run(job)
    serial = time.perf_counter()-start
    start = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        list(pool.map(run, jobs))
    parallel = time.perf_counter()-start
    build_info = binary_dir/"build-info.json"
    return dict(platform=platform.platform(), cpus_visible=os.cpu_count(),
                timing="Subprocess launch + unchanged Fortran calculation + report capture; excludes container startup",
                build=json.loads(build_info.read_text()) if build_info.exists() else None,
                cases=records, batch=dict(case=batch_case, count=count, workers=workers, serial_s=serial,
                                         parallel_s=parallel, speedup=serial/parallel))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary_dir", type=Path)
    parser.add_argument("--inputs", type=Path, default=ROOT/"benchmarks/reference/6sv2.1/inputs.json")
    parser.add_argument("--output", type=Path, default=ROOT/"target/6sv21-performance.json")
    parser.add_argument("--repeat", type=int, default=7)
    parser.add_argument("--count", type=int, default=32)
    parser.add_argument("--workers", type=int, default=8)
    args = parser.parse_args()
    if min(args.repeat, args.count, args.workers) < 1:
        parser.error("repeat, count, and workers must be positive")
    result = benchmark(args.binary_dir, json.loads(args.inputs.read_text()), args.repeat, args.count, args.workers)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2)+"\n", encoding="utf8")
    print(json.dumps(result["batch"]))
