"""Verify complete L-MEB branches or corrected GORT with separately supplied MATLAB.

Original files are checked against the source manifest and never modified.
Explicit corrections are applied only to disposable copies in .reference-cache.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[2]


def patch(model,work):
    if model=='lmeb':
        file=work/'tauw1i.m';raw=file.read_bytes();needle=b'yy= reflect'
        assert raw.count(needle)==1
        raw=raw.replace(needle,b'if ifwat==1; tg=teff; end % defined water/ice emission temperature\r\n'+needle,1)
        raw=raw.replace(b'ymod=[tbv, tbh];',b'autres=[real(eps2),imag(eps2),tg];\r\nymod=[tbv, tbh];',1)
        file.write_bytes(raw)
    else:
        file=work/'compute_gap_fractions.m';s=file.read_bytes().decode('latin1')
        s=s.replace('temp1=Bgap(i+1)*sin(2.0*theta);','temp1=Bgap(i+1)*sin(2.0*i*PI/180);')
        s=s.replace('temp1=Wgap(i+1)*sin(2.0*theta);','temp1=Wgap(i+1)*sin(2.0*i*PI/180);')
        needle='Wgap(i+1)=exp(-tau_p*s_r);';assert needle in s
        s=s.replace(needle,'temp=lambda_v*PI*R*R*s_p; s_r=temp/(-expm1(-temp))*s_1;\n        '+needle)
        file.write_bytes(s.encode('latin1'))


def compare(got,want):
    if isinstance(want,(int,float)):
        error=abs(got-want);assert error<=2e-9+2e-9*abs(want),(got,want)
        return error
    if isinstance(want,list):
        assert len(got)==len(want)
        return max((compare(a,b) for a,b in zip(got,want,strict=True)),default=0.)
    if isinstance(want,dict):
        return max((compare(got[k],v) for k,v in want.items() if k!='name'),default=0.)
    return 0.


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model',choices=['lmeb','gort-gpp'],required=True)
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--matlab',default='matlab')
    args=parser.parse_args();source=args.source.resolve();fixture=ROOT/'benchmarks/reference'/args.model
    manifest=json.loads((fixture/'upstream-provenance.json').read_text(encoding='utf-8'))
    cache=ROOT/'.reference-cache';cache.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=args.model+'-source-',dir=cache) as directory:
        work=Path(directory)
        for record in manifest['files']:
            relative=record['path'];original=(source/relative).resolve();assert original.is_relative_to(source)
            raw=original.read_bytes();assert hashlib.sha256(raw).hexdigest()==record['sha256'],relative
            if relative.endswith('.m'):
                target=work/relative;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
        patch(args.model,work)
        shutil.copyfile(fixture/'full-inputs.json',work/'inputs.json')
        shutil.copyfile(fixture/'generate_full.m',work/'generate_full.m')
        prefs=work/'matlab-prefs';prefs.mkdir();env=os.environ|{'MATLAB_PREFDIR':str(prefs)}
        result=subprocess.run([args.matlab,'-batch',"cd('"+work.as_posix().replace("'","''")+"');generate_full"],env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        if result.returncode:raise RuntimeError(result.stdout+result.stderr)
        got=json.loads((work/'expected.json').read_text(encoding='utf-8'))
        want=json.loads((fixture/'full-expected.json').read_text(encoding='utf-8'))
        if isinstance(want,dict):got=got['cases'];want=want['cases']
        error=compare(got,want)
        print(f'{args.model}: {len(want)} extended MATLAB cases, maximum error {error:.3g}')


if __name__=='__main__':main()
