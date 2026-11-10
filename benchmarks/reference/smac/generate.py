"""Execute pinned, unmodified upstream SMAC; production Rust is never imported."""
import hashlib
import contextlib
import io
import importlib.util
import json
from pathlib import Path
import random
import sys
import warnings
import zlib

ROOT=Path(__file__).resolve().parents[3]
CACHE=ROOT/'.reference-cache/smac/upstream'
FILE=CACHE/'sen2like/sen2like/sen2like/atmcor/smac/smac.py'
PACK=ROOT/'crates/models/atmosphere/smac/data/coefficients.json.zlib'
DEST=Path(__file__).parent

def generate():
    spec=importlib.util.spec_from_file_location('original_smac',FILE)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    pack=json.loads(zlib.decompress(PACK.read_bytes())); records={r['id']:r for r in pack['records']}
    captured={}
    def profile(frame,event,arg):
        if event=='return' and frame.f_code.co_name in ('smac_dir','smac_inv'):
            captured.update({key:float(frame.f_locals[key]) for key in ['tg','ttetas','ttetav','s','atm_ref','ray_ref','aer_ref']})
    defaults=dict(coefficient_id='sentinel-2a.msi.B04.continental',direction='toa_to_surface',reflectance=0.2,solar_zenith_deg=30.,view_zenith_deg=5.,relative_azimuth_deg=60.,pressure_hpa=1013.25,aot550=0.1,ozone_atm_cm=0.3,water_vapour_g_cm2=2.)
    def call(p):
        rec=records[p['coefficient_id']]; raw=pack['sources'][rec['source_key']]['text']
        target=CACHE/'execution'/rec['source_key'];target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw.encode('ascii'))
        c=module.coeff(str(target),rec['band'] if rec['column'] is not None else None)
        f=module.smac_inv if p['direction']=='toa_to_surface' else module.smac_dir
        captured.clear();sys.setprofile(profile)
        try:
            with warnings.catch_warnings(), contextlib.redirect_stdout(io.StringIO()):
                warnings.simplefilter('error')
                result=float(f(p['reflectance'],p['solar_zenith_deg'],p['relative_azimuth_deg'],p['view_zenith_deg'],0.,p['pressure_hpa'],p['aot550'],p['ozone_atm_cm'],p['water_vapour_g_cm2'],c))
        finally:sys.setprofile(None)
        return dict(reflectance=result,gas_transmittance=captured['tg'],downward_scattering_transmittance=captured['ttetas'],upward_scattering_transmittance=captured['ttetav'],spherical_albedo=captured['s'],atmospheric_reflectance=captured['atm_ref'],rayleigh_reflectance=captured['ray_ref'],aerosol_reflectance=captured['aer_ref'])
    cases=[]
    # Default first, then every distributed coefficient in both directions.
    candidates=[('default',defaults)]
    for key in records:
        for direction in ['surface_to_toa','toa_to_surface']:
            candidates.append((key+'/'+direction,{**defaults,'coefficient_id':key,'direction':direction}))
    rng=random.Random(1994)
    common=[k for k in records if k.startswith(('sentinel-','landsat-','modis.')) and '.B10.' not in k]
    for i in range(96):
        candidates.append((f'random-{i}',{**defaults,'coefficient_id':rng.choice(common),'direction':rng.choice(['surface_to_toa','toa_to_surface']),'reflectance':rng.uniform(0.01,0.8),'solar_zenith_deg':rng.uniform(0,65),'view_zenith_deg':rng.uniform(0,60),'relative_azimuth_deg':rng.uniform(0,180),'pressure_hpa':rng.uniform(600,1050),'aot550':rng.uniform(0.01,0.8),'ozone_atm_cm':rng.uniform(0.15,0.5),'water_vapour_g_cm2':rng.uniform(0.1,5)}))
    for aot in [0,1]:
        for angle in [0,70]:
            for direction in ['surface_to_toa','toa_to_surface']:
                candidates.append((f'boundary-{aot}-{angle}-{direction}',{**defaults,'aot550':aot,'solar_zenith_deg':angle,'view_zenith_deg':angle,'direction':direction,'water_vapour_g_cm2':0.,'ozone_atm_cm':0.}))
    for name,p in candidates:
        expected=call(p)
        cases.append({'name':name,'input':p,'expected':expected})
    curves=[]
    for direction in ['toa_to_surface','surface_to_toa']:
        p={**defaults,'direction':direction}
        curves.append({'input':p,'sweep_aot550':[i/20 for i in range(21)],'sweep_reflectance':[call({**p,'aot550':i/20})['reflectance'] for i in range(21)]})
    result=dict(source='Unmodified ESA Sen2Like SMAC (Olivier Hagolle / CNES)',source_sha256=hashlib.sha256(FILE.read_bytes()).hexdigest(),coefficient_pack_sha256=hashlib.sha256(PACK.read_bytes()).hexdigest(),tolerance={'absolute':2e-11,'relative':2e-10},cases=cases,curves=curves)
    raw=(json.dumps(result,ensure_ascii=True,indent=2,allow_nan=False)+'\n').encode()
    (DEST/'expected.json').write_bytes(raw)
    (DEST/'fixture-sha256.txt').write_text(hashlib.sha256(raw).hexdigest()+'  expected.json\n',encoding='ascii')
    print(f'{len(cases)} original-reference cases; {len(curves)} complete curves')

if __name__=='__main__':generate()
