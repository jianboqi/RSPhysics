# Model API v0.1 RFC

Status: implemented, evolving draft; originally recorded 2026-11-09. The public
API has not been formally released. The original PROSPECT-only observations below
describe the initial implementation; the current catalog contains additional independent models.

The public API, scientific models, and Python bindings are in RSPhysics. Product
WASM/Worker/Web/Desktop integration is separate; see [design v0.3](../design-v0.3.md).
Website source visibility/licensing was subsequently updated by
[ADR 0004](../decisions/0004-apache-project-license.md). Models retain their own
typed inputs/outputs; the common layer supplies protocols and mechanisms.

## 📘 Layers and invocation

`PhysicsModel` has associated `Input` / `Output` types and methods `metadata`,
`validate_input`, `run`, and `validate_output`. v0.1 uses serializable `ModelError`
at the boundary; implementations may map their own internal error types.
Public direct Rust `run` calls must also validate inputs.

`TypedAdapter<M>` implements object-safe `ModelAdapter`. It generates and compiles
input/output JSON Schemas at creation. A call checks input schema, decodes typed
parameters, checks the scientific domain, runs the solver, checks scientific
outputs, serializes, and checks output schema. Equations remain in the solver.

Use Serde/Schemars and preferably `deny_unknown_fields`. Schema validation covers
structure, required fields, and declared ranges. Models check cross-field
constraints, finite values, energy, output shapes, and scientific applicability.
Shared type constructors/deserialization enforce their own domains, sometimes
more strictly than generated schemas. Schemas do not replace scientific checks.

`Catalog` stores build-time linked adapters, rejects duplicate IDs, and does not
load dynamic third-party code. Its initial scientific entry was
`vegetation.prospect-d`; templates/fixtures are excluded. `Runtime` supplies
`list_models`, `describe`, `run`, and `run_batch`. `manifest_for` constructs
equivalent provenance for already validated dedicated execution paths; it does
not itself replace parameter or scientific validation.

## 📘 Identity, versions, and errors

IDs contain at least two dot-separated lowercase segments. Segments start with a
letter, contain lowercase letters/digits/hyphens, and cannot end with a hyphen:
for example, `vegetation.prospect-d`. IDs are independent of crate names.

Scientific versions are model-defined stable identities. Implementation versions
are complete SemVer values; API version is strictly `0.1`; platform version comes
from the Runtime package. External resources use named SHA-256 values. Future
models may version independently of the workspace.

Errors serialize as `{ "kind": "invalid_parameter", "message": "..." }`.
Kinds are `invalid_parameter`, `out_of_domain`, `missing_data`,
`backend_unavailable`, `non_convergence`, `unknown_model`, and `internal`.
An output-contract violation is an internal error, not a successful result.

## 📘 Descriptors and model.toml

`ModelDescriptor` contains `metadata`, `input_schema`, and `output_schema`.
Metadata comes from the model's `model.toml`; schemas come from actual Rust
types, avoiding a second manually maintained structural definition.

| Section | Fields |
|---|---|
| `model` | id, name, family, category, scientific_version, implementation_version, api_version, status, limitations |
| `source` | crate, authors, references, license |
| `capabilities` | native, python, wasm, batch, precision (currently f64) |
| `validation` | evidence array: kind, suite, implementation_version, backend |
| `data.hashes` | Resource names mapped to `sha256:<64 hex digits>` |
| `input_semantics` / `output_semantics` | Field JSON Pointers mapped to unit, optional quantity, axes, description |

See the parsable [template](../../templates/rust-model/model.toml). Design fields
such as `origin_reference`, `original_language`, and `resource_manifest` remain
proposals, not accepted fields; detailed provenance is in `LICENSE-ORIGIN.md`.
Unknown TOML fields are rejected. Changes require updates to this RFC, the parser,
and compatibility tests.

Semantic `axes` name dimensions; they do not infer shapes. Models check lengths
and output dimensions. Known radiometric quantities require their canonical
units. Semantic tables define the contract; the current implementation does not
verify every pointer against complex nested schemas or convert units automatically.

## 📘 Capabilities and evidence

`native`, `python`, and `wasm` are independent capabilities. Runtime binding context
selects the actual backend. Capability is separate from validation status. The
initial PROSPECT-D native/Python paths use one solver and versioned evidence.
Keeping a WASM descriptor field does not promise public-library browser support.
Unverified product capabilities must not be labelled supported.

`experimental` requires no scientific evidence. `reference-validated` requires
implementation-agreement evidence for the current implementation version;
`benchmark-evaluated` also requires scientific-comparison evidence. Evidence must
name a nonempty suite, match the implementation version, and reference a supported
backend. Declarations do not prove passing checks: CI and scientific maintainers
review actual suites and tolerances.

The template's `UNSET` is only a development placeholder. Official-catalog checks
reject unresolved license placeholders; tools do not establish legal permission.

## 📘 Batch execution and provenance

Generic v0.1 batches accept parameter objects for one model, execute in input
order, and fail on the first error with a zero-based row and original error kind.
No partial result is returned. Empty batches still check the model, backend, and
batch capability. No implicit broadcasting or global thread pool is created.
The initial dedicated PROSPECT NumPy path accepts `(batch,8)` and returns
`(batch,2101)` spectra, copying inputs before releasing the GIL and handing
Rust-owned output buffers to NumPy. Later unified batches are described in the
[infrastructure guide](../python-infrastructure.md).

Results record model/scientific/implementation/API/platform versions, build ID,
backend, precision, decoded parameters with defaults filled, resource hashes,
evidence, and output units. Algorithm options must be typed parameters or fixed
by implementation version, never undocumented environment settings. Callers
should remove sensitive parameters before sharing; Runtime does not collect paths.

Revision/build ID are explicitly injected by the build environment and remain
null when absent. Local builds therefore lack full release provenance.
`git_revision` identifies the public scientific source; product revision must be
recorded separately. A product CI `github.sha` must not impersonate the core SHA.
Manifest resource declarations require verification by actual resource loaders;
placeholder/unverified hashes are not validation evidence.

## 📘 References and follow-up

The workspace shares an integration lockfile. Schemas are generated from Rust and
compiled validators are reused. Schema validation disables default network/file
reference resolution. The initial interface was exercised with PROSPECT parameters,
2,101-band coefficients, 32 official R cases, NumPy I/O, and a local Wheel. The
original next steps were a second model, catalog generation, and formal releases;
actual later progress is in [implementation status](../implementation-status.md).
WASM Worker remains product work.

[^1]: Cargo workspace reference. https://doc.rust-lang.org/cargo/reference/workspaces.html
[^2]: Schemars 1.2.3. https://docs.rs/schemars/1.2.3/schemars/
[^3]: jsonschema 0.61.0. https://docs.rs/jsonschema/0.61.0/jsonschema/

The workspace/schema implementation follows these upstream references.[^1][^2][^3]
