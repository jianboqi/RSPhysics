# RSPhysics: Overall Design for Open Remote Sensing Physics Infrastructure

> **Status**: Architecture proposal v0.2 (historical; scope superseded by [v0.3](design-v0.3.md) and subsequent ADRs)  
> **Date**: 2026-11-09  
> **Positioning**: Open, Reproducible, Extensible Infrastructure for Remote Sensing Physics Models  
> **Audience**: Core developers, scientific model maintainers, external contributors, project leads, and coding assistants such as Codex

---

## 📘 0. Executive summary and agreed decisions

RSPhysics proposes replacing a centrally hosted remote sensing model platform with open infrastructure using **Rust for shared scientific implementations, source contributions, Python/Web/Desktop interfaces, scientific consistency, and reproducibility**.

**Agreed decisions at the time of this proposal:**

1. **Rust is the core language.** Original Fortran/C/C++ implementations provide migration evidence and numerical references. Models are migrated incrementally.
2. **The project is intended to be fully open source**, with public repository collaboration and contributions from researchers.
3. **Models enter through source contributions.** Contributors fork the repository, add a Rust model crate following the common conventions, build and test locally, and submit a PR for scientific and engineering review.
4. **Use a Git monorepo and Cargo workspace.** A plugin marketplace, dynamic binary loader, and separate model registry service are not initial priorities.
5. **The Python package is a primary deliverable**, targeting `pip install rsphysics`, common discovery/execution, NumPy batches, and suitable model-specific interfaces.
6. **Prefer local browser computation:** Rust to WebAssembly in a Web Worker, minimizing dependence on compute servers. Static assets/CDNs still require hosting.
7. **Desktop reuses the model sources**, running native Rust; a Tauri UI can follow later.
8. **Manage scientific identity, implementation versions, and platform releases separately.** Results must be traceable to source, parameters, and coefficient data.
9. **Implementation consistency and scientific comparison are both important:** standardized physical inputs/outputs, original-reference checks, and cross-model benchmarks.
10. **LESS3 is outside any mandatory open-source or Rust-rewrite scope.** Any integration is through an explicit independent-project protocol. RSPhysics licensing decisions do not automatically change LESS3 licensing.

**Guiding principle:** shared source management, independent model development, common interfaces, scientific validation first, and platform-specific builds.

---

## 📘 1. Vision and success criteria

### 1.1 Problems to address

Traditional platforms often face these problems:

- Centrally deployed models require server CPU/GPU resources and difficult concurrency/queue management.
- Different languages, executables, and parameter files prevent consistent invocation.
- Website/backend coupling requires repeated interface and page development for each model.
- Original implementations lack systematic numerical benchmarks, dependency records, CI, and cross-model comparisons.
- Researchers struggle to integrate models into Python workflows, teaching software, and third-party systems.

### 1.2 Three primary audiences

| Audience | Main needs | Primary deliverable |
|---|---|---|
| Remote sensing researchers | Convenient use, batches, retrieval workflows, reproducibility | PyPI `rsphysics` |
| Students and educators | Interactive experiments without local environment setup | Web Lab with local WASM computation |
| Model authors and developers | Source contributions, shared tooling, community adoption | Model SDK, templates, PR workflow |

Desktop targets large files, offline teaching, and complex local tasks, after the three priorities above.

### 1.3 Measures of success

Success requires more than counting ported models:

1. One validated Rust implementation builds for native, Python, and WASM according to its declared capabilities.
2. An external team can contribute a model using the template without changing the common core, and pass automated and human review.
3. Build tooling connects new models to the Python API and website catalog without rewriting the application layer for every model.
4. Users can identify, cite, and reproduce a particular scientific and implementation version.
5. The platform distinguishes implementation fidelity from differences between scientific models under comparable conditions.

---

## 📘 2. Architectural principles

**A. One scientific implementation**  
Maintain one authoritative Rust scientific kernel for each model. Python, Web, and Desktop supply thin bindings rather than independently rewriting the equations.

**B. Source-first contribution**  
Accept reviewed source PRs and include models at build time. Initially exclude arbitrary third-party binary hot loading, online plugin installation, and execution.

