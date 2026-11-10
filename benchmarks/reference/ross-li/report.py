"""Audit the installed Python package against both recorded Ross-Li references."""
import hashlib,json,platform,sys
from pathlib import Path
import numpy as np
import rsphysics as rsp
FOLDER=Path(__file__).resolve().parent
suite=json.loads((FOLDER/'expected.json').read_text())
report=dict(suite=suite['suite'],oracle_sha256=hashlib.sha256((FOLDER/'expected.json').read_bytes()).hexdigest(),original_commit=suite['commit'],original_precision=suite['original_precision'],tolerance=suite['tolerance'],original_tolerance=suite['original_tolerance'],system=platform.platform(),python=sys.version.split()[0],numpy=np.__version__,cases=len(suite['cases']),samples=0,original_samples=0,max_absolute_error=0.,max_normalized_error=0.,max_original_absolute_error=0.)
for case in suite['cases']:
    result=rsp.surface.ross_li(**case['parameters'])['result']
    for key,expected in list(case['point'].items())+[(k,case[k]) for k in ('principal_brf','hemisphere_brf')]:
        actual=np.asarray(result[key]);expected=np.asarray(expected);error=abs(actual-expected)
        allowed=suite['tolerance']['absolute']+suite['tolerance']['relative']*abs(expected)
        assert np.all(error<=allowed),(case['name'],key)
        report['samples']+=actual.size;report['max_absolute_error']=max(report['max_absolute_error'],float(error.max()));report['max_normalized_error']=max(report['max_normalized_error'],float((error/allowed).max()))
    for key,expected in list(case['original_point'].items())+[(k,case['original_'+k]) for k in ('principal_brf','hemisphere_brf')]:
        actual=np.asarray(result[key]);expected=np.asarray(expected);error=abs(actual-expected)
        allowed=suite['original_tolerance']['absolute']+suite['original_tolerance']['relative']*abs(expected)
        assert np.all(error<=allowed)
        report['original_samples']+=actual.size;report['max_original_absolute_error']=max(report['max_original_absolute_error'],float(error.max()))
report['status']='passed';(FOLDER/'validation-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
