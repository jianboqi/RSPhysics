# Direct 6SV2.1 native boundary

This crate embeds two original-Fortran libraries built by GNU Fortran. It loads
their C ABI with `libloading`; it never starts a 6S executable. Unsafe operations
are confined to the private native module. Scientific Rust crates retain the
workspace's unsafe-code prohibition.

`rsp_sixs_run(const char *directory)` takes a NUL-terminated calculation directory
and returns a status code. Its caller retains the string and library handles
throughout the call. Input cards, original formatted output, and auxiliary files
are scoped to a private directory removed after collecting results. Windows and
Linux calls serialize and restore model-owned writable state. macOS uses a
unique library image for each call. Embedded kernels and required shared runtime
assets are verified against their build-time bytes before loading from the
content-addressed kernel cache.

The public typed configuration and report parser live in `rsphysics-sixs`.
Original executables are built only by separate reference tooling. Compiler,
source hashes, compatibility corrections, and state-reset details are recorded
by the component build tools.