**C. Typed core, declarative boundary**  
Use typed parameters, arrays, `f64`, and explicit physical meanings/units internally; use declarative metadata for discovery and interaction. Keep JSON parsing outside numerical hot loops.

**D. Scientific correctness before performance**  
Establish reference tests before optimization. Performance work must not silently alter equations, numerical methods, or accuracy.

**E. Static catalog, not an online registry service**  
Generate the official catalog at build time from repository manifests. A static JSON catalog needs no separate registry service.

**F. Stable public API, flexible internal design**  
Keep Python and Model API contracts stable where practical; evolve internal Rust crates through SemVer.

**G. Explicit capabilities**  
Declare `native`, `python`, and `wasm` capabilities explicitly. Successful compilation does not establish numerical or performance validation.

---

## 📘 3. System architecture

```text
                    ┌─────────────────────────────────────────┐
                    │        GitHub: RSPhysics Monorepo      │
                    │  Rust Core + Model API + Model Source   │
                    │  Metadata + Reference Tests + Docs     │
                    └───────────────────┬─────────────────────┘
                                        │
                        Cargo Workspace / Build-time Catalog
                                        │
               ┌────────────────────────┴──────────────────────┐
               │             Shared Rust Runtime               │
               │  run / run_batch / validate / provenance      │
               └───────────┬─────────────┬─────────────┬───────┘
                           │             │             │
                   ┌───────▼───────┐ ┌───▼────────┐ ┌──▼─────────┐
                   │ PyO3/maturin │ │ wasm-bindgen│ │ Native CLI │
                   │ Python wheel │ │ WASM modules│ │ Tauri API  │
                   └───────┬───────┘ └───┬────────┘ └──┬─────────┘
                           │             │             │
                      pip / NumPy    Web Lab (local)   Desktop
```

### 3.1 Layers

| Layer | Contents | Constraint |
|---|---|---|
| Scientific foundations | Units, spectra, geometry, radiation, numerical algorithms | No Web/Python/Tauri dependencies |
| Model API | Identity, descriptions, errors, parameter/result protocols | Stable where practical; testable |
| Models | Independent PROSPECT, SAIL, GORT, 6S, and other crates | Independent tests and explicit scientific sources |
| Runtime / Catalog | Compile-time registration, validation, batching, provenance | Independent of specific frontends |
| Bindings | Python, WASM, native CLI | No duplicate physical equations |
| Applications | Web Lab, Desktop, documentation site | Metadata-driven interfaces |

### 3.2 Systems excluded from the initial version

- Login and account permissions.
- Central compute queues and scheduling clusters.
- Plugin marketplaces, dynamic-library ABI frameworks, and WASM Component distribution services.
- Services that automatically execute uploaded models in production.
- A separate website or Python project for each model.

Consider these only when concrete demand emerges; source contributions and CI suffice for the initial ecosystem.

---

## 📘 4. Suggested source layout

```text
rsphysics/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── Cargo.toml                       # Workspace
├── Cargo.lock                       # Pin integrated build dependencies
├── rust-toolchain.toml              # Pin the tested toolchain
│
├── crates/
│   ├── rsphysics-core/             # Spectral axes, units, geometry, radiation
│   ├── rsphysics-numerics/         # Integration, interpolation, roots, solvers
│   ├── rsphysics-model-api/        # Common traits, descriptors, errors
│   ├── rsphysics-catalog/          # Build-time static registration
│   ├── rsphysics-runtime/          # Execution, batches, validation, provenance
│   └── models/
│       ├── vegetation/
│       │   ├── prospect-5/
│       │   ├── prospect-d/
│       │   ├── sail/
│       │   └── gort/
│       ├── atmosphere/
│       │   └── sixs/
│       ├── water/
│       ├── soil/
│       ├── snow/
│       └── ecosystem/
│
├── bindings/
│   ├── python/                     # PyO3 + maturin + pyproject.toml
│   └── wasm/                       # Bindings per model/model group
│
├── apps/
│   ├── web/                        # React + TypeScript
│   ├── desktop/                    # Tauri; later phase
│   └── cli/                        # rsphysics command
│
├── benchmarks/
│   ├── reference/                  # Original-implementation comparisons
│   ├── scientific/                 # Cross-model/observation comparisons
│   └── performance/                # Runtime and memory
│
├── data/
│   └── model-catalog/             # Static manifests and citations
├── docs/
│   ├── architecture/
│   ├── models/
│   ├── tutorials/
│   └── decisions/                  # Architecture decision records
├── templates/
│   └── rust-model/                 # Minimal contributor template
├── tooling/
│   ├── scaffold/                   # Create crates/manifests
│   ├── generate-catalog/           # Static catalog and binding inventories
│   └── reference-runner/           # Original-model reference generation
└── .github/
    ├── workflows/
    ├── ISSUE_TEMPLATE/
    └── PULL_REQUEST_TEMPLATE.md
```

