"""Recompute FLUSPECT-CX references with the separately obtained original SCOPE files."""
from pathlib import Path
import argparse,hashlib,json,shutil,subprocess,tempfile,zlib

ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--source',type=Path,required=True,help='Directory containing fluspect_B_CX.m and Optipar2017_ProspectD.mat')
    parser.add_argument('--matlab',default='matlab')
    args=parser.parse_args();source=args.source.resolve();fixtures=ROOT/'benchmarks/reference/fluspect-cx'
    provenance=json.loads((fixtures/'provenance.json').read_text(encoding='utf-8'))
    for name,key in [('fluspect_B_CX.m','src/RTMs/fluspect_B_CX.m'),('Optipar2017_ProspectD.mat','input/fluspect_parameters/Optipar2017_ProspectD.mat')]:
        assert hashlib.sha256((source/name).read_bytes()).hexdigest()==provenance['source'][key]['sha256'],name
    with tempfile.TemporaryDirectory(prefix='fluspect-reference-',dir=source.parent) as directory:
        work=Path(directory);shutil.copyfile(fixtures/'inputs.json',work/'inputs.json')
        script="addpath('"+source.as_posix().replace("'","''")+"');load('"+(source/'Optipar2017_ProspectD.mat').as_posix().replace("'","''")+"','optipar');\n"
        script+='''spectral.wlP=400:2400;p=jsondecode(fileread('inputs.json'));result=cell(numel(p),1);
for i=1:numel(p)
 c=p(i);leafbio=struct('N',c.n,'Cab',c.chlorophyll,'Cca',c.carotenoids,'Cant',c.anthocyanins,'Cs',c.brown_pigments,'Cw',c.water,'Cdm',c.dry_matter,'V2Z',c.xanthophyll_state,'fqe',c.fluorescence_efficiency,'Cp',0,'Cbc',0);
 o=fluspect_B_CX(spectral,leafbio,optipar);
 if ~isfield(o,'Mb'),o.Mb=zeros(53,71);o.Mf=zeros(53,71);end
 result{i}=struct('reflectance',o.refl,'transmittance',o.tran,'chlorophyll_absorption_fraction',o.kChlrel,'carotenoid_absorption_fraction',o.kCarrel,'fluorescence_backward',o.Mb,'fluorescence_forward',o.Mf);
end
f=fopen('actual.json','w');fwrite(f,jsonencode(result),'char');fclose(f);
'''
        (work/'verify_reference.m').write_text(script,encoding='utf-8')
        subprocess.run([args.matlab,'-batch',"cd('"+work.as_posix().replace("'","''")+"');verify_reference"],check=True)
        got=json.loads((work/'actual.json').read_text(encoding='utf-8'));want=json.loads(zlib.decompress((fixtures/'expected.json.zlib').read_bytes()))
        def compare(a,b):
            if isinstance(b,dict):return max(compare(a[k],v) for k,v in b.items())
            if isinstance(b,list):
                assert len(a)==len(b)
                return max(compare(x,y) for x,y in zip(a,b))
            assert abs(a-b)<=2e-10+2e-9*abs(b),(a,b)
            return abs(a-b)
        error=compare(got,want)
        print(f'OK original FLUSPECT-CX: {len(got)} cases, max absolute error {error:.3g}')

if __name__=='__main__':main()
