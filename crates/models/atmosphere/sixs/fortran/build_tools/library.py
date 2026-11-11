"""Build a direct C ABI around the original 6SV2.1 programs.

Scientific statements and coefficients are preserved. Build copies turn the
program into a subroutine, replace STOP with RETURN, check input I/O, and scope
auxiliary paths to a caller-owned directory. Original executables remain oracles.
"""

from __future__ import annotations
import argparse, concurrent.futures, hashlib, json, os, re, shlex, shutil, subprocess
from pathlib import Path
from build import COMPATIBILITY_PATCHES, object_names, prepare_sources, verify_source

HERE = Path(__file__).resolve().parent


def fixed(code, label=""):
    result = []
    while len(code) > 120:
        cuts = [i + 1 for i, c in enumerate(code[:118]) if c in " ,"]
        if not cuts:
            raise RuntimeError("Cannot safely wrap fixed-form statement: " + code)
        cut = cuts[-1]
        result.append(code[:cut])
        code = code[cut:]
    result.append(code)
    return (
        "\n".join(
            (label.rjust(5) + " " if i == 0 else "     &") + line
            for i, line in enumerate(result)
        )
        + "\n"
    )


def records(text):
    group = []
    for physical in text.splitlines(keepends=True):
        expanded = physical.expandtabs(8).rstrip("\r\n")
        if not expanded.strip() or expanded[0] in "cC*!":
            if group:
                yield group
                group = []
            yield [physical]
            continue
        continuation = len(expanded) > 5 and expanded[5] not in " 0"
        if not continuation and group:
            yield group
            group = []
        group.append(physical)
    if group:
        yield group


def control_end(code, start):
    depth = 0
    quote = None
    for i in range(start, len(code)):
        char = code[i]
        if quote:
            if char == quote:
                quote = None
        elif char in "'\"":
            quote = char
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if not depth:
                return i
    raise RuntimeError("Unterminated I/O statement: " + code)


