# ADR 0001: M0–M1 bootstrap scope

Date: 2026-11-09. Status: adopted at bootstrap.

Later boundary update: ADR 0002 assigns product bindings and cross-platform
acceptance separately. The original three-backend/WASM checks below are historical;
the current public repository does not own WASM builds or browser support.

Use Rust 1.97.0 / Edition 2024, Cargo Workspace, and a pinned Cargo.lock. Start with
foundational types, model protocol, native Runtime, and a compilable template.
Keep the original overall design. Core types depend on no Python/Web/Desktop framework.

Keep the official catalog empty until a versioned original model, coefficients,
and reference outputs exist. Fixtures test engineering contracts only. Template
execution returns an explicit error, never invented scientific results. Do not
automatically assign Apache-2.0 while source/data permission is unknown; crates
remain unpublished. Subsequent original-code licensing is recorded in ADR 0004.

M0 inventories use CSV; source and data licenses are recorded separately. Model
API is an evolving draft until PROSPECT validates it. Catalog registration is
initially explicit Rust; generation, optional features, and contribution without
core changes are M2 work.

Bootstrap CI covered Windows/Linux/macOS native builds and shared-crate WASM
compilation. ADR 0002 subsequently removed public WASM checks. Public M1 targets
Python bindings and native/Python agreement; product CI owns WASM/browser evidence.