Each model is an independent crate, testable with `cargo test -p <crate-name>`. Python/WASM builds need not enable every model together.

---

## 📘 5. Model API requirements

### 5.1 Typed internal Rust interface

The following **interface proposal** illustrates structure; it is not a released, compilable API.

```rust
pub trait PhysicsModel {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    fn descriptor() -> ModelDescriptor;
    fn run(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}
```

Each model defines its own `Input` and `Output`. Avoid an unconstrained `HashMap<String, f64>` as the internal representation for every scientific model.

### 5.2 Common dynamic boundary

An explicit adapter wraps a concrete model for common execution (illustrative):

```text
list_models() -> Vec<ModelDescriptor>
describe(model_id) -> ModelDescriptor
run(model_id, parameters, run_options) -> ResultEnvelope
run_batch(model_id, parameter_table, run_options) -> BatchResult
```

Design considerations:

- A trait with associated types is unsuitable as a directly heterogeneous collection of trait objects; use an object-safe adapter or generated explicit dispatch.
- The common parameter protocol supports discovery and UI. Decode and validate before calling the typed kernel.
- Pass structured arrays or buffers for throughput; avoid one Python/Rust or JS/WASM call per pixel.
- Common error categories include invalid parameters, inputs outside the scientific domain, missing coefficients, unavailable backends, nonconvergence, and internal failures.

### 5.3 Common physical semantics

Specify physical meaning as well as field names:

- **Wavelength:** use `nm` internally by default, preserving axes and sampling information.
- **Angles:** state radians/degrees, relative-azimuth orientation, and zenith definitions.
- **Radiation:** distinguish radiance, irradiance, BRDF, BRF, reflectance, hemispherical albedo, and brightness temperature.
- **Dimensions:** name axes for points, spectra, angle sweeps, and scene batches.
- **Data:** identify sources and versions of coefficients, response functions, and aerosol/soil/leaf resources.
- **Validity:** descriptors state conditions, parameter domains, and limitations.

Two results are not physically interchangeable merely because both use `Vec<f64>`.

---

## 📘 6. Model sources, metadata, and registration

### 6.1 Minimal model directory

```text
crates/models/vegetation/prospect-d/
├── Cargo.toml
├── model.toml
├── README.md
├── LICENSE-ORIGIN.md       # Original sources, citations, permissions
├── src/
│   ├── lib.rs
│   ├── parameters.rs
│   └── solver.rs
├── tests/
│   ├── reference.rs
│   └── edge_cases.rs
└── testdata/
    ├── inputs.json
    └── expected.json
```

### 6.2 Example `model.toml`

> These fields are a **proposed convention**. Verify scientific parameters, papers, authors, links, and licensing against original materials when adding a model. Example identifiers do not establish implemented capability.

```toml
[model]
id = "vegetation.prospect-d"
family = "PROSPECT"
name = "PROSPECT-D"
scientific_version = "D"
implementation_version = "0.1.0"
category = "vegetation"
api_version = "0.1"
status = "experimental"

[source]
crate = "rsphysics-prospect-d"
original_language = "Fortran"
origin_reference = "TO_BE_VERIFIED"
license = "TO_BE_REVIEWED"

[capabilities]
native = true
python = true
wasm = true
batch = true
precision = "f64"

[validation]
reference_suite = "prospect-d-reference-v1"

[data]
resource_manifest = "data-resources.toml"
```