def adapt(source):
    changes = {}
    # Never reconnect the host's preconnected stdin/stdout units. In particular,
    # dup2-based output capture can otherwise invalidate the Fortran runtime's
    # handles. Keep continuation records intact while relocating explicit I/O.
    for path in source.glob("*.f"):
        lines = []
        for line in path.read_text().splitlines(keepends=True):
            expanded = line.expandtabs(8)
            if expanded.strip() and expanded[0] not in "cC*!":
                expanded = re.sub(
                    r"(?i)\b(read\s*\(\s*)5(?=\s*[,\)])", r"\g<1>45", expanded
                )
                expanded = re.sub(
                    r"(?i)\b(write\s*\(\s*)6(?=\s*[,\)])", r"\g<1>46", expanded
                )
                expanded = re.sub(r"(?i)\biwr\s*=\s*6\b", "iwr=46", expanded)
                expanded = re.sub(r"(?i)\biread\s*=\s*5\b", "iread=45", expanded)
                expanded = re.sub(r"(?i)\bprint\s*\*\s*,", "write(46,*) ", expanded)
                line = expanded
            lines.append(line)
        path.write_text("".join(lines), encoding="ascii", newline="\n")
    # The original scratch capacity is 14 arrays of ten million doubles.
    # Allocate only through the order already selected by the original series
    # criterion. Keep its nser limit, lower bounds, equations and tolerances.
    # Local allocatables are released on every return, including allocation errors.
    path = source / "MIE.f"
    text = path.read_text()
    arrays = [
        "xj",
        "xy",
        "Rn",
        "IDnY",
        "RDnX",
        "RDnY",
        "IGnX",
        "RGnX",
        "RAn",
        "IAn",
        "RBn",
        "IBn",
        "TAUn",
        "PIn",
    ]
    for line in text.splitlines():
        if "double precision" in line and "nser)" in line:
            names = re.findall(r"(\w+)\((?:0|-1):nser\)", line)
            assert names
            text = text.replace(
                line,
                fixed(
                    "double precision, allocatable :: "
                    + ",".join(name + "(:)" for name in names)
                ).rstrip(),
            )
    allocation = (
        "allocate("
        + ",".join(
            name + "(" + ("-1" if name == "xy" else "0") + ":mu+1)" for name in arrays
        )
        + ",stat=rsp_ios,errmsg=rsp_message)"
    )
    anchor = "      Rn(mu)=0.D+00"
    assert text.count(anchor) == 1
    text = text.replace(
        anchor,
        fixed(allocation)
        + fixed("if(rsp_ios.ne.0) then")
        + fixed("rsp_failed=.true.")
        + fixed("return")
        + fixed("endif")
        + anchor,
    )
    path.write_text(text, encoding="ascii", newline="\n")
    for name in [
        "main.f",
        "mainlutaero.f",
        "AEROSO.f",
        "AKTOOL.f",
        "MIE.f",
        "IAPIBRDF.f",
        "IAPIALBE.f",
    ]:
        path = source / name
        out = []
        count = 0
        for record in records(path.read_text()):
            first = record[0].expandtabs(8).rstrip("\r\n")
            if not first.strip() or first[0] in "cC*!":
                out.extend(record)
                continue
            code = "".join(
                x.expandtabs(8).rstrip("\r\n")[6:132] for x in record
            ).strip()
            label = first[:5].strip()
            if re.match(r"program\s+", code, re.I):
                out.append(fixed("subroutine rsp_main"))
                out.append(fixed("use rsp_io"))
                # PROGRAM locals have program lifetime. Retain that storage
                # duration after conversion, and include it in the reset image.
                out.append(fixed("save"))
                count += 1
                continue
            if (
                name in ["AEROSO.f", "MIE.f", "IAPIBRDF.f", "IAPIALBE.f"]
                and re.match(r"subroutine\s+", code, re.I)
            ) or (
                name == "AKTOOL.f"
                and re.match(r"subroutine\s+(akbrdf|dakg)\b", code, re.I)
            ):
                out.extend(record)
                out.append(fixed("use rsp_io"))
                continue
            if re.search(r"\bstop\b", code, re.I):
                if name not in ["main.f", "mainlutaero.f"]:
                    out.append(fixed("rsp_failed=.true.", label))
                    message = "Original " + name + " rejected its input"
                    out.append(fixed("rsp_message='" + message + "'"))
                    out.append(fixed("return"))
                    count += 1
                    continue
                out.append(
                    fixed(re.sub(r"\bstop\b", "return", code, flags=re.I), label)
                )
                count += 1
                continue
            match = re.search(r"\b(read|open)\s*\(", code, re.I)
            if match:
                end = control_end(code, code.index("(", match.start()))
                if match[1].lower() == "open":
                    code = re.sub(
                        r"\bfile\s*=\s*('[^']*'|\w+)",
                        r"file=rsp_path(\1)",
                        code,
                        flags=re.I,
                    )
                    end = control_end(code, code.index("(", match.start()))
                code = code[:end] + ",iostat=rsp_ios,iomsg=rsp_message" + code[end:]
                out.append(fixed("rsp_ios=0", label))
                out.append(fixed(code))
                out.append(fixed("if(rsp_ios.ne.0) then"))
                out.append(fixed("rsp_failed=.true."))
                out.append(fixed("return"))
                out.append(fixed("endif"))
                if match[1].lower() == "read":
                    target = (
                        code[end + len(",iostat=rsp_ios,iomsg=rsp_message") + 1 :]
                        .strip()
                        .lower()
                    )
                    bounds = {"num_z": (1, 50), "irsunph": (2, 50), "nbmu": (3, 1001)}
                    if target == "rmin,rmax,icp":
                        target = "icp"
                        bounds[target] = (1, 4)
                    if target in bounds:
                        lo, hi = bounds[target]
                        condition = f"{target}.lt.{lo}.or.{target}.gt.{hi}"
                        if target == "nbmu":
                            condition += ".or.mod(nbmu,2).eq.0"
                        out.append(fixed(f"if({condition}) then"))
                        out.append(fixed("rsp_failed=.true."))
                        out.append(
                            fixed("rsp_message='Original array dimension exceeded'")
                        )
                        out.append(fixed("return"))
                        out.append(fixed("endif"))
                count += 1
                continue
            if re.match(r"call\s+mie\b", code, re.I):
                out.append(fixed("rsp_mie_used=.true."))
            out.extend(record)
            if name in ["main.f", "mainlutaero.f", "AEROSO.f", "MIE.f"] and re.match(
                r"call\s+(aeroso|akbrdf|iapibrdf|iapialbe|mie|exscphase)\b", code, re.I
            ):
                out.append(fixed("if(rsp_failed) return"))
        path.write_text("".join(out), encoding="ascii", newline="\n")
        changes[name] = count
    return changes


