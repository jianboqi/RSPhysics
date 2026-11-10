"""80-digit RPV equation reference, independent of RSPhysics and JRC REAL."""
from pathlib import Path
import json,random
import mpmath as mp
mp.mp.dps=80
def evaluate(p,s=None,v=None,a=None):
    d=lambda x:mp.mpf(str(x))
    s=d(p['solar_zenith_deg'] if s is None else s)*mp.pi/180
    v=d(p['view_zenith_deg'] if v is None else v)*mp.pi/180
    a=d(p['relative_azimuth_deg'] if a is None else a)*mp.pi/180
    rho=d(p['rho0']);k=d(p['minnaert_k']);g=d(p['phase_asymmetry'])
    rhoc=rho if p['hotspot_mode']=='original_three_parameter' else d(p['hotspot_rho'])
    cg=mp.cos(s)*mp.cos(v)+mp.sin(s)*mp.sin(v)*mp.cos(a)
    distance=mp.sqrt((mp.tan(s)-mp.tan(v))**2+4*mp.tan(s)*mp.tan(v)*mp.sin(a/2)**2)
    M=(mp.cos(s)*mp.cos(v))**(k-1)*(mp.cos(s)+mp.cos(v))**(k-1)
    F=(1-g*g)/(1+g*g+2*g*cg)**mp.mpf('1.5');H=1+(1-rhoc)/(1+distance)
    brf=rho*M*F*H
    return {key:float(value) for key,value in dict(brf=brf,brdf=brf/mp.pi,minnaert_factor=M,phase_function=F,hotspot_factor=H).items()}
def generate():
    default=dict(rho0=.2,minnaert_k=.7,phase_asymmetry=-.2,hotspot_rho=.2,hotspot_mode='original_three_parameter',solar_zenith_deg=30.,view_zenith_deg=20.,relative_azimuth_deg=0.)
    rng=random.Random(283482);cases=[]
    variants=[{},dict(rho0=0.),dict(minnaert_k=1.,phase_asymmetry=0.,hotspot_mode='independent_four_parameter',hotspot_rho=1.),dict(solar_zenith_deg=50.,view_zenith_deg=50.),dict(solar_zenith_deg=0.,view_zenith_deg=0.),dict(phase_asymmetry=-.95),dict(phase_asymmetry=.95,solar_zenith_deg=85.,view_zenith_deg=85.,relative_azimuth_deg=180.)]
    for _ in range(30):variants.append(dict(rho0=rng.uniform(.01,.9),minnaert_k=rng.uniform(.05,1.9),phase_asymmetry=rng.uniform(-.8,.8),hotspot_mode='independent_four_parameter',hotspot_rho=rng.random(),solar_zenith_deg=rng.uniform(0,85),view_zenith_deg=rng.uniform(0,85),relative_azimuth_deg=rng.uniform(0,180)))
    for i,delta in enumerate(variants):
        p=default|delta;case=dict(name=f'precision-{i}',parameters=p,expected=evaluate(p))
        if i==0:
            case['principal_brf']=[evaluate(p,v=abs(v),a=180 if v<0 else 0)['brf'] for v in range(-85,86)]
            case['hemisphere_brf']=[evaluate(p,v=v,a=a)['brf'] for v in range(0,86,5) for a in range(0,360,10)]
        cases.append(case)
    return {'cases':cases}
if __name__=='__main__':Path(__file__).with_name('precision.json').write_text(json.dumps(generate(),indent=2)+'\n',encoding='utf8',newline='\n')
