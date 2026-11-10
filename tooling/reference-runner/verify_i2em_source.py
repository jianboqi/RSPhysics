"""Recompute pinned co-pol fixtures with a separately obtained original MATLAB file."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--matlab',default='matlab')
    parser.add_argument('--extended',action='store_true',help='verify x-power/x-exponential with 1e-12 Hankel quadrature tolerance')
    args=parser.parse_args()
    source=args.source.resolve();raw=source.read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='348485b133dcc309d3d4a64727420c96775d686aada6f4b29a35877759d6d046','unexpected source revision'
    text=raw.decode('utf-8').replace('\r\n','\n')
    text=text[text.index('function [sigma_0_vv sigma_0_hh] = I2EM_Bistat_model'):text.index('function sigvh = IEMX_model')]
    if args.extended:text=text.replace(', 0,9)',', 0,9,1e-12)')
    fixtures=ROOT/'benchmarks/reference/i2em'
    with tempfile.TemporaryDirectory(prefix='i2em-reference-',dir=source.parent) as directory:
        work=Path(directory)
        (work/'I2EM_Bistat_model.m').write_text(text,encoding='utf-8')
        shutil.copyfile(fixtures/('full-inputs.json' if args.extended else 'inputs.json'),work/'inputs.json')
        (work/'verify_reference.m').write_text("""p=jsondecode(fileread('inputs.json'));r=zeros(numel(p),2);
for i=1:numel(p)
 c=p(i);names={'exponential','gaussian','x_power','x_exponential'};sp=find(strcmp(c.correlation_function,names));xx=1.5;
 if isfield(c,'correlation_exponent');xx=c.correlation_exponent;end
 [vv,hh]=I2EM_Bistat_model(c.frequency_ghz,c.rms_height_m,c.correlation_length_m,c.observation_zenith_deg,c.observation_zenith_deg,180,complex(c.permittivity_real,-c.permittivity_loss),sp,xx);
 r(i,:)=[vv,hh];
end
f=fopen('actual.json','w');fwrite(f,jsonencode(r),'char');fclose(f);
""",encoding='utf-8')
        subprocess.run([args.matlab,'-batch',"cd('"+work.as_posix().replace("'","''")+"');verify_reference"],check=True)
        got=json.loads((work/'actual.json').read_text(encoding='utf-8'))
        want=json.loads((fixtures/('full-expected.json' if args.extended else 'expected.json')).read_text(encoding='utf-8'))
        assert len(got)==len(want)
        error=max(abs(a-b) for aa,bb in zip(got,want) for a,b in zip(aa,bb))
        assert error<=2e-9,error
        print(f'OK original I2EM MATLAB: {len(got)} cases, max error {error:.3g} dB')

if __name__=='__main__':main()
