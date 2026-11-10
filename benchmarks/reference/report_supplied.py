"""Compare the installed Python package with retained original MATLAB fixtures."""
from pathlib import Path
import json
import numpy as np
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[2]
for domain,slug in [('microwave','lmeb'),('ecosystem','gort-gpp')]:
    ref=ROOT/'benchmarks/reference'/slug
    inputs=json.loads((ref/'inputs.json').read_text())
    original=json.loads((ref/'expected.json').read_text())
    samples=0;max_abs=0.;max_scaled=0.
    for case,want in zip(inputs,original['cases'],strict=True):
        actual=rsp.run(f'{domain}.{slug}',case['parameters'])['result']
        for field,value in want.items():
            if field=='name':continue
            a=np.asarray(actual[field]);b=np.asarray(value)
            if a.shape!=b.shape:raise ValueError(f'shape mismatch: {field}')
            error=np.abs(a-b);scaled=error/(2e-9+2e-9*np.abs(b))
            if not np.all(np.isfinite(a)) or np.any(scaled>1):raise ValueError(f'reference disagreement: {case["name"]} {field}')
            samples+=a.size;max_abs=max(max_abs,float(error.max()));max_scaled=max(max_scaled,float(scaled.max()))
    report=dict(model_id=f'{domain}.{slug}',reference_runtime=original['runtime'],backend='installed Python Wheel',cases=len(inputs),samples=samples,max_absolute_error=max_abs,max_tolerance_fraction=max_scaled,tolerance=dict(absolute=2e-9,relative=2e-9),scope='Implementation agreement with supplied original, not validation against observations')
    (ref/'validation-report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
