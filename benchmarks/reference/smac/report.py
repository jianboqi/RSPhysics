"""Report installed Python/Rust agreement with unmodified original SMAC."""
import json
from pathlib import Path
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]
def main():
    suite=json.loads((Path(__file__).parent/'expected.json').read_text())
    max_abs=max_normalized=0.;samples=0
    def check(a,b):
        nonlocal max_abs,max_normalized,samples
        error=abs(a-b);allowed=suite['tolerance']['absolute']+suite['tolerance']['relative']*abs(b)
        assert error<=allowed
        max_abs=max(max_abs,error);max_normalized=max(max_normalized,error/allowed);samples+=1
    for c in suite['cases']:
        out=rsp.atmosphere.smac(**c['input'])['result']
        for key,value in c['expected'].items():check(out[key],value)
    for c in suite['curves']:
        out=rsp.atmosphere.smac(**c['input'])['result']
        for a,b in zip(out['sweep_reflectance'],c['sweep_reflectance']):check(float(a),b)
    report=dict(backend='installed Python wheel / shared Rust',cases=len(suite['cases']),samples=samples,max_absolute=max_abs,max_normalized=max_normalized,coefficient_sets=len(rsp.atmosphere.smac_coefficients()),pack_sha256=suite['coefficient_pack_sha256'])
    (Path(__file__).parent/'validation-report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
