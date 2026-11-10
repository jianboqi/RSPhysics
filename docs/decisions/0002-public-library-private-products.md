# ADR 0002: Separate public scientific library and products

Date: 2026-11-09. Status: responsibility boundary adopted; product organization
was initially undecided. This updates ADR 0001's ownership of M1 bindings and
multi-backend acceptance while retaining scientific validation first.

Update 2026-11-10: RSPhysics-app source is now intended to be public; see
[ADR 0003](0003-open-source-web-product.md) and the later
[license decision](0004-apache-project-license.md). Private visibility below is
historical. Technical separation, lockfile ownership, and validation responsibility remain.

RSPhysics is a remote-sensing scientific library. The public repository maintains
Rust kernels/models, Model API, references, and Python bindings, with Python/PyPI
as its main distribution. User-owned Web/Desktop/UI/WASM products may be separate.
Each model defines its parameters; the common layer only owns shared types/mechanisms.

The existing structure fits this boundary without splitting core crates. A single
private RSPhysics-apps project was recommended for shared Web/Desktop/UI, with
separate product repositories possible for access/team needs. This record neither
chooses that organization for the owner nor creates external repositories.

Products consume the same public Git URL/full SHA and own their Cargo.lock. The
core lockfile does not lock downstream graphs. Local path/patch overrides must
cover relevant direct core dependencies consistently. Core crates do not import products.

Public acceptance covers native science, Python/NumPy, and pip installation, not
generic WASM compilation/browser support. Product CI owns bindings, WASM, browsers,
and Tauri. Others may evaluate/compile public source to WASM under its licenses;
this is not an official all-model browser guarantee.

`BuildInfo.git_revision` identifies the scientific core. Product integration must
record its own revision separately; a product `github.sha` must not impersonate
the core SHA. Verify actual code/data permissions for every source.

See [design v0.3](../design-v0.3.md).
