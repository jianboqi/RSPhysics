"""Installed-wheel verification; records measured error without vendor-code claims."""
import json
from pathlib import Path
import numpy as np
import rsphysics as rsp
def main():
    folder=Path(__file__).parent;reference=json.loads((folder/'expected.json').read_text());result={}
    maximum=0.;normalized=0.;count=0
    for case in reference['fresnel']:
        p=case['parameters'];actual=rsp.optics.fresnel(p['angle_deg'],complex(p['n_real'],p['k']),n_incident=p['n_incident'])
        for key,value in case['expected'].items():
            expected=complex(*value) if key.startswith('amplitude') else value
            difference=float(abs(actual[key]-expected));allowed=2e-12+2e-11*abs(expected)
            assert difference<=allowed
            maximum=max(maximum,difference);normalized=max(normalized,difference/allowed);count+=1
    result['fresnel']=dict(cases=32,compared_fields=count,max_absolute_error=maximum,max_normalized_error=normalized)
    for group in ['planck_wavelength','planck_frequency']:
        error=0.;inversion_error=0.;zeros=0
        for case in reference[group]:
            if group=='planck_wavelength':
                actual=float(rsp.radiometry.planck_radiance(case['wavelength_nm'],case['temperature_k']));expected=case['radiance_per_nm'];inverse=float(rsp.radiometry.inverse_planck(case['wavelength_nm'],expected))
            else:
                actual=float(rsp.radiometry.planck_frequency_radiance(case['frequency_hz'],case['temperature_k']));expected=case['radiance_per_hz'];inverse=float(rsp.radiometry.inverse_planck_frequency(case['frequency_hz'],expected))
            if expected==0:assert actual==0;zeros+=1
            else:error=max(error,abs(actual/expected-1));assert abs(actual/expected-1)<=2e-12
            target=case['inverse_temperature_k']
            if target==0:assert inverse==0
            else:inversion_error=max(inversion_error,abs(inverse/target-1));assert abs(inverse/target-1)<=2e-12
        result[group]=dict(cases=32,max_relative_forward_error=error,max_relative_inverse_error=inversion_error,zero_or_underflow_cases=zeros)
    output=dict(package_version=rsp.__version__,reference=reference['reference'],precision_digits=80,results=result,limits='Single passive nonmagnetic interface with lossless incident medium; vacuum blackbody radiance, not sensor-band or surface-temperature validation. No model coupling.')
    (folder/'validation-report.json').write_text(json.dumps(output,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
