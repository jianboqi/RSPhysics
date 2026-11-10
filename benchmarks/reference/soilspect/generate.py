"""Independent NumPy/SciPy evaluation of the published SOILSPECT equations.

No RSPhysics code is imported. Refined flux integrals use 96-point mu and
192-point azimuth Gauss-Legendre quadrature (the Rust solver uses 32/64).
"""
from pathlib import Path
import json
import numpy as np
from scipy.special import roots_legendre

def point(p,ci,ce,phi):
    product=np.sqrt(np.maximum(0,(1-ci*ci)*(1-ce*ce)))*np.cos(phi)
    g=np.clip(ci*ce+product,-1,1);gp=np.clip(ci*ce-product,-1,1)
    phase=1+p['phase_b']*g+p['phase_c']*(3*g*g-1)/2+p['phase_b_prime']*gp+p['phase_c_prime']*(3*gp*gp-1)/2
    h=p['roughness_h']
    with np.errstate(divide='ignore',invalid='ignore'):
        bg=np.where(g==1,1.,0.) if h==0 else h/(h+np.sqrt((1-g)/(1+g)))
    w=p['single_scattering_albedo'];gamma=np.sqrt(1-w)
    single=w/(4*(ci+ce))*(1+bg)*phase
    multiple=w/(4*(ci+ce))*((1+2*ci)/(1+2*gamma*ci)*(1+2*ce)/(1+2*gamma*ce)-1)
    return single+multiple,single,multiple

def integrals(p,n=96):
    x,w=roots_legendre(n);mu=(x+1)/2;w=w/2
    a,aw=roots_legendre(2*n);phi=(a+1)*np.pi/2;aw=aw/2
    ce=mu[:,None];ph=phi[None,:]
    def d(ci):return float(2*np.sum(w*mu*np.sum(point(p,ci,ce,ph)[0]*aw,axis=1)))
    direct=d(np.cos(np.deg2rad(p['solar_zenith_deg'])))
    diffuse=float(2*sum(wi*mi*d(mi) for mi,wi in zip(mu,w)))
    return direct,diffuse

def generate():
    base=dict(single_scattering_albedo=.6,roughness_h=.1,phase_b=.3,phase_c=.1,phase_b_prime=0.,phase_c_prime=0.,solar_zenith_deg=30.,view_zenith_deg=20.,relative_azimuth_deg=0.)
    cases=[]
    variants=[{},dict(single_scattering_albedo=0.),dict(single_scattering_albedo=1.),dict(single_scattering_albedo=.75),dict(roughness_h=0.),dict(roughness_h=1.),dict(roughness_h=.01),dict(phase_b_prime=.15,phase_c_prime=.2),dict(solar_zenith_deg=0.,view_zenith_deg=0.),dict(solar_zenith_deg=50.,view_zenith_deg=50.),dict(solar_zenith_deg=85.,view_zenith_deg=85.,relative_azimuth_deg=180.),dict(phase_b=-.8,phase_c=.4,phase_b_prime=.3,phase_c_prime=-.2)]
    for i,v in enumerate(variants):
        p=base|v;brf,single,multiple=point(p,np.cos(np.deg2rad(p['solar_zenith_deg'])),np.cos(np.deg2rad(p['view_zenith_deg'])),np.deg2rad(p['relative_azimuth_deg']))
        dh,bh=integrals(p)
        curve=point(p,np.cos(np.deg2rad(p['solar_zenith_deg'])),np.cos(np.deg2rad(np.abs(np.arange(-85,86)))),np.where(np.arange(-85,86)<0,np.pi,0))[0]
        cases.append(dict(name=f'soilspect-{i}',parameters=p,expected=dict(brf=float(brf),brdf=float(brf/np.pi),single_scattering_brf=float(single),multiple_scattering_brf=float(multiple),directional_hemispherical_reflectance=dh,bihemispherical_reflectance=bh),principal_brf=curve.tolist()))
    return {'cases':cases}
if __name__=='__main__':
    Path(__file__).with_name('reference.json').write_text(json.dumps(generate(),indent=2)+'\n',encoding='utf8',newline='\n')
