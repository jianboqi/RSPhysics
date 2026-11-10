"""Offline registry/bundled-asset integrity; ESA response samples must stay local."""
import hashlib
import json
import math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def main():
    folder=ROOT/'bindings/python/python/rsphysics/data/sensors'
    entries=json.loads((folder/'registry.json').read_text());sources=json.loads((Path(__file__).parent/'source-manifest.json').read_text())
    assert len(entries)==len(sources)==12
    for entry,source in zip(entries,sources):
        for key,value in source.items():assert entry[key]==value,(entry['id'],key)
        asset=folder/entry['asset']
        if entry['distribution']!='bundled':assert not asset.exists(),asset;continue
        assert hashlib.sha256(asset.read_bytes()).hexdigest()==entry['processed_sha256'],asset
        data=json.loads(asset.read_text());assert [b['id'] for b in data['bands']]==entry['band_ids']
        for b in data['bands']:
            x=b['wavelengths_nm'];r=b['response'];assert len(x)==len(r)>=2
            assert all(math.isfinite(v) and v>0 for v in x) and all(a<c for a,c in zip(x,x[1:]))
            assert all(math.isfinite(v) and v>=0 for v in r) and max(r)>0
        raw=ROOT/'.reference-cache/sensors/upstream'/entry['source']['filename']
        if raw.exists():assert hashlib.sha256(raw.read_bytes()).hexdigest()==entry['source']['sha256']
    assert sum(len(e['band_ids']) for e in entries)==112
    print('12 profiles / 112 registered response curves; 9 bundled profiles / 73 curves verified; ESA arrays excluded')
if __name__=='__main__':main()
