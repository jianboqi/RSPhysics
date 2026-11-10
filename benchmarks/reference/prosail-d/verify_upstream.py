"""Offline data/fixture identity checks; optionally check the disposable author sources."""
from pathlib import Path
import hashlib
import json
import math
import argparse
ROOT=Path(__file__).resolve().parents[3]
COMMIT='46549958295b605ac1e211a693d4b2287c3a5a13'
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--require-cache',action='store_true');args=parser.parse_args()
    folder=Path(__file__).resolve().parent
    provenance=json.loads((folder/'upstream-provenance.json').read_text())
    assert provenance['commit']==COMMIT and provenance['version']=='3.1.1'
    for name,record in provenance['files'].items():
        assert record['url']==f'https://raw.githubusercontent.com/jbferet/prosail/{COMMIT}/{name}'
    data=ROOT/'crates/models/vegetation/prosail-d/data/soil.tsv'
    assert hashlib.sha256(data.read_bytes()).hexdigest()==provenance['files']['data-raw/dataSpec_SOIL_ATM.txt']['sha256']
    suite=json.loads((folder/'expected.json').read_text())
    assert suite['canopy_upstream_commit']==provenance['commit']
    assert suite['soil_sha256']==hashlib.sha256(data.read_bytes()).hexdigest()
    assert suite['leaf_coefficients_sha256']==hashlib.sha256((ROOT/'crates/models/vegetation/prospect-d/data/coefficients.tsv').read_bytes()).hexdigest()
    assert suite['leaf_upstream_commit']=='4649d5006ddf6bce609951765a88ba2498fcd8ab'
    assert suite['wavelengths_nm']==list(range(400,2501))
    assert len(suite['cases'])==25
    assert sum(c['expected_source']=='author_r_fourSAIL' for c in suite['cases'])==24
    assert [{k:c[k] for k in ('name','parameters','expected_source')} for c in suite['cases']]==json.loads((folder/'inputs.json').read_text())
    for case in suite['cases']:
        assert case['expected_source']==('analytic_bare_soil_limit_not_fourSAIL' if case['parameters']['leaf_area_index']==0 else 'author_r_fourSAIL')
        for key in ('brf','hemispherical_reflectance'):
            assert len(case[key])==2101 and all(math.isfinite(v) and v>=0 for v in case[key])
        assert all(v<=1 for v in case['hemispherical_reflectance'])
    cache=ROOT/'.reference-cache/prosail-d/upstream'
    if args.require_cache or cache.exists():
        for name,record in provenance['files'].items():
            source=(cache/name).resolve()
            assert source.is_relative_to(cache.resolve())
            assert source.stat().st_size==record['bytes']
            assert hashlib.sha256(source.read_bytes()).hexdigest()==record['sha256'],name
        description=(cache/'DESCRIPTION').read_text()
        assert 'Version: 3.1.1' in description and 'License: MIT + file LICENSE' in description
    print('PROSAIL sources/data/25 offline cases verified; disposable cache '+('checked' if cache.exists() else 'absent'))
if __name__=='__main__':main()
