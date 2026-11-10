"""Compare supplied SORT Python source with frozen full-spectrum fixtures.

Run in an environment with the reference source's NumPy/SciPy/Pandas/Matplotlib
dependencies: python tooling/reference-runner/verify_sort_source.py --source DIR
The supplied source is kept outside the published source tree.
"""
import argparse,hashlib,json,sys
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('--source',type=Path,required=True)
args=parser.parse_args()
source=args.source.resolve();root=Path(__file__).resolve().parents[2]
provenance=json.loads((root/'benchmarks/reference/sort/provenance.json').read_text(encoding='utf-8'))
for relative,want in provenance['source_hashes'].items():
    assert hashlib.sha256((source/relative).read_bytes()).hexdigest()==want,relative
sys.path.insert(0,str(source))
from forward import SORTParameters,SORTLParameters,simulate_sort,simulate_sort_l
import numpy as np

mapping={'soil_brightness':'B','spectral_alpha_deg':'alpha_deg','spectral_gamma_deg':'gamma_deg','water_coverage':'eps','water_film_thickness_mm':'L','particle_volume_fraction':'d_i','particle_refractive_index':'n_i','particle_extinction_coefficient':'k_i','phase_asymmetry':'b','phase_backscatter':'c','hotspot_amplitude':'B0','hotspot_width':'h','solid_filling_factor':'F','rms_slope':'M','multifacet_reflectance':'r0'}
for slug,parameters,simulate in [('sort',SORTParameters,simulate_sort),('sort-l',SORTLParameters,simulate_sort_l)]:
    folder=root/'benchmarks/reference'/slug
    inputs=json.loads((folder/'inputs.json').read_text(encoding='utf-8'))
    expected=json.loads((folder/'expected.json').read_text(encoding='utf-8'))['cases']
    maximum=0.0
    for case,want in zip(inputs,expected):
        out=simulate(parameters(**{mapping.get(k,k):v for k,v in case['parameters'].items()}))
        for key,value in out.items():
            wanted=np.asarray(want['wavelengths_nm' if key=='wavelength_nm' else key])
            maximum=max(maximum,float(np.max(np.abs(value-wanted))))
            np.testing.assert_allclose(value,wanted,rtol=2e-9,atol=2e-9)
    print(f'{slug}: {len(inputs)} complete reference spectra, max absolute difference {maximum:.3g}')
