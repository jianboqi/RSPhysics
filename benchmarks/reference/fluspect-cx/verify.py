"""Offline integrity checks for the pinned original FLUSPECT-CX numerical cases."""
from pathlib import Path
import hashlib,json,zlib

ROOT=Path(__file__).resolve().parent

def verify():
    manifest=json.loads((ROOT/'manifest.json').read_text(encoding='utf-8'))
    assert set(manifest)=={'inputs.json','expected.json.zlib','full-inputs.json','full-expected.json.zlib','negative-emission-evidence.json.zlib','provenance.json'}
    for name,want in manifest.items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==want,name
    provenance=json.loads((ROOT/'provenance.json').read_text(encoding='utf-8'))
    expected_raw=zlib.decompress((ROOT/'expected.json.zlib').read_bytes())
    assert hashlib.sha256(expected_raw).hexdigest()==provenance['expected_raw_sha256']
    expected=json.loads(expected_raw);inputs=json.loads((ROOT/'inputs.json').read_text(encoding='utf-8'))
    assert len(inputs)==len(expected)==12
    data=ROOT.parents[2]/'crates/models/vegetation/fluspect-cx/data'
    metadata=json.loads((data/'provenance.json').read_text(encoding='utf-8'))
    for table in metadata['tables'].values():
        raw=(data/table['file']).read_bytes()
        assert hashlib.sha256(raw).hexdigest()==table['sha256']
        assert hashlib.sha256(zlib.decompress(raw)).hexdigest()==table['raw_sha256']
    for result in expected:
        assert len(result['reflectance'])==len(result['transmittance'])==2001
        for key in ['fluorescence_backward','fluorescence_forward']:
            assert len(result[key])==53 and all(len(row)==71 for row in result[key])
    full=json.loads(zlib.decompress((ROOT/'full-expected.json.zlib').read_bytes()))
    full_inputs=json.loads((ROOT/'full-inputs.json').read_text(encoding='utf-8'))
    assert len(full)==len(full_inputs)==8
    for params,result in zip(full_inputs,full):
        nf,ne=(211,351) if params['spectral_resolution']=='full_1nm' else (53,71)
        for key in ['fluorescence_backward','fluorescence_forward']:
            assert len(result[key])==nf and all(len(row)==ne for row in result[key])
            assert all(v>=0 for row in result[key] for v in row)
    evidence=json.loads(zlib.decompress((ROOT/'negative-emission-evidence.json.zlib').read_bytes()))
    assert evidence and all(r['wavelength_nm']==641 for r in evidence)
    print('OK FLUSPECT-CX: 12 light + 8 full/PRO cases; original negative-emission evidence and both optical tables verified')

if __name__=='__main__':verify()
