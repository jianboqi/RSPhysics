"""Recompute complete FLUSPECT-CX references with separately obtained author files."""
from pathlib import Path
import argparse,hashlib,json,shutil,subprocess,tempfile,zlib

ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--source',type=Path,required=True,help='Directory containing fluspect_B_CX.m and both pinned D/PRO MAT tables')
    parser.add_argument('--matlab',default='matlab')
    args=parser.parse_args();source=args.source.resolve();fixtures=ROOT/'benchmarks/reference/fluspect-cx'
    provenance=json.loads((fixtures/'provenance.json').read_text(encoding='utf-8'))
    recorded=provenance['full_modes']['source']
    for name,key in [('fluspect_B_CX.m','src/RTMs/fluspect_B_CX.m'),('Optipar2017_ProspectD.mat','input/fluspect_parameters/Optipar2017_ProspectD.mat'),('Optipar2021_ProspectPRO_CX.mat','input/fluspect_parameters/Optipar2021_ProspectPRO_CX.mat')]:
        assert hashlib.sha256((source/name).read_bytes()).hexdigest()==recorded[key]['sha256'],name
    inputs=json.loads((fixtures/'inputs.json').read_text(encoding='utf-8'))+json.loads((fixtures/'full-inputs.json').read_text(encoding='utf-8'))
    expected=json.loads(zlib.decompress((fixtures/'expected.json.zlib').read_bytes()))+json.loads(zlib.decompress((fixtures/'full-expected.json.zlib').read_bytes()))
    with tempfile.TemporaryDirectory(prefix='fluspect-reference-',dir=source.parent) as directory:
        work=Path(directory);(work/'inputs.json').write_text(json.dumps(inputs),encoding='utf-8')
        text=(source/'fluspect_B_CX.m').read_text(encoding='utf-8')
        full=text.replace('function leafopt = fluspect_B_CX(','function leafopt = fluspect_reference_full(').replace('int         = 5;','int         = 1;').replace('(640:4:850)','(640:1:850)')
        assert 'int         = 1;' in full
        (work/'fluspect_reference_full.m').write_text(full,encoding='utf-8')
        for name in ['Optipar2017_ProspectD.mat','Optipar2021_ProspectPRO_CX.mat']:shutil.copyfile(source/name,work/name)
        script="addpath('"+source.as_posix().replace("'","''")+"');\n"
        script+='''spectral.wlP=400:2400;p=jsondecode(fileread('inputs.json'));result=cell(numel(p),1);
for i=1:numel(p)
 c=p(i);if strcmp(c.optical_coefficients,'prospect_d'),load('Optipar2017_ProspectD.mat','optipar');else,load('Optipar2021_ProspectPRO_CX.mat','optipar');end
 % Correct the negative 641 nm emission calibration residue; raw evidence is retained.
 optipar.phi=max(optipar.phi,0);
 leafbio=struct('N',c.n,'Cab',c.chlorophyll,'Cca',c.carotenoids,'Cant',c.anthocyanins,'Cs',c.brown_pigments,'Cw',c.water,'Cdm',c.dry_matter,'V2Z',c.xanthophyll_state,'fqe',c.fluorescence_efficiency,'Cp',c.protein,'Cbc',c.carbon_based_constituents);
 if strcmp(c.spectral_resolution,'full_1nm'),o=fluspect_reference_full(spectral,leafbio,optipar);nf=211;ne=351;else,o=fluspect_B_CX(spectral,leafbio,optipar);nf=53;ne=71;end
 if ~isfield(o,'Mb'),o.Mb=zeros(nf,ne);o.Mf=zeros(nf,ne);end
 result{i}=struct('reflectance',o.refl,'transmittance',o.tran,'chlorophyll_absorption_fraction',o.kChlrel,'carotenoid_absorption_fraction',o.kCarrel,'fluorescence_backward',o.Mb,'fluorescence_forward',o.Mf);
end
f=fopen('actual.json','w');fwrite(f,jsonencode(result),'char');fclose(f);
'''
        (work/'verify_reference.m').write_text(script,encoding='utf-8')
        subprocess.run([args.matlab,'-batch',"cd('"+work.as_posix().replace("'","''")+"');verify_reference"],check=True)
        got=json.loads((work/'actual.json').read_text(encoding='utf-8'))
        def compare(a,b,absolute):
            if isinstance(b,dict):return max(compare(a[k],v,absolute) for k,v in b.items())
            if isinstance(b,list):
                assert len(a)==len(b)
                return max(compare(x,y,absolute) for x,y in zip(a,b))
            assert abs(a-b)<=absolute+2e-9*abs(b),(a,b)
            return abs(a-b)
        error=max(compare(a,b,2e-8 if i==10 else 2e-10) for i,(a,b) in enumerate(zip(got,expected)))
        assert len(got)==len(expected)
        print(f'OK original FLUSPECT-CX: {len(got)} light/full/PRO cases, max absolute error {error:.3g}')

if __name__=='__main__':main()
