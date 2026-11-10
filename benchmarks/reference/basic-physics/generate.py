"""Independent 80-digit direct textbook equations; no RSPhysics imports.

Production Fresnel uses normalized normal-wavevectors/admittances; this oracle
uses complex Snell angles and standard E-field r/t expressions. Production
Planck uses logarithmic branches; this oracle evaluates direct exponentials.
"""
import hashlib
import json
import math
from pathlib import Path
import random
import mpmath as mp
mp.mp.dps=80
H=mp.mpf('6.62607015e-34');C=mp.mpf('299792458');K=mp.mpf('1.380649e-23')
def number(v):return mp.mpf(str(v))
def complex_record(z):return [float(mp.re(z)),float(mp.im(z))]
def fresnel(p):
    angle=number(p['angle_deg'])*mp.pi/180;n1=number(p['n_incident']);n2=mp.mpc(number(p['n_real']),number(p['k']))
    cos1=mp.cos(angle);cos2=mp.sqrt(1-(n1*mp.sin(angle)/n2)**2)
    q=n2*cos2
    if mp.im(q)<0 or mp.im(q)==0 and mp.re(q)<0:cos2=-cos2
    rs=(n1*cos1-n2*cos2)/(n1*cos1+n2*cos2)
    rp=(n2*cos1-n1*cos2)/(n2*cos1+n1*cos2)
    ts=2*n1*cos1/(n1*cos1+n2*cos2)
    tp=2*n1*cos1/(n2*cos1+n1*cos2)
    R_s=abs(rs)**2;R_p=abs(rp)**2
    T_s=abs(ts)**2*mp.re(n2*cos2)/(n1*cos1)
    T_p=abs(tp)**2*mp.re(n2*mp.conj(cos2))/(n1*cos1)
    return dict(amplitude_r_s=complex_record(rs),amplitude_r_p=complex_record(rp),reflectance_s=float(R_s),reflectance_p=float(R_p),reflectance_unpolarized=float((R_s+R_p)/2),interface_transmittance_s=float(T_s),interface_transmittance_p=float(T_p),interface_transmittance_unpolarized=float((T_s+T_p)/2))
def main():
    assert mp.__version__=='1.3.0'
    optical=[]
    def add(name,theta=40.,n1=1.,n=1.5,k=0.):
        p=dict(angle_deg=theta,n_incident=n1,n_real=n,k=k);optical.append(dict(name=name,parameters=p,expected=fresnel(p)))
    add('normal-glass',0);add('matched-grazing',89.999999,n=1);add('brewster',math.degrees(math.atan(1.5)))
    add('total-internal-reflection',60,n1=1.5,n=1)
    critical=math.degrees(math.asin(1/1.5))
    add('below-critical',critical-1e-4,n1=1.5,n=1);add('above-critical',critical+1e-4,n1=1.5,n=1)
    add('weak-absorption-tir',60,n1=1.5,n=1,k=1e-12)
    add('absorbing-dielectric',k=.3);add('metal',70,n=.2,k=3)
    add('evanescent-negative-permittivity',50,n=0,k=2)
    add('small-index',0,n=1e-6);add('large-index',80,n=1e6)
    rng=random.Random(20261110)
    for i in range(20):add(f'seeded-{i:02d}',rng.uniform(0,89.5),rng.uniform(.8,2.),rng.uniform(.2,5.),rng.choice([0.,rng.uniform(.01,4.)]))
    thermal=[]
    pairs=[(10000.,0.),(1.,1.),(500.,5778.),(10000.,300.),(1e6,2.7),(1e9,300.),(1e100,1e200),(1e-100,1.)]
    for i in range(24):pairs.append((10**rng.uniform(2,9),10**rng.uniform(0,5)))
    for i,(w,t) in enumerate(pairs):
        lam=number(w)*mp.mpf('1e-9');temp=number(t)
        b=mp.mpf(0) if t==0 else 2*H*C**2/lam**5/mp.expm1(H*C/(lam*K*temp))*mp.mpf('1e-9')
        value=float(b);inv=mp.mpf(0) if value==0 else H*C/(lam*K)/mp.log1p(2*H*C**2/(lam**5)/(number(value)*mp.mpf('1e9')))
        thermal.append(dict(wavelength_nm=w,temperature_k=t,radiance_per_nm=value,inverse_temperature_k=float(inv)))
    frequencies=[(1.,0.),(1e6,1e6),(1e9,2.7),(1e12,300.),(1e16,300.),(1e-100,300.),(1e100,1.),(1e-50,1e200)]
    for i in range(24):frequencies.append((10**rng.uniform(3,16),10**rng.uniform(0,5)))
    freq=[]
    for nu,t in frequencies:
        n=number(nu);temp=number(t);b=mp.mpf(0) if t==0 else 2*H*n**3/C**2/mp.expm1(H*n/(K*temp));value=float(b)
        inv=mp.mpf(0) if value==0 else H*n/K/mp.log1p(2*H*n**3/(C**2*number(value)))
        freq.append(dict(frequency_hz=nu,temperature_k=t,radiance_per_hz=value,inverse_temperature_k=float(inv)))
    source=dict(reference='Independent direct published equations at 80 decimal digits, not an original vendor program',mpmath_version=mp.__version__,precision_digits=80,constants=dict(h=str(H),c=str(C),k=str(K)),sources=['https://www.nist.gov/pml/special-publication-330/sp-330-section-2','https://cie.co.at/eilvterm/17-24-005','https://arxiv.org/abs/1603.02720v5'],fresnel=optical,planck_wavelength=thermal,planck_frequency=freq)
    p=Path(__file__).parent/'expected.json';p.write_text(json.dumps(source,indent=2,allow_nan=False)+'\n')
    (p.parent/'fixture-sha256.txt').write_text(hashlib.sha256(p.read_bytes()).hexdigest()+'\n')
    print('32 Fresnel + 32 wavelength Planck + 32 frequency Planck references')
if __name__=='__main__':main()
