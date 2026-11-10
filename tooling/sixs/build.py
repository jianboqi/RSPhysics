"""Build and verify the preserved 6SV2.1 programs without editing their sources.

Native: python tooling/sixs/build.py native --fc gfortran
WASM (Flang/Emscripten environment): python tooling/sixs/build.py wasm
"""
from __future__ import annotations
import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
COMPONENT = ROOT / "components/6sv2.1"
SOURCE = COMPONENT / "upstream"

def verify_source():
    manifest = json.loads((COMPONENT / "source-manifest.json").read_text())
    expected = {item["path"] for item in manifest["files"]}
    actual = {p.relative_to(SOURCE).as_posix() for p in SOURCE.rglob("*") if p.is_file()}
    if actual != expected:
        raise RuntimeError("6SV2.1 source inventory differs from the distribution")
    for item in manifest["files"]:
        content = (SOURCE / item["path"]).read_bytes()
        if len(content) != item["size"] or hashlib.sha256(content).hexdigest() != item["sha256"]:
            raise RuntimeError(f"6SV2.1 source mismatch: {item['path']}")

def object_names():
    makefile = (SOURCE / "Makefile").read_text()
    return re.search(r"OBJECTS0\s*=\s*(.*?)\n\s*\n", makefile, re.S)[1].replace("\\", "").split()

def prepare_sources(output, compatibility=True):
    source = output / "source"
    source.mkdir(parents=True, exist_ok=True)
    for file in SOURCE.iterdir():
        if file.is_file():
            shutil.copyfile(file, source/file.name)
    if compatibility:
        for name in ["OS", "OSSURF", "ISO", "OSPOL"]:
            path = source/(name+".f")
            content = path.read_bytes()
            anchor = b"      snt=nt"
            assert content.count(anchor) == 1, name
            if name == "OSPOL":
                initialization = b"      do k=0,2\n      in(k,0)=0.\n      qn(k,0)=0.\n      un(k,0)=0.\n      enddo\n"
            else:
                initialization = b"      in(0)=0.d0\n      inm1(0)=0.d0\n      inm2(0)=0.d0\n"
            path.write_bytes(content.replace(anchor, initialization+anchor))
    return source

def build(kind, output, fc=None, jobs=8, compatibility=True):
    verify_source()
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source = prepare_sources(output, compatibility)
    fc = fc or os.environ.get("FC") or ("gfortran" if kind == "native" else "/opt/flang/host/bin/flang")
    # FC is an executable path; optional flags belong in FFLAGS.
    compiler_flags = shlex.split(os.environ.get("FFLAGS", ""))
    flags = ["-O2", "-ffixed-line-length-132"] if kind == "native" else ["--target=wasm32-unknown-emscripten", "-O2", "-ffixed-line-length=132"]
    if kind == "native":
        flags += ["-std=legacy", "-fallow-argument-mismatch"]
    flags += compiler_flags
    objects = object_names()
    names = [Path(x).stem for x in objects] + ["main", "mainlutaero"]
    def compile_one(name):
        process = subprocess.run([fc, *flags, "-I"+str(source), "-c", str(source / (name+".f")), "-o", str(output / (name+".o"))], capture_output=True)
        (output/(name+".log")).write_bytes(process.stdout+process.stderr)
        if process.returncode:
            raise RuntimeError(f"{name}.f compilation failed:\n"+process.stderr.decode(errors="replace")[-5000:])
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        list(pool.map(compile_one, names))
    for entry, main in [("sixsV2.1", "main"), ("sixsV2.1-lutaero", "mainlutaero")]:
        inputs = [str(output / (main+".o")), *[str(output/x) for x in objects]]
        if kind == "native":
            executable = output / (entry + (".exe" if os.name == "nt" else ""))
            runtime_flags = ["-static"] if os.name == "nt" else ["-static-libgfortran", "-static-libgcc"]
            if os.name == "nt":
                runtime_flags += ["-Wl,--stack,16777216"]
            subprocess.run([fc, *inputs, "-O2", *runtime_flags, "-lm", "-o", str(executable)], check=True)
        else:
            runtime = os.environ.get("FLANG_RUNTIME", "/opt/flang/wasm/lib/libFortranRuntime.a")
            subprocess.run(["emcc", "-O2", *inputs, runtime, "-o", str(output / (entry+".mjs")),
                "-sMODULARIZE=1", "-sEXPORT_ES6=1", "-sINVOKE_RUN=0", "-sEXPORTED_RUNTIME_METHODS=callMain,FS",
                "-sEXPORTED_FUNCTIONS=_main", "-sEXIT_RUNTIME=1", "-sALLOW_MEMORY_GROWTH=1", "-sSTACK_SIZE=16777216"], check=True)
    recipe = dict(source_archive_sha256="42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1", kind=kind,
                  compiler=subprocess.check_output([fc, "--version"], text=True).splitlines()[0], flags=flags,
                  programs=["sixsV2.1", "sixsV2.1-lutaero"],
                  compatibility_patches=["initialize-unused-zero-direction"] if compatibility else [])
    (output/"build-info.json").write_text(json.dumps(recipe, indent=2)+"\n", encoding="utf8")
    return output

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=["verify", "native", "wasm"])
    parser.add_argument("--output", type=Path)
    parser.add_argument("--fc")
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--original", action="store_true", help="Build the byte-exact original without compatibility fixes")
    args = parser.parse_args()
    if args.kind == "verify":
        verify_source()
        print("All original 6SV2.1 source hashes match.")
    else:
        print(build(args.kind, args.output or ROOT/"target"/("sixs-"+args.kind), args.fc, args.jobs, not args.original))
