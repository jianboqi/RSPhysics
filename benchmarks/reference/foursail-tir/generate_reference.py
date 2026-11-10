"""Independent 4SAIL thermal oracle: matrix exponentials of the transfer ODE.

Uses original MIT R geometric coefficients, never the Rust transport solution.
The original 20-segment exponential hotspot integration defines the numerical
joint-gap convention; the diffuse flux and observer integral use scipy.expm.
No reference sources or runtime dependencies are needed to run pinned fixtures.
"""
from pathlib import Path
import argparse,json,math,zlib,hashlib
import numpy as np
import scipy
from scipy.linalg import expm
from scipy.constants import h,c,k

def planck(w,t):
    if t==0:return 0.
    w=w*1e-9
    return 2*h*c*c/w**5/math.expm1(h*c/(w*k*t))*1e-6

def brightness(w,l):
    if l==0:return 0.
    w=w*1e-9
    return h*c/(w*k*math.log1p(2*h*c*c/w**5*1e-6/l))

def joint_gap(p,g):
    ks,ko,_,_,_,dso=g;l=p['leaf_area_index'];tss=math.exp(-ks*l)
    if l==0:return 1.,0.
    alpha=min(200.,2*dso/p['hotspot']/(ks+ko) if p['hotspot']>0 else 1e6)
    if alpha==0:return tss,-math.expm1(-ks*l)/ks
    nodes=np.r_[0.,-np.log1p(-np.arange(1,20)*(-math.expm1(-alpha)/20))/alpha,1.]
    logp=-(ks+ko)*l*nodes+l*math.sqrt(ks*ko)*(-np.expm1(-alpha*nodes))/alpha
    integral=0.
    for i in range(20):
        delta=logp[i+1]-logp[i]
        integral+=(nodes[i+1]-nodes[i])*math.exp(logp[i])*(math.expm1(delta)/delta if delta else 1.)
    return math.exp(logp[-1]),l*integral

def basis_response(p,g,b,solar=0.):
    ks,ko,bf,sob,_,_=g;l=p['leaf_area_index'];rho=1-p['leaf_emissivity'];rs=1-p['soil_emissivity'];eps=p['leaf_emissivity'];es=p['soil_emissivity']
    sb=(ks+bf)*rho/2;sf=(ks-bf)*rho/2;vb=(ko+bf)*rho/2;vf=(ko-bf)*rho/2
    sigb=(1+bf)*rho/2;att=1-(1-bf)*rho/2
    beam=solar/math.pi;tss=math.exp(-ks*l);too=math.exp(-ko*l)
    qs=eps*(b[0]-b[1]);qc=eps*b[1]
    m=np.array([[-att,sigb,qs+sf*beam,qc],[-sigb,att,-qs-sb*beam,-qc],[0,0,-ks,0],[0,0,0,0.]])
    f=expm(m*l)
    # D(0)=sky; U(L)=rho_s[D(L)+direct(L)]+thermal_soil(L).
    boundary=es*(b[3]+tss*(b[2]-b[3]))+rs*tss*beam
    u0=(boundary+rs*(f[0,0]*b[4]+f[0,2]+f[0,3])-(f[1,0]*b[4]+f[1,2]+f[1,3]))/(f[1,1]-rs*f[0,1])
    state=np.array([b[4],u0,1.,1.]);bottom=f@state
    # Integrate e^(-ko*x) [vb*D(x)+vf*U(x)] with a fifth augmented state.
    observer=np.zeros((5,5));observer[:4,:4]=m-np.eye(4)*ko;observer[4,:2]=[vb,vf]
    scattered=(expm(observer*l)@np.r_[state,0.])[4]
    joint,integral=joint_gap(p,g)
    radiance=(scattered+eps*b[1]*(1-too)+(ko*qs+sob*rho*beam)*integral
              +too*(rs*bottom[0]+es*b[3])+joint*(es*(b[2]-b[3])+rs*beam))
    down=math.pi*bottom[0]+tss*solar
    up=rs*down+math.pi*es*(b[3]+tss*(b[2]-b[3]))
    return np.array([radiance,math.pi*u0,down,up])