def checked(args, log, cwd=None):
    p = subprocess.run([str(x) for x in args], cwd=cwd, capture_output=True)
    Path(log).write_bytes(p.stdout + p.stderr)
    if p.returncode:
        raise RuntimeError(
            " ".join(map(str, args[:3]))
            + " failed:\n"
            + p.stderr.decode(errors="replace")[-6000:]
        )


def native_state(objects, fc, output):
    """Snapshot COMMON blocks and every writable original-object data section.

    This includes compiler-created saved locals, not Fortran runtime internals.
    Link-time-added symbols expose section starts without changing any contents.
    """
    directory = Path(shutil.which(fc) or fc).resolve().parent
    suffix = ".exe" if os.name == "nt" else ""

    def tool(name):
        local = directory / (name + suffix)
        return str(local) if local.exists() else name

    segments = []
    commons = {}
    for obj in objects:
        symbols = subprocess.check_output(
            [tool("nm"), "-S", "--defined-only", "--format=posix", str(obj)], text=True
        )
        for line in symbols.splitlines():
            parts = line.split()
            if len(parts) >= 3 and parts[1] == "C":
                commons[parts[0]] = max(commons.get(parts[0], 0), int(parts[2], 16))
        sections = subprocess.check_output([tool("objdump"), "-h", str(obj)], text=True)
        args = [tool("objcopy")]
        for section, size in re.findall(
            r"^\s*\d+\s+(\.(?:data|bss)(?:[.$][\w.$-]+)?)\s+([0-9a-fA-F]+)\s",
            sections,
            re.M,
        ):
            size = int(size, 16)
            if size:
                symbol = "rsp_saved_" + re.sub(r"\W", "_", obj.stem + "_" + section)
                args += ["--add-symbol", f"{symbol}={section}:0,global"]
                segments.append(
                    (
                        symbol,
                        size,
                        int(section.startswith(".bss")),
                        int(obj.stem == "MIE"),
                    )
                )
        if len(args) > 1:
            checked([*args, obj], output / (obj.stem + "-state.log"))
    segments += [(name, size, 1, 0) for name, size in sorted(commons.items())]
    declarations = "\n".join(
        f'extern unsigned char s{i}[] __asm__("{name}");'
        for i, (name, *_) in enumerate(segments)
    )
    entries = ",\n".join(
        f"{{s{i},{size},{zero},{mie}}}"
        for i, (_, size, zero, mie) in enumerate(segments)
    )
    code = (
        (HERE / "state.c.in")
        .read_text()
        .replace("@DECLARATIONS@", declarations)
        .replace("@SEGMENTS@", entries)
    )
    (output / "state.c").write_text(code, encoding="ascii")
    return dict(
        segments=len(segments),
        bytes=sum(size for _, size, *_ in segments),
        common_blocks=len(commons),
        saved_initialized_bytes=sum(size for _, size, zero, _ in segments if not zero),
    )