Rust types and structured schemas define actual inputs and outputs and generate website/Python documentation and parameter tables. Implement input/output schema validation in v0.1.

### 6.3 Build-time registration

- Maintain **one model inventory**, such as `model-catalog.toml`.
- Add new crates to the workspace and inventory through PRs.
- Check crate/metadata/feature/ID consistency and generate static registration and website catalogs.
- Python and CLI expose the features enabled in their builds.
- Build WASM packages per model/group and download them on demand.

**Constraint:** merging a new model requires rebuilding and publishing the relevant Wheels, WASM assets, and desktop binaries. New Rust sources cannot be loaded without compilation.

### 6.4 Model IDs and namespaces

- IDs are independent of crate names; prefer stable identifiers such as `domain.family.variant`.
- Avoid renaming published IDs; provide explicit aliases/migration statements when needed.
- Distinct scientific models or assumptions require distinct IDs.
- Preserve contributor and original-author attribution in manifests.

---

## 📘 7. Four distinct version categories

| Category | Example | Management | Purpose |
|---|---|---|---|
| **Scientific identity/version** | `PROSPECT-5`, `PROSPECT-D` | Stable independent model ID | Equations, assumptions, parameter system |
| **Implementation version** | `rsphysics-prospect-d 0.2.1` | Crate SemVer | Fixes, optimization, interfaces |
| **Platform/API version** | `rsphysics 0.3.0`, API `0.x` | Git tags, PyPI releases | User interfaces and compatibility |
| **Resource version** | `optical-coefficients 2026.1` | Resource manifest and SHA-256 | Reproducible tables and coefficients |

Changing a scientific model differs from refactoring code or improving runtime performance.

### 7.1 Version policy

- A shared monorepo can contain independently versioned crates. A Python release snapshots a combination of models.
- A Git tag such as `v0.3.0` identifies specific source, dependencies, and model membership.
- Associate the lockfile, toolchain, Wheels, WASM artifacts, and catalog with the same release.
- The latest package need not compile every historical implementation. Recover historical versions through tags, available Wheels, and archived tests; important scientific variants can coexist as crates.
- SemVer expresses interface compatibility intent, not identical scientific output. Document numerical changes and reference comparisons.

### 7.2 Per-run manifest

A proposed `rsphysics.run()` export:

```json
{
  "model_id": "vegetation.prospect-d",
  "scientific_version": "D",
  "implementation_version": "0.1.0",
  "platform_version": "0.1.0",
  "git_revision": "<commit-sha>",
  "backend": "wasm-cpu-f64",
  "data_hashes": {"coefficients": "sha256:<hash>"},
  "parameters": {"example": "..."},
  "validation_suite": "prospect-d-reference-v1"
}
```

The actual implementation should also include units, build identity, relevant numerical settings, and input references. Public exports must not automatically disclose private absolute paths or sensitive contents.

---

## 📘 8. Python as the primary library product

### 8.1 Installation and use

Target after publication:

```bash
pip install rsphysics
```

Suggested common API:

```python
import rsphysics as rsp

print(rsp.list_models())
print(rsp.describe("vegetation.prospect-d"))

result = rsp.run(
    "vegetation.prospect-d",
    params={
        "chlorophyll": 40.0,
        "water": 0.015,
        "dry_matter": 0.008,
    },
)
```

> This is an **API illustration**, not an existing package or a confirmed complete scientific parameter definition.

Offer typed, IDE-friendly functions such as `rsp.vegetation.prospect_d(...)`. Common and dedicated interfaces must call the same Rust solver.

### 8.2 Build tooling

- Build Windows, Linux, and macOS Wheels using PyO3 and maturin.
- Prefer NumPy inputs/outputs and execute batch loops inside Rust.
- Official compatible Wheels should not require users to install Rust.
- Cover supported OS/Python versions through CI matrices.
- Consider optional distribution for large models/resources when needed; avoid premature package fragmentation.

### 8.3 Public Python API stability

Research appendices, teaching scripts, and third-party code depend on Python names, units, versioning, errors, and batch conventions. Prioritize stable contracts.

---

## 📘 9. Browser computation

### 9.1 Execution strategy

