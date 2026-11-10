# ADR 0003: Publish website product source

Date: 2026-11-10. Status: adopted.

Later licensing update: original code now uses Apache-2.0; see
[ADR 0004](0004-apache-project-license.md). Website source publication and separate
library/product responsibilities remain. The GPL product choice below is historical.

After discussing GPLv3 and browser WASM delivery, the owner explicitly allowed
all website source to be public. This updates ADR 0002's visibility choice for
RSPhysics-app, without merging library and product responsibilities.

- RSPhysics owns public Rust science, models, references, Python/NumPy, and pip.
- The initial open-product choice licensed original front end, WASM adapter, Worker, UI, scripts, and tests GPL-3.0-only, retaining third-party notices. ADR 0004 supersedes that original-code choice.
- Product builds own browser/WASM acceptance; no generic core WASM CI is added.
- Distribution under applicable GPL conditions provides matching complete source: exact core version, bindings, necessary scripts, and lockfiles. Publishing only algorithm crates is insufficient.
- This choice does not relicense third-party models/data or automatically settle previously undecided core-source licenses. Review coefficient rights separately.
- Source visibility, remote repository creation, PyPI publication, and website deployment are separate actions. None had occurred; no invented source URL is supplied.

Product LICENSE, package metadata, and `docs/source-distribution.md` record the
decision. `private: true` / `publish = false` prevent accidental registry uploads;
they do not mean confidential source. Assess future dependencies by actual terms.
