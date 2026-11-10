"""Offline integrity check for the independent FvCB equations and fixtures."""
import hashlib
import json
import math
from pathlib import Path

root=Path(__file__).resolve().parent
manifest=json.loads((root/'source-manifest.json').read_text(encoding='utf8'))
for name,wanted in manifest['sha256'].items():
    assert hashlib.sha256((root/name).read_bytes()).hexdigest()==wanted,name
record=json.loads((root/'reference.json').read_text(encoding='utf8'))
assert len(record['cases'])==92
assert {case['model_id'] for case in record['cases']}=={'ecosystem.fvcb','ecosystem.fvcb-big-leaf'}
def finite(value):
    if isinstance(value,float):assert math.isfinite(value)
    elif isinstance(value,dict):
        for item in value.values():finite(item)
    elif isinstance(value,list):
        for item in value:finite(item)
finite(record)
print('OK FvCB: 92 independent leaf/canopy cases; equation and fixture hashes verified')
