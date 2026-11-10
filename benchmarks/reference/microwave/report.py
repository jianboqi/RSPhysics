"""Compare installed Python wheel with both independent microwave oracles."""
import json
from pathlib import Path
import numpy as np
import rsphysics as rsp
ROOT=Path(__file__).resolve().parents[3]
def main():
 for slug in ['tau-omega','water-cloud']:
  folder=ROOT/'benchmarks/reference'/slug;suite=json.loads((folder/'expected.json').read_text())
  count=0;maximum=0.;normalized=0.;sense_max=0.
  for case in suite['cases']:
   actual=rsp.run('microwave.'+slug,case['parameters'])['result']
   for key,expected in case['expected'].items():
    a=np.asarray(actual[key]);b=np.asarray(expected);error=np.abs(a-b);allowed=2e-12+2e-12*np.abs(b)
    assert a.shape==b.shape and np.all(np.isfinite(a)) and np.all(error<=allowed),(slug,case['name'],key)
    count+=a.size;maximum=max(maximum,float(np.max(error)));normalized=max(normalized,float(np.max(error/allowed)))
   for key,expected in case.get('sense_canopy',{}).items():
    error=abs(actual[key]-expected);assert error<=2e-12+2e-12*abs(expected);sense_max=max(sense_max,error)
  report=dict(model='microwave.'+slug,backend='python',suite=suite['suite'],cases=len(suite['cases']),compared_values=count,max_absolute=maximum,max_normalized_error=normalized,sense_canopy_max_absolute=sense_max if slug=='water-cloud' else None,reference_kind=suite['reference_kind'],scientific_validation='Implementation consistency only; not an observational accuracy assessment or a sensor calibration.')
  (folder/'validation-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