def build_library(kind, output, fc=None, jobs=8):
    verify_source()
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source = prepare_sources(output)
    changes = adapt(source)
    fc = (
        fc
        or os.environ.get("FC")
        or ("gfortran" if kind == "native" else "/opt/flang/host/bin/flang")
    )
    flags = (
        [
            "-O2",
            "-ffixed-line-length-132",
            "-std=legacy",
            "-fallow-argument-mismatch",
            "-ffp-contract=off",
            "-fPIC",
        ]
        if kind == "native"
        else ["--target=wasm32-unknown-emscripten", "-O2", "-ffixed-line-length=132"]
    )
    flags += shlex.split(os.environ.get("FFLAGS", ""))
    checked(
        [fc, *flags, "-c", HERE / "io.f90", "-J" + str(output), "-o", output / "io.o"],
        output / "io.log",
    )
    shared = [Path(x).stem for x in object_names()]

    def compile_one(name):
        checked(
            [
                fc,
                *flags,
                "-I" + str(source),
                "-I" + str(output),
                "-c",
                source / (name + ".f"),
                "-o",
                output / (name + ".o"),
            ],
            output / (name + ".log"),
        )

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        list(pool.map(compile_one, shared + ["main", "mainlutaero"]))
    checked(
        [
            fc,
            *flags,
            "-I" + str(output),
            "-c",
            HERE / "entry.f90",
            "-o",
            output / "entry.o",
        ],
        output / "entry.log",
    )
    info = {}
    for entry, main in [("sixsV2.1", "main"), ("sixsV2.1-lutaero", "mainlutaero")]:
        # Separate directories prevent duplicate saved-section symbols in the two DLLs.
        objects = [output / (x + ".o") for x in shared] + [output / (main + ".o")]
        link = [output / "io.o", output / "entry.o", *objects]
        if kind == "native":
            # Restore objects after each link: objcopy adds section-start symbols.
            backups = {p: p.read_bytes() for p in objects}
            if os.sys.platform == "darwin":
                # Mach-O has no GNU objcopy section aliases. The Rust adapter
                # loads a unique library image per call on macOS instead.
                (output / "state.c").write_text(
                    "extern int rsp_fortran_run(const char *);\nint rsp_sixs_run(const char *p) { return rsp_fortran_run(p); }\n"
                )
                info[entry] = {"reset": "fresh library image per calculation"}
            else:
                info[entry] = native_state(objects, fc, output)
            directory = Path(shutil.which(fc) or fc).resolve().parent
            cc = directory / ("gcc.exe" if os.name == "nt" else "gcc")
            if not cc.exists():
                cc = "gcc"
            checked(
                [
                    cc,
                    "-O2",
                    "-fPIC",
                    "-std=c11",
                    "-c",
                    output / "state.c",
                    "-o",
                    output / "state.o",
                ],
                output / (entry + "-state-compile.log"),
            )
            extension = (
                ".dll"
                if os.name == "nt"
                else (".dylib" if os.sys.platform == "darwin" else ".so")
            )
            runtime = (
                ["-static"]
                if os.name == "nt"
                else (
                    ["-static-libgfortran", "-static-libgcc"]
                    if os.sys.platform == "darwin"
                    else []
                )
            )
            if os.name == "nt":
                runtime += ["-Wl,--stack,16777216"]
            elif os.sys.platform == "darwin":
                quad = subprocess.check_output(
                    [fc, "-print-file-name=libquadmath.a"], text=True
                ).strip()
                if Path(quad).is_file():
                    runtime += ["-Wl,-force_load," + quad]
                runtime += ["-Wl,-dead_strip_dylibs"]
            else:
                # GNU's distributed static runtime is not position independent.
                # Bundle its shared runtimes alongside the embedded kernel.
                runtime += [
                    "-Wl,-Bsymbolic",
                    "-Wl,--disable-new-dtags,-rpath,$ORIGIN/runtime:$ORIGIN",
                ]
                dependencies = output / "runtime"
                dependencies.mkdir(exist_ok=True)
                for name in ["libgfortran.so.5", "libquadmath.so.0", "libgcc_s.so.1"]:
                    path = Path(
                        subprocess.check_output(
                            [fc, "-print-file-name=" + name], text=True
                        ).strip()
                    )
                    if path.is_file():
                        shutil.copy2(path, dependencies / name)
            checked(
                [
                    fc,
                    "-shared",
                    *link,
                    output / "state.o",
                    *runtime,
                    "-lm",
                    "-o",
                    output / (entry + extension),
                ],
                output / (entry + "-link.log"),
            )
            for p, content in backups.items():
                p.write_bytes(content)
        else:
            # Each browser calculation starts a fresh module / linear memory.
            runtime = os.environ.get(
                "FLANG_RUNTIME", "/opt/flang/wasm/lib/libFortranRuntime.a"
            )
            checked(
                [
                    "emcc",
                    "-O2",
                    *link,
                    runtime,
                    "-o",
                    output / (entry + ".mjs"),
                    "-sMODULARIZE=1",
                    "-sEXPORT_ES6=1",
                    "-sEXPORTED_FUNCTIONS=_rsp_fortran_run",
                    "-sEXPORTED_RUNTIME_METHODS=ccall,FS",
                    "-sALLOW_MEMORY_GROWTH=1",
                    "-sSTACK_SIZE=16777216",
                    "-sEXIT_RUNTIME=0",
                    "--no-entry",
                ],
                output / (entry + "-link.log"),
            )
    metadata = dict(
        abi_version=1,
        source_archive_sha256="42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1",
        kind=kind,
        compiler=subprocess.check_output([fc, "--version"], text=True).splitlines()[0],
        flags=flags,
        adapter_changes=changes,
        compatibility_patches=COMPATIBILITY_PATCHES,
        state_snapshot=info,
        wasm_reset="fresh module per calculation" if kind == "wasm" else None,
    )
    (output / "library-build-info.json").write_text(
        json.dumps(metadata, indent=2) + "\n"
    )
    print(output, flush=True)


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("kind", choices=["native", "wasm"])
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--fc")
    p.add_argument("--jobs", type=int, default=8)
    a = p.parse_args()
    build_library(a.kind, a.output, a.fc, a.jobs)
