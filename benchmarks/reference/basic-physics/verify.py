"""Offline integrity check for the independent high-precision reference fixture."""
import hashlib
import json
from pathlib import Path
def main():
    folder=Path(__file__).parent;p=folder/'expected.json'
    assert hashlib.sha256(p.read_bytes()).hexdigest()==(folder/'fixture-sha256.txt').read_text().strip()
    data=json.loads(p.read_text())
    assert data['precision_digits']==80 and data['mpmath_version']=='1.3.0'
    for key in ['fresnel','planck_wavelength','planck_frequency']:assert len(data[key])==32
    print('32 Fresnel / 32 wavelength Planck / 32 frequency Planck independent references verified')
if __name__=='__main__':main()
