"""Select only FC, retaining the platform's Rust/C/C++ compiler environment."""

import os
from pathlib import Path
import shutil
import subprocess

compiler = os.environ.get("RSP_FORTRAN_COMPILER", "gfortran")
resolved = shutil.which(compiler)
if resolved is None:
    raise SystemExit(f"GNU Fortran compiler not found: {compiler}")
subprocess.run([resolved, "--version"], check=True)
if destination := os.environ.get("GITHUB_ENV"):
    with Path(destination).open("a", encoding="utf-8") as stream:
        stream.write(f"FC={resolved}\n")
