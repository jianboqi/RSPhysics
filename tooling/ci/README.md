# Package release

The Wheels workflow tests installed binary wheels on CPython 3.11–3.15 for five
platforms, and separately builds and tests the source distribution. The verified
bundle is available as the `rsphysics-distributions` workflow artifact. Its
`dist/` directory contains the installable files; `SHA256SUMS` records their
checksums.

## Trusted Publishing setup

In [PyPI account publishing](https://pypi.org/manage/account/publishing/), add
a GitHub pending publisher for the first release:

| Field | Value |
|---|---|
| PyPI project name | `rsphysics` |
| GitHub owner | `jianboqi` |
| Repository | `RSPhysics` |
| Workflow filename | `wheels.yml` |
| GitHub environment | `pypi` |

No PyPI password or long-lived API token is stored in GitHub. Only the publish
job has permission to obtain an OIDC token. Pushes, pull requests and manual
workflow runs build and validate artifacts without publishing.

## Publish a version

1. Update the version together in `Cargo.toml` and
   `bindings/python/pyproject.toml`, and regenerate `Cargo.lock` if required.
2. Commit the release changes and wait for Checks and Wheels to pass on main.
3. Create and publish a GitHub release using the exact tag `v<version>`,
   targeting the verified commit. The release event rebuilds, tests and
   publishes that revision's complete distributions through the `pypi`
   environment. Ordinary tag pushes do not upload to PyPI.
4. Confirm that the workflow's Windows, Linux and macOS PyPI installation jobs
   succeed. They install the published version and execute the public examples.

The publication job downloads only artifacts from its own workflow run,
checks their hashes, and uploads wheels and the source archive together.
Version mismatches or missing distributions fail before publication.

Local preparation checks:

```sh
python tooling/ci/verify_distribution.py --tag v0.1.0
python -m unittest discover -s tooling/ci/tests
python tooling/ci/verify_documentation.py
python bindings/python/examples/quickstart.py --plot --output-dir target/quickstart
python bindings/python/examples/all_models.py
```

The documentation and plotting checks require the optional `examples`
dependencies in addition to the installed package.
