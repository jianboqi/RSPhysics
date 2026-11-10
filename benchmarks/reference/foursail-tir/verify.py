"""Check pinned 4SAIL-TIR fixtures without external numerical dependencies."""
from pathlib import Path
import hashlib,json,math,zlib
ROOT=Path(__file__).resolve().parent
def finite(v):
    if isinstance(v,float):assert math.isfinite(v)
    elif isinstance(v,list):
        for item in v:finite(item)
    elif isinstance(v,dict):
        for item in v.values():finite(item)
def main():
    manifest=json.loads((ROOT/'manifest.json').read_text())
    for name,want in manifest['files'].items():
        p=(ROOT/name).resolve();assert p.is_relative_to(ROOT)
        raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==want,name
        if p.suffix=='.zlib':raw=zlib.decompress(raw)
        if '.json' in p.name:finite(json.loads(raw))
    suite=json.loads((ROOT/'expected.json').read_text());assert len(suite['cases'])==28
    for c in suite['cases']:
        o=c['result'];assert abs(o['leaf_emission_weight']+o['soil_emission_weight']+o['sky_reflectance']-1)<2e-8
    assert len(suite['default_arrays']['principal_brightness_temperature_k'])==171
    assert len(suite['default_arrays']['hemisphere_brightness_temperature_k'])==648
    assert len(suite['default_arrays']['wavelengths_nm'])==61
    print('OK 4SAIL-TIR: 28 independent ODE cases, 819 directions, 61 wavelengths; hashes and Kirchhoff closure')
if __name__=='__main__':main()
