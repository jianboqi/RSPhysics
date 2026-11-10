"""Offline fixture and embedded-resource integrity checks for both soil models."""
from pathlib import Path
import hashlib,json

ROOT=Path(__file__).resolve().parents[3]
for slug in ['sort','sort-l']:
    folder=ROOT/'benchmarks/reference'/slug
    manifest=json.loads((folder/'manifest.json').read_text())
    for name,expected in manifest['fixture_hashes'].items():
        assert hashlib.sha256((folder/name).read_bytes()).hexdigest()==expected,name
    basis=ROOT/'crates/models/soil/sort/data/soil_basis.csv'
    assert hashlib.sha256(basis.read_bytes()).hexdigest()==manifest['soil_basis_sha256']
    print(f'{slug}: fixture and spectral resource checksums match')
