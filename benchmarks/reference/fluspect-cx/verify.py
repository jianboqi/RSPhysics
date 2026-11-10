"""Offline integrity checks for the pinned original FLUSPECT-CX numerical cases."""
from pathlib import Path
import hashlib,json,zlib

ROOT=Path(__file__).resolve().parent

def verify():
    manifest=json.loads((ROOT/'manifest.json').read_text(encoding='utf-8'))
    assert set(manifest)=={'inputs.json','expected.json.zlib','provenance.json'}
    for name,want in manifest.items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==want,name
    provenance=json.loads((ROOT/'provenance.json').read_text(encoding='utf-8'))
    expected_raw=zlib.decompress((ROOT/'expected.json.zlib').read_bytes())
    assert hashlib.sha256(expected_raw).hexdigest()==provenance['expected_raw_sha256']
    expected=json.loads(expected_raw);inputs=json.loads((ROOT/'inputs.json').read_text(encoding='utf-8'))
    assert len(inputs)==len(expected)==12
    data=ROOT.parents[2]/'crates/models/vegetation/fluspect-cx/data'
    metadata=json.loads((data/'provenance.json').read_text(encoding='utf-8'))
    raw=(data/'optics.csv.zlib').read_bytes()
    assert hashlib.sha256(raw).hexdigest()==metadata['sha256']
    assert hashlib.sha256(zlib.decompress(raw)).hexdigest()==metadata['raw_sha256']
    for result in expected:
        assert len(result['reflectance'])==len(result['transmittance'])==2001
        for key in ['fluorescence_backward','fluorescence_forward']:
            assert len(result[key])==53 and all(len(row)==71 for row in result[key])
    print('OK FLUSPECT-CX: 12 original MATLAB spectra and full 53x71 matrices; optics and fixtures verified')

if __name__=='__main__':verify()