def coefficients(p,g):
    responses=np.column_stack([basis_response(p,g,np.eye(5)[i]) for i in range(5)]+[basis_response(p,g,np.zeros(5),1.)])
    # A uniform unit radiance field is an equilibrium solution of the ODE.
    assert abs(responses[0,:5].sum()-1)<2e-8
    assert abs(responses[1,:5].sum()-math.pi)<2e-8
    return responses

def point(p,g):
    a=coefficients(p,g);weights=a[0,:4]
    b=np.array([planck(p['wavelength_nm'],p[n]) for n in ('sunlit_leaf_temperature_k','shaded_leaf_temperature_k','sunlit_soil_temperature_k','shaded_soil_temperature_k','sky_brightness_temperature_k')]+[p['direct_irradiance_w_m2_um']])
    values=a@b;parts=a[0]*b
    return dict(radiance_w_m2_sr_um=values[0],brightness_temperature_k=brightness(p['wavelength_nm'],values[0]),
        effective_emissivity=1-a[0,4],leaf_emission_weight=weights[:2].sum(),soil_emission_weight=weights[2:].sum(),
        sunlit_leaf_weight=weights[0],shaded_leaf_weight=weights[1],sunlit_soil_weight=weights[2],shaded_soil_weight=weights[3],
        sky_reflectance=a[0,4],direct_reflectance_factor=a[0,5]*math.pi,
        sunlit_leaf_radiance_w_m2_sr_um=parts[0],shaded_leaf_radiance_w_m2_sr_um=parts[1],sunlit_soil_radiance_w_m2_sr_um=parts[2],shaded_soil_radiance_w_m2_sr_um=parts[3],
        reflected_sky_radiance_w_m2_sr_um=parts[4],reflected_direct_radiance_w_m2_sr_um=parts[5],
        hemispherical_emissivity=1-a[1,4]/math.pi,upwelling_flux_w_m2_um=values[1],ground_downwelling_irradiance_w_m2_um=values[2],ground_upwelling_flux_w_m2_um=values[3])

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--geometry',type=Path,required=True);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
    folder=Path(__file__).resolve().parent;cases=json.loads((folder/'cases.json').read_text())
    raw=args.geometry.read_bytes();geo=json.loads(zlib.decompress(raw) if args.geometry.suffix=='.zlib' else raw);output=[]
    for i,cse in enumerate(cases):
        result=point(cse['parameters'],geo['geometry'][i]);output.append({'name':cse['name'],'parameters':cse['parameters'],'result':result})
    base=cases[0]['parameters'];n=len(cases);curves={k:[] for k in ('principal_brightness_temperature_k','principal_effective_emissivity','hemisphere_brightness_temperature_k','hemisphere_effective_emissivity')}
    for j in range(819):
        p=geo['parameters'][n+j];o=point(p,geo['geometry'][n+j]);prefix='principal_' if j<171 else 'hemisphere_'
        for name in ('brightness_temperature_k','effective_emissivity'):curves[prefix+name].append(o[name])
    wavelengths=list(range(8000,14001,100));a=coefficients(base,geo['geometry'][0]);spectra=[];temperatures=[]
    for w in wavelengths:
        p=base|{'wavelength_nm':w};o=point(p,geo['geometry'][0]);spectra.append(o['radiance_w_m2_sr_um']);temperatures.append(o['brightness_temperature_k'])
    document={'oracle':'Independent scipy.linalg.expm solution of Verhoef et al. (2007) transfer ODE; original MIT R geometry; original 20-segment hotspot convention','scipy_version':scipy.__version__,'cases':output,'default_arrays':curves|{'wavelengths_nm':wavelengths,'spectral_radiance_w_m2_sr_um':spectra,'spectral_brightness_temperature_k':temperatures}}
    args.out.write_text(json.dumps(document,indent=2)+'\n',encoding='utf-8');print('Computed',len(cases),'independent cases,',819,'angular directions and',len(wavelengths),'wavelengths')

if __name__=='__main__':main()
