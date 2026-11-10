"""Verify full SORT roughness/film operators using separately supplied Python source.

Requires the author's NumPy/SciPy/Pandas/Matplotlib dependencies. Runtime
RSPhysics packages do not depend on those reference-only libraries.
"""
from pathlib import Path
import argparse
import hashlib
import json
import sys
import zlib

ROOT=Path(__file__).resolve().parents[2]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',type=Path,required=True);args=parser.parse_args()
    source=args.source.resolve();folder=ROOT/'benchmarks/reference/sort'
    provenance=json.loads((folder/'provenance.json').read_text(encoding='utf-8'))
    for relative,want in provenance['source_hashes'].items():
        original=(source/relative).resolve();assert original.is_relative_to(source)
        assert hashlib.sha256(original.read_bytes()).hexdigest()==want,relative
    sys.path.insert(0,str(source))
    import numpy as np
    from sort_kernel import full as f
    from sort_kernel.layer import adding
    maximum=0.
    def close(got,want):
        nonlocal maximum
        np.testing.assert_allclose(got,want,rtol=2e-9,atol=2e-9)
        maximum=max(maximum,float(np.max(np.abs(np.asarray(got)-want))))
    roughness=json.loads((folder/'roughness-expected.json').read_text(encoding='utf-8'))
    for c in roughness:
        p=f.SORTParameters(M=c['rms_slope'],r0=c['multifacet_reflectance'],roughness_correction=c['mode'])
        i,e,az=np.deg2rad(c['geometry_deg'])
        values=f._shiltz_single_scalar_geometry(np.array(c['ssa']),p,i,e,az,p.M,integration_points=200)
        if c['mode']=='shiltz_full':
            phase=np.arccos(np.cos(i)*np.cos(e)+np.sin(i)*np.sin(e)*np.cos(az))
            values+=.19*p.M*p.r0*(1+6.5*np.exp(-4/np.pi*(np.pi-phase)**2))
        close(values,c['expected'])
    layers=json.loads((folder/'layer-expected.json').read_text(encoding='utf-8'))
    for c in layers:
        mode=c['mode'];p=f.SORTParameters(roughness_correction=mode,M=.2,r0=.2,F=.05,b=.2,c=0,B0=.1,h=.2)
        def boundary(i,e,a):
            if mode.startswith('hapke'):return f._hapke_rough_reflectance(c['ssa'],p,i,e,a,mode=mode)[0]
            return f._sort_bidirectional_kernel(i,e,a,single_scattering_albedo=c['ssa'],b=p.b,c=p.c,F=p.F,B0=p.B0,h=p.h)
        got,_=adding(boundary,c['index'],c['tau'],*np.deg2rad([25.,15.,60.]));close(got,c['expected'])
    data=f.load_sort_spectral_data(str(ROOT/'crates/models/soil/sort/data/soil_basis.csv'))
    spectra=json.loads(zlib.decompress((folder/'extended-expected.json.zlib').read_bytes()))
    for c in spectra:
        dry,wet=f.sort_forward(data,f.SORTParameters(**c['parameters']),return_dry=True)
        close(dry,c['dry_brf']);close(wet,c['wet_brf'])
    print(f'SORT complete source: {len(roughness)} slope cases, {len(layers)} angular adding cases, {len(spectra)} full spectra, maximum error {maximum:.3g}')


if __name__=='__main__':main()
