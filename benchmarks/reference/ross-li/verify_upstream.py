"""Verify pinned provenance and offline dual-oracle fixtures without a C runtime."""
import argparse,hashlib,json,math
from pathlib import Path
from prepare import HASHES,COMMIT
ROOT=Path(__file__).resolve().parents[3];FOLDER=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--require-cache',action='store_true');args=parser.parse_args()
    provenance=json.loads((FOLDER/'upstream-provenance.json').read_text())
    assert provenance['commit']==COMMIT
    cache=ROOT/'.reference-cache/ross-li/upstream'
    for name,expected in HASHES.items():
        assert provenance['files'][name]['sha256']==expected
        assert provenance['files'][name]['url']==f'https://raw.githubusercontent.com/arthurelmes/ambrals_forward/{COMMIT}/{name}'
        if args.require_cache or cache.exists():assert hashlib.sha256((cache/name).read_bytes()).hexdigest()==expected
    suite=json.loads((FOLDER/'expected.json').read_text())
    assert suite['commit']==COMMIT and suite['high_precision']==dict(library='mpmath',version='1.3.0',decimal_digits=80)
    assert len(suite['cases'])==32
    assert [{k:c[k] for k in ('name','parameters')} for c in suite['cases']]==json.loads((FOLDER/'inputs.json').read_text())
    count=0;maximum=0
    for case in suite['cases']:
        for key,length in [('principal_brf',171),('hemisphere_brf',648)]:
            assert len(case[key])==length and len(case['original_'+key])==length
        pairs=[(case['point'][k],case['original_point'][k]) for k in case['original_point']]
        pairs+=list(zip(case['principal_brf'],case['original_principal_brf']))+list(zip(case['hemisphere_brf'],case['original_hemisphere_brf']))
        for precise,original in pairs:
            assert math.isfinite(precise) and math.isfinite(original)
            error=abs(precise-original);maximum=max(maximum,error);count+=1
            assert error<=suite['original_tolerance']['absolute']+suite['original_tolerance']['relative']*abs(original)
    assert count==26304
    assert suite['original_vs_precise']['status']=='passed' and suite['original_vs_precise']['max_absolute_error']==maximum
    print('Ross-Li pinned sources and 32 dual-oracle cases verified; original cache '+('present' if cache.exists() else 'absent'))
if __name__=='__main__':main()
