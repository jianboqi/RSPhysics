"""80-digit published-equation oracles; optional unmodified SenSE canopy reference.

No RSPhysics module, Rust output or production implementation is imported.
Run with mpmath==1.3.0, numpy (only the SenSE comparison).
"""
import ast
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import random
from typing import Dict, Union
import mpmath as mp

ROOT=Path(__file__).resolve().parents[3]
mp.mp.dps=80
DEFAULTS={
 'tau-omega':dict(soil_temperature_k=300.,vegetation_temperature_k=295.,soil_reflectivity=.3,vegetation_optical_depth=.3,single_scattering_albedo=.05,observation_zenith_deg=40.),
 'water-cloud':dict(vegetation_water_content=1.,scattering_coefficient=.1,attenuation_coefficient=.2,soil_backscatter=.1,observation_zenith_deg=40.),
}
TOTAL={'tau-omega':'brightness_temperature_k','water-cloud':'sigma0'}

def precise(slug,p):
 p={k:mp.mpf(str(v)) for k,v in p.items()};c=mp.cos(p['observation_zenith_deg']*mp.pi/180)
 if slug=='tau-omega':
  gamma=mp.exp(-p['vegetation_optical_depth']/c)
  soil=p['soil_temperature_k']*(1-p['soil_reflectivity'])*gamma
  # Split the two canopy paths independently, as in ATBD Eq.1.
  veg=p['vegetation_temperature_k']*(1-p['single_scattering_albedo'])*(1-gamma)
  reflected=veg*p['soil_reflectivity']*gamma
  return dict(brightness_temperature_k=float(soil+veg+reflected),soil_emission_k=float(soil),vegetation_emission_k=float(veg),reflected_vegetation_emission_k=float(reflected),transmissivity=float(gamma))
 t=mp.exp(-2*p['attenuation_coefficient']*p['vegetation_water_content']/c)
 veg=p['scattering_coefficient']*p['vegetation_water_content']*c*(1-t)
 soil=p['soil_backscatter']*t
 return dict(sigma0=float(veg+soil),vegetation_backscatter=float(veg),attenuated_soil_backscatter=float(soil),two_way_transmissivity=float(t))

def cases(slug):
 out=[]
 def add(name,**kw):out.append(dict(name=name,parameters=DEFAULTS[slug]|kw))
 add('default');add('nadir',observation_zenith_deg=0);add('angle-limit',observation_zenith_deg=80)
 if slug=='tau-omega':
  add('bare-soil',vegetation_optical_depth=0)
  add('black-soil',soil_reflectivity=0);add('reflecting-soil',soil_reflectivity=1)
  add('no-scattering',single_scattering_albedo=0);add('no-canopy-emission',single_scattering_albedo=1)
  add('opaque-canopy',vegetation_optical_depth=5,observation_zenith_deg=80)
  add('isothermal-black',soil_reflectivity=0,single_scattering_albedo=0,soil_temperature_k=300,vegetation_temperature_k=300)
  add('weak-canopy',vegetation_optical_depth=1e-15)
  add('temperature-limits',soil_temperature_k=150,vegetation_temperature_k=350)
  ranges=dict(soil_temperature_k=(150,350),vegetation_temperature_k=(150,350),soil_reflectivity=(0,1),vegetation_optical_depth=(0,5),single_scattering_albedo=(0,1),observation_zenith_deg=(0,80))
 else:
  add('bare-soil',vegetation_water_content=0);add('transparent-canopy',attenuation_coefficient=0)
  add('no-vegetation-scattering',scattering_coefficient=0);add('no-soil',soil_backscatter=0)
  add('zero-signal',soil_backscatter=0,scattering_coefficient=0)
  add('large-signal',vegetation_water_content=10,scattering_coefficient=2,attenuation_coefficient=2,soil_backscatter=10)
  add('weak-canopy',attenuation_coefficient=1e-15)
  add('very-small-water',vegetation_water_content=1e-14)
  add('maximum-angle-opacity',observation_zenith_deg=80,vegetation_water_content=10,attenuation_coefficient=2)
  ranges=dict(vegetation_water_content=(0,10),scattering_coefficient=(0,2),attenuation_coefficient=(0,2),soil_backscatter=(0,10),observation_zenith_deg=(0,80))
 rng=random.Random(20261109)
 for i in range(20):add(f'seeded-{i:02d}',**{k:rng.uniform(*v) for k,v in ranges.items()})
 return out

def sense_class():
 import numpy as np
 source=ROOT/'.reference-cache/water-cloud/upstream/model.py'
 if hashlib.sha256(source.read_bytes()).hexdigest()!='1131ace6d3d0c1ec8776fe2fb03bb226de3d956ae00c3cc5b0d095d449fc6fa3':raise ValueError('SenSE source hash mismatch')
 tree=ast.parse(source.read_text())
 cls=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name=='WaterCloudCanopy')
 # Execute this original class verbatim, with its original decorators/NumPy methods.
 # Avoid importing other SenSE models and their unrelated optional dependencies.
 namespace=dict(dataclass=dataclass,np=np,Union=Union,Dict=Dict)
 exec(compile(ast.Module(body=[cls],type_ignores=[]),str(source),'exec'),namespace)
 return namespace['WaterCloudCanopy']

def main():
 if mp.__version__!='1.3.0':raise ValueError('mpmath must be 1.3.0')
 cls=sense_class()
 import numpy as np
 for slug in DEFAULTS:
  kind='independent 80-digit NASA ATBD equations; not original NASA operational code' if slug=='tau-omega' else 'independent 80-digit published equations plus SenSE canopy code; not original 1978 author code'
  suite=dict(suite=slug+'-published-equations-v1',reference_kind=kind,precision_digits=80,mpmath_version=mp.__version__,numpy_version=np.__version__,tolerance=dict(absolute=2e-12,relative=2e-12),cases=[])
  for case in cases(slug):
   p=case['parameters'];out=precise(slug,p);out['sweep_zenith_deg']=list(range(81));out['sweep_total']=[precise(slug,p|dict(observation_zenith_deg=a))[TOTAL[slug]] for a in range(81)]
   case['expected']=out
   if slug=='water-cloud':
    a,b,w,theta=p['scattering_coefficient'],p['attenuation_coefficient'],p['vegetation_water_content'],np.deg2rad(p['observation_zenith_deg'])
    upstream=cls(a,b,a,b,a,b,w,w,theta)
    case['sense_canopy']=dict(vegetation_backscatter=float(upstream.sigma_c()['hh']),two_way_transmissivity=float(upstream.tau_h))
    angles=np.deg2rad(np.arange(81));sweep=cls(a,b,a,b,a,b,w,w,angles)
    case['sense_canopy_sweep']=dict(vegetation_backscatter=sweep.sigma_c()['hh'].tolist(),two_way_transmissivity=sweep.tau_h.tolist())
   suite['cases'].append(case)
  folder=ROOT/'benchmarks/reference'/slug;folder.mkdir(parents=True,exist_ok=True)
  (folder/'inputs.json').write_text(json.dumps([dict(name=c['name'],parameters=c['parameters']) for c in suite['cases']],indent=2)+'\n')
  (folder/'expected.json').write_text(json.dumps(suite,indent=2)+'\n')
  print(slug,len(suite['cases']),'cases generated')
if __name__=='__main__':main()
