# 6SV2.1 and Fortran runtime notices

This package includes libraries built from the original 6SV2.1 Fortran
distribution by Eric Vermote and collaborators. The source distribution retains
every original file, its attribution, and the per-file source manifest.

Original archive: https://salsa.umd.edu/files/6S/6sV2.1.tar

Archive SHA-256: 42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1

The retrieved archive contains no standalone license file. Its original
authorship and applicable terms are not replaced by Apache-2.0. The RSPhysics
interfaces and build adapters are project-original Apache-2.0 code. Build-copy
corrections are documented in components/6sv2.1/COMPATIBILITY.md in the source
distribution and repository.

Native binaries also include or bundle applicable GNU compiler runtimes:

- libgfortran and libgcc: GPL-3.0-or-later with the GCC Runtime Library Exception
  3.1. See GCC_GPL-3.0.txt and GCC_RUNTIME_EXCEPTION.txt.
- libquadmath, where included: LGPL-2.1-or-later. See LGPL-2.1.txt.

GNU runtime sources and build instructions are available from
https://gcc.gnu.org/releases/ and https://gcc.gnu.org/install/ . The native build
records the selected compiler version in library-build-info.json. Source builds
can select a replacement GNU toolchain through FC; the complete RSPhysics
source archive includes the library build recipe and original scientific source.

The browser's separately compiled runtime is not included in the Python wheel.