- Compile Rust to WASM with JS/TypeScript bindings through `wasm-bindgen`.
- Execute inside Web Workers to keep the UI responsive.
- Use the static catalog for model discovery and parameter forms.
- Download per-model/group bundles on demand rather than loading everything initially.
- Use CPU `f64` as the numerical baseline; GPU/WebGPU is a later optional optimization.
- Provide batch/resource limits, cancellation, progress, and resource information for expensive tasks.
- Save configurations locally and export CSV/JSON/Python; IndexedDB/PWA caching is optional.

### 9.2 Website information architecture

1. **Catalog:** categories, scientific versions, authors, papers, platforms, validation status, and sources.
2. **Model detail:** assumptions, inputs/outputs, validity, implementation versions, and reference cases.
3. **Lab:** parameter entry, spectra/plots, parameter sweeps, and export.
4. **Compare:** multiple models under comparable scenarios, with differences explained.
5. **Docs:** contributions, Python API, conventions, and scientific validation.

Prefer metadata-driven UI. New models should not require hand-written forms. Reviewed specialized UI is appropriate for complex inputs such as 3D scenes.

### 9.3 Limitations

Native Rust does not imply browser compatibility. OS libraries, external resources, and memory constraints can prevent WASM support; mark unsupported models truthfully.

---

## 📘 10. Desktop and CLI

### Desktop

- Consider Tauri 2 with shared Web UI.
- Run the native Rust kernel rather than requiring WASM on desktop.
- Support larger memory, parallelism, local files, and offline packages.
- Develop Desktop after Python/Web foundations stabilize.

### CLI

Proposed commands:

```bash
rsphysics models list
rsphysics models describe vegetation.prospect-d
rsphysics run --model vegetation.prospect-d --input input.json
rsphysics benchmark --suite prospect-d-reference-v1
```

CLI, Python, and Web share the Rust runtime and model metadata.

---

## 📘 11. Scientific validation and comparison

### 11.1 Implementation validation

Question: **Does the Rust implementation faithfully implement the scientific model?**

- Archive original source identity, build method, inputs, and outputs.
- Cover typical, boundary, extreme, and invalid inputs.
- Compare intermediate variables as well as final outputs.
- Use absolute/relative errors, spectral RMSE, conservation, monotonicity, symmetry, and appropriate physical checks.
- Specify model-specific tolerances rather than an arbitrary project-wide threshold.

### 11.2 Scientific benchmarking

Question: **How do models differ in performance, applicability, and bias under shared scenarios or observations?**

- Share physical scenarios/observations rather than merely matching parameter names.
- Document parameter mappings and geometry correspondences.
- Account for model assumptions, validity, and available input information.
- Record dataset sources, licenses, and versions.
- Compare physical accuracy, runtime, memory, and spectral/angular scope.
- State evaluation conditions for rankings; avoid claims of universal superiority.

### 11.3 Backend consistency

Compare native Rust, PyO3, and WASM wherever supported. Use model-specific tolerances for floating-point differences rather than assuming bitwise identity across CPUs and browsers.

### 11.4 Suggested validation states

- `experimental`: runnable, with incomplete reference testing.
- `reference-validated`: agrees with a specified original reference.
- `benchmark-evaluated`: evaluated in a specified cross-model/observation benchmark.
- `deprecated`: discouraged for new use, with retained provenance.

Bind validation status to an **implementation version and test-suite version**, rather than permanently attaching it to a model name.

---

## 📘 12. Community contributions

### 12.1 Workflow

1. Fork the official repository and create a branch.
2. Generate a crate from the template, or fix an existing model.
3. Add source, Cargo/model manifests, README, references, and licensing information.
4. Provide original-reference methodology, input/output fixtures, and boundary tests.
5. Register the crate in the workspace/catalog and run local checks.
6. Submit a PR for CI and scientific review.
7. Merge after review; include the model in subsequent Python/Web/Desktop releases.

### 12.2 Required materials

| Material | Acceptance requirement |
|---|---|
| Rust solver | No private build environment; conforms to interfaces |
| Description | Stable ID, scientific version, fields, units |
| Scientific basis | Papers, original source, authors/maintainers |
| Permissions | Redistribution rights for code and data |
| Reference tests | Check results and scientific scope |
| Example | At least one Rust/Python invocation |
| Capabilities | Accurate native/python/wasm status |
| Documentation | Numerical methods, limitations, version changes |

