"""Verify the pinned I2EM MATLAB numerical fixtures without a MATLAB runtime."""
from pathlib import Path
import hashlib
import json
import math

ROOT = Path(__file__).resolve().parent

def verify():
    hashes = json.loads((ROOT/'manifest.json').read_text(encoding='utf-8'))
    assert set(hashes) == {'inputs.json','expected.json','provenance.json'}
    for name, want in hashes.items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest() == want, name
    inputs=json.loads((ROOT/'inputs.json').read_text(encoding='utf-8'))
    expected=json.loads((ROOT/'expected.json').read_text(encoding='utf-8'))
    assert len(inputs)==len(expected)==20
    assert {p['correlation_function'] for p in inputs}=={'exponential','gaussian'}
    assert all(len(row)==2 and all(math.isfinite(v) for v in row) for row in expected)
    print('OK I2EM: 20 original MATLAB HH/VV cases, SHA-256 verified')

if __name__=='__main__': verify()
