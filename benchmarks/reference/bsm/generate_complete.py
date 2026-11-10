"""Independent published BSM ensemble; SciPy PMF and Fresnel flux quadrature."""
from pathlib import Path
import json
import numpy as np
from scipy.special import roots_legendre
from scipy.stats import poisson
ROOT=Path(__file__).resolve().parents[3]
TABLE=np.loadtxt(ROOT/'crates/models/soil/bsm/data/BSM_coef.dat',skiprows=1)
def tav(angle,n):
    x,w=roots_legendre(96);lo=np.cos(np.deg2rad(angle));mu=lo+(1-lo)*(x+1)/2
    ct=np.sqrt(1-(1-mu[:,None]**2)/n**2)
    rs=((mu[:,None]-n*ct)/(mu[:,None]+n*ct))**2
    rp=((n*mu[:,None]-ct)/(n*mu[:,None]+ct))**2
    return np.sum(w[:,None]/2*mu[:,None]*(2-rs-rp)/(1+lo),axis=0)
def compute(p):
    _,n,kw,g1,g2,g3=TABLE.T
    lat,lon=np.deg2rad([p['shape_latitude_deg'],p['shape_longitude_deg']]);b=p['soil_brightness']
    dry=b*(np.sin(lat)*g1+np.cos(lat)*np.sin(lon)*g2+np.cos(lat)*np.cos(lon)*g3)
    mu=max(0,(p['soil_moisture_percent']-5)/p['moisture_capacity_percent'])
    if mu==0:return dry,dry.copy()
    bac=1-(1-dry)*(dry*tav(90,2/n)/tav(90,np.array([2.]))+1-dry)
    back=1-tav(90,n)/n**2;top=1-tav(40,n)
    k=np.arange(1,101);prob=poisson.pmf(k,mu)
    trans=np.exp(-2*kw[:,None]*p['water_film_thickness_cm']*k)
    film=top[:,None]+((1-top)*(1-back)*bac)[:,None]*trans/(1-(back*bac)[:,None]*trans)
    wet=dry*poisson.pmf(0,mu)+film@prob
    return dry,wet
if __name__=='__main__':
    original=json.loads(Path(__file__).with_name('reference.json').read_text())['cases'];cases=[]
    for case in original:
        p=case['parameters']|{'poisson_mode':'complete'};dry,wet=compute(p)
        cases.append(dict(name=case['name']+'-complete',parameters=p,expected=dict(dry_reflectance=dry.tolist(),wet_reflectance=wet.tolist())))
    Path(__file__).with_name('complete.json').write_text(json.dumps({'cases':cases},indent=2)+'\n',encoding='utf8',newline='\n')