### 12.3 Review responsibilities

- **Engineering:** interfaces, errors, dependencies, security, tests, builds.
- **Scientific:** equations, parameter meanings, references, validity, citations.
- **Licensing:** planned redistribution rights for code and data.

Contributors need not master all three platforms. Model authors focus on science and references; maintainers support bindings and packaging.

### 12.4 Security and supply chain

External PR code and build scripts are untrusted. Use isolated runners, least privilege, and dependency review; do not expose PyPI tokens, repository secrets, or production deployment permissions to unreviewed PR workflows.

---

## 📘 13. Licensing and governance

- Evaluate Apache-2.0 for original Rust infrastructure, subject to institutional/team IP policies.
- Access to original model source does not itself grant redistribution or relicensing rights. Check ownership, dependencies, source licenses, and resource permissions separately.
- Models without redistribution rights cannot simply enter a fully open repository. Record candidates or independently distributable portions as appropriate.
- Preserve authorship, papers, and software citations, potentially through `CITATION.cff`.
- Establish contribution/conduct guides, maintainers, scientific reviewers, deprecation policy, and ADRs.
- LESS3 code, commercial licensing, and specialized high-performance implementations remain independent of RSPhysics.

---

## 📘 14. Build, CI, and releases

### 14.1 Proposed developer commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p rsphysics-prospect-d
cargo build -p rsphysics-cli --release
```

Optional model-group Cargo features can help control builds; avoid an unmaintainable feature matrix. Core API remains independent of model implementations.

### 14.2 CI pipeline

```text
PR submission
  ├─ format / clippy / Rust unit tests
  ├─ model.toml / schema / catalog checks
  ├─ reference comparisons and scientific checks
  ├─ Python build and smoke tests
  ├─ WASM/browser smoke tests for declared supported models
  └─ license, dependency, and security checks
                      ↓ merge
Release tag → multi-OS Python Wheels → PyPI
            → model/group WASM → static Web release
            → CLI / Desktop artifacts
            → catalog, changelog, and documentation
