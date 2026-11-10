"""Compile unchanged original C; independently evaluate JRC equations at 80 digits."""
from pathlib import Path
import ctypes,hashlib,json,math,os,platform,random,shutil,subprocess
import mpmath as mp
from prepare import HASHES
ROOT=Path(__file__).resolve().parents[3];FOLDER=Path(__file__).resolve().parent
CACHE=ROOT/'.reference-cache/ross-li';mp.mp.dps=80
def compile_oracle():
    if mp.__version__!='1.3.0':raise ValueError('use the recorded mpmath 1.3.0')
    for name,expected in HASHES.items():
        if hashlib.sha256((CACHE/'upstream'/name).read_bytes()).hexdigest()!=expected:raise ValueError('original source changed: '+name)
    build=CACHE/'build';build.mkdir(parents=True,exist_ok=True)
    for name in ['ambralsfor.c','ambralsfor.h']:shutil.copyfile(CACHE/'upstream'/name,build/name)
    shutil.copyfile(FOLDER/'oracle_driver.c',build/'driver.c')
    if os.name=='nt':
        vswhere=Path(os.environ.get('ProgramFiles(x86)','C:/Program Files (x86)'))/'Microsoft Visual Studio/Installer/vswhere.exe'
        installation=subprocess.check_output([str(vswhere),'-latest','-products','*','-property','installationPath'],text=True).strip()
        vcvars=Path(installation)/'VC/Auxiliary/Build/vcvars64.bat'
        (build/'build.cmd').write_text(f'@call "{vcvars}" >nul\n@if errorlevel 1 exit /b %errorlevel%\n@cl /nologo /LD /O2 /fp:precise driver.c /link /OUT:oracle.dll\n')
        subprocess.run(['cmd.exe','/d','/c','build.cmd'],cwd=build,check=True)
        dll=build/'oracle.dll'
    else:
        dll=build/'oracle.so';subprocess.run(['cc','-shared','-fPIC','-O2','driver.c','-lm','-o',str(dll)],cwd=build,check=True)
    library=ctypes.CDLL(str(dll));fn=library.oracle;fn.argtypes=[ctypes.c_double]*6;fn.restype=ctypes.c_double
    return library,fn,hashlib.sha256(dll.read_bytes()).hexdigest()
def precise(p):
    s,v,a=[mp.mpf(str(p[key]))*mp.pi/180 for key in ('solar_zenith_deg','view_zenith_deg','relative_azimuth_deg')]
    cosxi=mp.cos(s)*mp.cos(v)+mp.sin(s)*mp.sin(v)*mp.cos(a);cosxi=min(mp.mpf(1),max(mp.mpf(-1),cosxi));xi=mp.acos(cosxi)
    vol=((mp.pi/2-xi)*cosxi+mp.sin(xi))/(mp.cos(s)+mp.cos(v))-mp.pi/4
    # Published primed-angle equations, with the fixed b/r ratio equal to one.
    sp=mp.atan(mp.tan(s));vp=mp.atan(mp.tan(v));ts=mp.tan(sp);tv=mp.tan(vp)
    d=mp.sqrt(max(mp.mpf(0),ts**2+tv**2-2*ts*tv*mp.cos(a)))
    secs=1/mp.cos(sp)+1/mp.cos(vp);cost=min(mp.mpf(1),2*mp.sqrt(d**2+(ts*tv*mp.sin(a))**2)/secs);t=mp.acos(cost)
    overlap=(t-mp.sin(t)*mp.cos(t))*secs/mp.pi
    cp=mp.cos(sp)*mp.cos(vp)+mp.sin(sp)*mp.sin(vp)*mp.cos(a)
    geo=overlap-secs+(1+cp)/(2*mp.cos(sp)*mp.cos(vp))
    brf=mp.mpf(str(p['f_iso']))+mp.mpf(str(p['f_vol']))*vol+mp.mpf(str(p['f_geo']))*geo
    return dict(brf=float(brf),brdf=float(brf/mp.pi),k_iso=1.,k_vol=float(vol),k_geo=float(geo))
