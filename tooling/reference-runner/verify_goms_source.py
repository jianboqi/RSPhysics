"""Compare all GOMS bands, terrain cases and angular grids to separately supplied C.

Run in a C compiler environment, e.g. a Visual Studio x64 developer prompt:
python tooling/reference-runner/verify_goms_source.py --source PATH/Goms.c --cc cl
Original files are read only. Temporary sources/binaries stay in .reference-cache.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import subprocess
import tempfile
import zlib

ROOT = Path(__file__).resolve().parents[2]
SOURCE_SHA = 'd40e48bf3cd9cf0063ce3e567aab71b62ebc25b234b1dae045dd69406ee337af'
SCALARS = ['normalized_crown_density', 'crown_axis_ratio', 'crown_height_ratio',
           'relative_height_variation', 'solar_zenith_deg', 'solar_azimuth_deg',
           'view_zenith_deg', 'view_azimuth_deg', 'slope_deg', 'slope_aspect_deg']
OPTICS = ['sunlit_ground_reflectance', 'sunlit_crown_reflectance', 'shaded_reflectance']
HARNESS = r'''
#include <stdio.h>
#include <math.h>
#include SOURCE_FILE
static int visible(void){
 int i;for(i=0;i<2;i++){
  tip=atan(brratio*tan(InputDirection[2*i]*dtor));phip=InputDirection[2*i+1]*dtor;
  if(i==1){while(phip>PI)phip-=2*PI;while(phip < -PI)phip+=2*PI;}
  slope();if(!isfinite(tip)||!isfinite(phip)||cos(tip)<=1e-10)return 0;
 }return 1;
}
int main(void){
 double den,br,hb,spread,sun,sunaz,view,viewaz,tilt,aspect;int count,i,valid;
 while(scanf("%lf %lf %lf %lf %lf %lf %lf %lf %lf %lf %d",&den,&br,&hb,&spread,&sun,&sunaz,&view,&viewaz,&tilt,&aspect,&count)==11){
  if(count<1||count>10)return 2;
  lambda=den;brratio=br;hbratio=hb;htdif=spread;NumberOfBand=count;tsp=tilt*dtor;phsp=aspect*dtor;
  InputDirection[0]=sun;InputDirection[1]=sunaz;InputDirection[2]=view;InputDirection[3]=viewaz;
  for(i=0;i<count;i++)if(scanf("%lf",&RG[i])!=1)return 3;
  for(i=0;i<count;i++)if(scanf("%lf",&RC[i])!=1)return 3;
  for(i=0;i<count;i++)if(scanf("%lf",&RZ[i])!=1)return 3;
  valid=visible();if(valid)Goms();
  if(valid){for(i=0;i<count;i++)if(!isfinite(OutputBRDF[i]))valid=0;for(i=0;i<3;i++)if(!isfinite(reference_weights[i]))valid=0;}
  printf("%d",valid);
  for(i=0;i<count;i++)printf(" %.17g",valid?OutputBRDF[i]:0.);
  for(i=0;i<3;i++)printf(" %.17g",valid?reference_weights[i]:0.);
  printf("\n");
 }return 0;
}
'''


def instrument(source):
    source = source.replace('void Goms(void)', 'double reference_weights[3];\nvoid Goms(void)')
    needle = 'kz = 1.0 - kg - kc;'
    assert needle in source
    return source.replace(needle, needle + '\nreference_weights[0]=kg;reference_weights[1]=kc;reference_weights[2]=kz;')


def corrected(source):
    source = source.replace('#define PI 3.1415926', '#define PI 3.14159265358979323846')
    source = source.replace('over = (temp - sin(temp)*cos(temp)) * (secti+sectv)/PI;',
                            'over = (temp - sin(temp)*cos(temp)) * (secti+sectv);')
    source = source.replace('if(tsp < 0.01) return;', 'if(tsp == 0) return;')
    start = source.index('        phip = asin(y/sqrt(r));')
    end = source.index('        return;', start)
    source = source[:start] + '        phip = atan2(y,x);\n' + source[end:]
    source = source.replace('        thetav = tip;',
                            '        thetav = tip;\n        tanti=tan(thetai);tantv=tan(thetav);')
    return source.replace('        if(vaz < 0.) vaz = -vaz;',
                          '        vaz=fmod(fabs(vaz),2*PI);if(vaz>PI)vaz=2*PI-vaz;')


def run(executable, rows, cwd):
    lines = []
    for p in rows:
        values = [p[k] for k in SCALARS] + [len(p[OPTICS[0]])]
        values += [x for k in OPTICS for x in p[k]]
        lines.append(' '.join(format(x, '.17g') for x in values))
    output = subprocess.check_output([str(executable)], input=('\n'.join(lines)+'\n').encode(), cwd=cwd)
    result = [list(map(float, line.split())) for line in output.decode().splitlines()]
    assert len(result) == len(rows)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--cc', default='cl' if os.name == 'nt' else 'cc')
    args = parser.parse_args()
    raw = args.source.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == SOURCE_SHA, 'unexpected Goms.c revision'
    source = raw.decode('gb18030').replace('\r\n', '\n')
    fixture = ROOT/'benchmarks/reference/goms'
    inputs = json.loads((fixture/'inputs.json').read_text(encoding='utf-8'))
    expected = json.loads(zlib.decompress((fixture/'expected.json.zlib').read_bytes()))
    cache = ROOT/'.reference-cache';cache.mkdir(exist_ok=True)
    maximum = 0.;samples = 0
    with tempfile.TemporaryDirectory(prefix='goms-source-', dir=cache) as directory:
        work = Path(directory)
        for name, text in [('original', source), ('consistent', corrected(source))]:
            # One translation unit avoids command-shell macro quoting differences.
            (work/(name+'.c')).write_text(instrument(text)+'\n'+HARNESS.replace('#include SOURCE_FILE',''), encoding='utf-8')
            executable = work/(name+'.exe' if os.name == 'nt' else name)
            if Path(args.cc).stem.lower() == 'cl':
                command = [args.cc, '/nologo', '/O2', '/fp:precise', name+'.c', '/Fe:'+str(executable)]
            else:
                command = [args.cc, '-O2', name+'.c', '-lm', '-o', str(executable)]
            subprocess.run(command, cwd=work, check=True, capture_output=True)
        for p, want in zip(inputs, expected, strict=True):
            mode = 'original' if p['geometry_convention']=='legacy_c' else 'consistent'
            executable = work/(mode+'.exe' if os.name == 'nt' else mode)
            rows = [p]
            rows += [p|dict(view_zenith_deg=abs(v),view_azimuth_deg=(p['solar_azimuth_deg']+(180 if v<0 else 0))%360) for v in want['principal_view_zenith_deg']]
            rows += [p|dict(view_zenith_deg=v,view_azimuth_deg=a) for v,a in zip(want['hemisphere_view_zenith_deg'],want['hemisphere_azimuth_deg'],strict=True)]
            rows += [p|dict(view_zenith_deg=p['solar_zenith_deg'],view_azimuth_deg=p['solar_azimuth_deg'])]
            got = run(executable, rows, work);nb = len(p[OPTICS[0]])
            oracle = [[1]+want['brf']+[want[k] for k in ['sunlit_ground_weight','sunlit_crown_weight','shaded_weight']]]
            for prefix in ['principal','hemisphere']:
                for j, valid in enumerate(want[prefix+'_valid']):
                    row=[valid]+[want[prefix+'_brf'][b][j] for b in range(nb)]
                    if prefix=='principal':row += [None,None,want['principal_shaded_weight'][j]]
                    oracle.append(row)
            oracle.append([1]+want['hotspot_brf']+[None,None,want['hotspot_shaded_weight']])
            for a,b in zip(got,oracle,strict=True):
                for x,y in zip(a,b):
                    if y is None:continue
                    error=abs(x-y);maximum=max(maximum,error);samples+=1
                    assert error<=2e-10+2e-9*abs(y),(x,y,error)
    print(f'GOMS original/corrected C: {len(inputs)} cases, {samples} values, maximum error {maximum:.3g}')


if __name__ == '__main__':
    main()
