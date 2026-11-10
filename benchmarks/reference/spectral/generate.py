"""Independent exact rational integration oracle; no RSPhysics, NumPy or Rust.

Expand linear interpolants as polynomials in wavelength, integrate products
symbolically with fractions.Fraction. This differs from production's quadrature
and interpolation-weight plans. Only synthetic response curves are distributed.
"""
from fractions import Fraction as F
import json
from pathlib import Path
import random

def line(x,y,q):
    for i in range(len(x)-1):
        if x[i]<=q<=x[i+1]:
            slope=(y[i+1]-y[i])/(x[i+1]-x[i]);return [y[i]-slope*x[i],slope]
    raise ValueError('outside oracle axis')

def integral(curves):
    lo=max(x[0] for x,y in curves);hi=min(x[-1] for x,y in curves)
    knots=sorted({lo,hi}|{v for x,y in curves for v in x if lo<v<hi});total=F(0)
    for a,b in zip(knots,knots[1:]):
        coefficients=[F(1)]
        for x,y in curves:
            c=line(x,y,(a+b)/2);result=[F(0)]*(len(coefficients)+1)
            for i,v in enumerate(coefficients):
                for j,w in enumerate(c):result[i+j]+=v*w
            coefficients=result
        total+=sum(v*(b**(i+1)-a**(i+1))/(i+1) for i,v in enumerate(coefficients))
    return total

def main():
    rng=random.Random(20261109);cases=[]
    for i in range(32):
        x=sorted({400,2500}|set(rng.sample(range(401,2500),8)))
        rx=sorted({450,2400}|set(rng.sample(range(451,2400),6)))
        ix=sorted({400,2500}|set(rng.sample(range(401,2500),5)))
        y=[F(rng.randrange(-100,500),100) for _ in x]
        r=[F(rng.randrange(1,100),100) for _ in rx]
        e=[F(rng.randrange(1,1000),100) for _ in ix]
        signal=(list(map(F,x)),y);response=(list(map(F,rx)),r);illumination=(list(map(F,ix)),e)
        cases.append(dict(name=f'rational-{i:02d}',wavelengths_nm=x,values=list(map(float,y)),response_wavelengths_nm=rx,response=list(map(float,r)),irradiance_wavelengths_nm=ix,irradiance=list(map(float,e)),expected_response_mean=float(integral([signal,response])/integral([response])),expected_irradiance_mean=float(integral([signal,response,illumination])/integral([response,illumination]))))
    path=Path(__file__).parent/'expected.json';path.write_text(json.dumps(dict(reference='Exact rational polynomial integration, independent of production quadrature',seed=20261109,cases=cases),indent=2)+'\n')
    print('32 independent exact rational spectral cases')
if __name__=='__main__':main()