def main():
    library,original,dllhash=compile_oracle()
    provenance=json.loads((FOLDER/'upstream-provenance.json').read_text())
    base=dict(f_iso=.2,f_vol=.1,f_geo=.02,solar_zenith_deg=30.,view_zenith_deg=20.,relative_azimuth_deg=0.)
    inputs=[]
    def add(name,**changes):inputs.append(dict(name=name,parameters=base|changes))
    add('default');add('nadir',view_zenith_deg=0);add('overhead-sun',solar_zenith_deg=0);add('both-nadir',solar_zenith_deg=0,view_zenith_deg=0)
    add('hotspot',view_zenith_deg=30);add('forward',relative_azimuth_deg=180);add('orthogonal',relative_azimuth_deg=90)
    add('max-view',view_zenith_deg=85);add('max-sun',solar_zenith_deg=85);add('both85-backward',solar_zenith_deg=85,view_zenith_deg=85)
    add('both85-forward',solar_zenith_deg=85,view_zenith_deg=85,relative_azimuth_deg=180)
    add('near-hotspot',view_zenith_deg=30,relative_azimuth_deg=1e-6)
    add('isotropic',f_vol=0,f_geo=0);add('black',f_iso=0,f_vol=0,f_geo=0);add('nonphysical-weights',f_iso=0,f_vol=0,f_geo=1)
    add('brf-above-one',f_iso=1,f_vol=1,f_geo=0,solar_zenith_deg=85,view_zenith_deg=85)
    rng=random.Random(20261109)
    for i in range(16):add(f'sample-{i:02d}',f_iso=rng.uniform(.02,.8),f_vol=rng.uniform(0,.5),f_geo=rng.uniform(0,.1),solar_zenith_deg=rng.uniform(0,85),view_zenith_deg=rng.uniform(0,85),relative_azimuth_deg=rng.uniform(0,180))
    def cpoint(p):
        angles=[p[k] for k in ('solar_zenith_deg','view_zenith_deg','relative_azimuth_deg')]
        return dict(brf=original(*angles,p['f_iso'],p['f_vol'],p['f_geo']),k_vol=original(*angles,0,1,0),k_geo=original(*angles,0,0,1))
    suite=dict(suite='ross-li-ambrals-jrc-v1',commit=provenance['commit'],original_precision='original AMBRALS float inputs/intermediates/return; unchanged C scientific functions',formula_reference='https://rami-benchmark.jrc.ec.europa.eu/_www/definition.php',high_precision=dict(library='mpmath',version=mp.__version__,decimal_digits=80),tolerance=dict(absolute=2e-12,relative=2e-12),original_tolerance=dict(absolute=1e-4,relative=2e-5),original_executable_sha256=dllhash,reference_platform=platform.platform(),cases=[])
    max_original=0.;samples=0
    for case in inputs:
        p=case['parameters'];result=precise(p);legacy=cpoint(p)
        principal=[];hemi=[];original_principal=[];original_hemi=[]
        for signed in range(-85,86):
            q=p|dict(view_zenith_deg=abs(signed),relative_azimuth_deg=180 if signed<0 else 0)
            principal.append(precise(q)['brf']);original_principal.append(cpoint(q)['brf'])
        for z in range(0,86,5):
            for a in range(0,360,10):
                q=p|dict(view_zenith_deg=z,relative_azimuth_deg=min(a,360-a))
                hemi.append(precise(q)['brf']);original_hemi.append(cpoint(q)['brf'])
        for a,b in [(result[k],legacy[k]) for k in legacy]+list(zip(principal,original_principal))+list(zip(hemi,original_hemi)):
            error=abs(a-b);max_original=max(error,max_original);samples+=1
            assert error<=suite['original_tolerance']['absolute']+suite['original_tolerance']['relative']*abs(b),(case['name'],a,b)
        suite['cases'].append(case|dict(point=result,original_point=legacy,principal_brf=principal,hemisphere_brf=hemi,original_principal_brf=original_principal,original_hemisphere_brf=original_hemi))
    suite['original_vs_precise']=dict(samples=samples,max_absolute_error=max_original,status='passed')
    (FOLDER/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
    (FOLDER/'expected.json').write_text(json.dumps(suite,separators=(',',':'))+'\n')
    print(f'{len(inputs)} cases; {samples} original-C comparisons; max abs={max_original:.9e}')
if __name__=='__main__':main()