```

PRs need not run every expensive suite. Combine change-based selection, required baseline checks, and periodic full testing; run all critical validation before releases.

### 14.3 Distribution

- PyPI: `rsphysics`, subject to name availability before publication.
- crates.io: publish useful core components when needed; not every internal crate requires publication.
- Web: versioned static resources and demand-loaded WASM.
- Docs: versioned alongside releases, retaining historical access.
- GitHub releases: model inventory, software changes, and scientific-result changes.

---

## 📘 15. Migration from the existing platform

### 15.1 Inventory first

Record these fields for each model:

```text
model_id, model_family, scientific_version, original_language,
original_source, license, third_party_data, dependencies,
reference_cases, compute_cost, memory_cost, wasm_feasibility,
migration_priority, owner
```

Even where original source is available, assess licensing, resources, and portability separately.

### 15.2 Suggested migration sequence

| Phase | Models | Validation focus |
|---|---|---|
| Pilot | A specified PROSPECT scientific version | Rust core, parameter/spectral conventions, native/Python/WASM |
| Second group | SAIL, GORT | Coupling, geometry, complex inputs/results |
| Expansion | Hapke, soil, water | Cross-domain semantics and common numerics |
| Complex solvers | 6S, RT3 | Multiple scattering, resources, iteration |
| Long tail | Large/specialized models | Individual accuracy, rights, dependencies, portability |

Prioritize demand, scientific impact, complexity, references, licensing, and browser feasibility; line count alone is insufficient.

### 15.3 Website transition

- Run old and new systems in parallel, retaining reference results and transition links.
- Launch a small validated collection before completing all migrations.
- Map historical IDs and document migrations to preserve citations.
- Retire old compute endpoints after scientific acceptance and adequate replacement workflows.

---

## 📘 16. Milestones and acceptance

### M0 — Conventions and references

**Deliver:** asset/rights inventories, Model API v0.1 proposal, spectral/angular/quantity conventions, and original PROSPECT references.

**Accept:** fixed scripts reproduce original inputs/outputs with traceable sources and licensing.

### M1 — Minimal three-platform workflow

**Deliver:** workspace, core crates, one PROSPECT solver, Python Wheel, minimal WASM page, numerical CI.

**Accept:** the same Rust solver runs natively, through Python, and in-browser within model tolerances without a compute server.

### M2 — Contribution workflow

**Deliver:** template, manifest convention, static catalog, PR guide, metadata checks, and a second model, preferably SAIL.

**Accept:** a simulated external contributor adds a crate without core changes and completes local build, CI, review, and Python/Web release integration.

### M3 — Shared modeling interface

**Deliver:** catalog/details, metadata forms, plots, result/script export, provenance records, and GORT or another structural model.

**Accept:** standard models need no complete hand-written frontend; browser workflows support computation and export.

### M4 — Scientific comparison and release

**Deliver:** implementation/scientific benchmark systems, standard scenarios, documentation, PyPI release, and versioned publishing.

**Accept:** reproducible specified versions, clearly conditioned comparisons, pip installation, and browser access.

### M5 — Coverage and desktop

**Deliver:** incremental migrations, batch performance, Tauri desktop, offline teaching, and richer composition.

**Accept:** continued model extension without redesigning the core, serving all three audiences.

---

## 📘 17. Initial implementation tasks

Proceed in this order, before building a full website:

1. Initialize the workspace, conventions, CI, and pinned toolchain.
2. Draft the Model API RFC: IDs, descriptors, schemas, errors, capabilities.
3. Implement core spectral axes, angles/geometry, quantity/unit conventions, and numerical tests.
4. Implement the model template and manifest checker.
5. Select an original PROSPECT implementation and generate golden references.
6. Implement Rust PROSPECT and establish native reference agreement.
7. Add PyO3/maturin calls with NumPy output.
8. Add wasm-bindgen and browser Worker execution.
9. Add static catalog/discovery build tooling.
10. Have a second developer contribute another model using the external-PR workflow.
11. Build catalog and experiment UI after validating this workflow.

### Review questions

- Does the model reuse common scientific types and compatible units?
- Does each version identify science, implementation, or release?
- Are standard inputs/outputs traceable to the original reference?
- Are build instructions, resource permissions, and platform claims accurate?
- Do Python/WASM call the same solver without duplicated equations?
- Are large dependencies justified and their packaging costs understood?
- Are changes to numerical results explained and regression-tested?

---

## 📘 18. Risks and open decisions

| Risk | Suggested response |
|---|---|
| Excessive rewrite cost | Incremental migration; conventions and references first |
| Less mature Rust scientific ecosystem | Careful shared-library choices; dedicated complex-solver assessment |
| Oversized Web bundles | Shared source, separate model/group bundles, demand loading |
| Uncontrolled contribution quality | PR, CI, engineering/scientific/rights review |
| Confused scientific/software versions | Four version categories and run manifests |
| Cross-platform numerical differences | Model-specific tolerances and recorded settings/backends |
| Oversized Python package | Initial curated models; split/optional groups when justified |
| Available code with unclear permissions | Verify code/data rights before public inclusion |
| Growing maintenance burden | Layer maintainers, conventions, automated builds/tests |

**Decisions for project leadership:**

- Organization, final brand, and domain.
- Core license, initially considering Apache-2.0.
- Initial PROSPECT version and specific SAIL/GORT sources.
- Initial OS/Python support matrix.
- Minimum scientific test coverage and review ownership.
- Existing unit crates versus domain-specific newtypes.
- Whether large models may have independent source directories and restricted platform builds.

---

## 📘 19. Project positioning

> **RSPhysics is a community-developed, open-source reference implementation framework for remote sensing physics models with traceable scientific results.**
>
> It provides common Rust model conventions and validated computations through Python, WebAssembly, and desktop applications for research, education, and development. Research teams can contribute source under common testing, versioning, and comparison conventions.

**Initial priority: a repeatable Model API + PROSPECT + Python + WASM + external-source-PR workflow, before a complete website or all-model migration.**

---

*This is a historical design proposal, not a statement of implemented capability. Proposed API names and commands require implementation and release verification.*
