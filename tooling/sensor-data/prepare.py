"""Rebuild distributable USGS/NASA profiles. ESA assets stay in a local cache.

Run after downloads listed in source-manifest.json are in .reference-cache/sensors/upstream.
No network or third-party parser dependency is required by this transformation.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
MODULE=ROOT/'bindings/python/python/rsphysics/_sensor_io.py'
spec=importlib.util.spec_from_file_location('sensor_io',MODULE);io=importlib.util.module_from_spec(spec);spec.loader.exec_module(io)
def main():
    source=Path(__file__).parent/'source-manifest.json'
    entries=json.loads(source.read_text())
    dest=ROOT/'bindings/python/python/rsphysics/data/sensors';dest.mkdir(parents=True,exist_ok=True)
    for entry in entries:
        path=ROOT/'.reference-cache/sensors/upstream'/entry['source']['filename']
        assert hashlib.sha256(path.read_bytes()).hexdigest()==entry['source']['sha256'],path
        data=(json.dumps(io.parse(path,entry),ensure_ascii=False,separators=(',',':'))+'\n').encode()
        entry['processed_sha256']=hashlib.sha256(data).hexdigest()
        entry['asset']=entry['id'].replace('.','-')+'.json'
        if entry['distribution']=='bundled':(dest/entry['asset']).write_bytes(data)
        else:
            local=ROOT/'target/sensor-runtime-cache';local.mkdir(exist_ok=True);(local/entry['asset']).write_bytes(data)
        print(entry['id'],len(entry['band_ids']),entry['distribution'],entry['processed_sha256'])
    (dest/'registry.json').write_text(json.dumps(entries,indent=2)+'\n')
if __name__=='__main__':main()
