# Deferred work and documentation policy

Maintained core documentation is English. Preserve physical units, exact API and
model identifiers, reference versions, numerical evidence, and source/license
notices when editing or translating documents.

## 📘 Website localization

Chinese/English bilingual UI is a later task in `RSPhysics-app`; it is not urgent
and is not part of this documentation change. Keep the existing interface until
that task is scheduled. Future localization should cover navigation, model
descriptions, parameter explanations, plots, errors, and exports consistently.

## 📘 Existing engineering follow-up

- Run the configured native/Python CI across supported operating systems.
- Complete publication and third-party data/SMAC permission work before claiming a cleared release.
- Verify external model contribution, catalog generation, and optional model features.
- Evaluate Tauri separately after the Python and browser products are established.

Model intake continues through pinned original references, typed Rust solvers,
Python, product WASM, and browser verification. There is no generic model-coupling
or inversion work authorized by the documentation language decision.
