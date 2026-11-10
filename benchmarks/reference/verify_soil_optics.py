"""Offline integrity check for independent soil/surface model evidence."""
from pathlib import Path
import hashlib,json,tomllib
ROOT=Path(__file__).resolve().parents[2]
for name in ['gsv','bsm','rpv','soilspect']:
    folder=Path(__file__).parent/name
    manifest=json.loads((folder/'source-manifest.json').read_text(encoding='utf8'))
    for f,expected in manifest['evidence_sha256'].items():
        assert hashlib.sha256((folder/f).read_bytes()).hexdigest()==expected,(name,f)
    fixture=json.loads((folder/'reference.json').read_text(encoding='utf8'))
    assert fixture['cases']
    path=ROOT/'crates/models'/('surface' if name=='rpv' else 'soil')/name
    model=tomllib.loads((path/'model.toml').read_text(encoding='utf8'))
    resources={'gsv':{'dry_vectors':'DryVec.txt','moisture_vector':'SMVec.txt'},'bsm':{'bsm_coefficients':'BSM_coef.dat'}}.get(name,{})
    for key,file in resources.items():
        assert model['data']['hashes'][key]=='sha256:'+hashlib.sha256((path/'data'/file).read_bytes()).hexdigest()
    print(f'{name}: {len(fixture["cases"])} reference cases, fixture/source evidence and resources verified')
