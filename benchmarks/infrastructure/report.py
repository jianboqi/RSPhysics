"""Installed-wheel end-to-end evidence and native-vs-JSON transport timings.

Times are local medians, not a performance guarantee. Both paths compute the
same scientific models and retain manifests. No external response downloads.
"""
from pathlib import Path
import json
import platform
import statistics
import time
import numpy as np
import rsphysics as rsp
ROOT=Path(__file__).resolve().parents[2]
def main():
    timings=[]
    for model in ['vegetation.prospect-d','vegetation.prosail-d','surface.ross-li','microwave.tau-omega','microwave.water-cloud']:
        matrix=np.repeat(rsp.parameter_grid(model,{}),64,axis=0);columns=rsp.parameters_for(model)
        rows=[dict(zip(columns,row)) for row in matrix]
        native=rsp.run_array(model,matrix);legacy=rsp.run_batch(model,rows)
        for key,value in legacy[0]['result'].items():np.testing.assert_allclose(native['result'][key] if key=='wavelengths_nm' else native['result'][key][0],value,rtol=2e-12,atol=2e-12)
        def median(fn):
            samples=[]
            for _ in range(3):
                start=time.perf_counter();out=fn();samples.append((time.perf_counter()-start)*1000);del out
            return statistics.median(samples)
        a=median(lambda:rsp.run_array(model,matrix));b=median(lambda:rsp.run_batch(model,rows))
        timings.append(dict(model=model,samples=64,native_numpy_ms=a,json_runtime_ms=b,json_over_native_ratio=b/a))
    spectra=rsp.vegetation.prosail_d()['result'];sensors=[]
    for entry in rsp.sensors.list_sensors():
        if not entry['available']:continue
        sensor=rsp.sensors.load(entry['id'])
        if '.tirs' in sensor.id:
            # These assets are intentionally not applied to an optical spectrum.
            sensors.append(dict(sensor=sensor.id,bands=len(sensor.bands),status='thermal response loaded; optical coverage deliberately unsupported'));continue
        partial='modis' in sensor.id
        out=sensor.convolve(spectra['wavelengths_nm'],spectra['brf'],allow_partial=partial)
        sensors.append(dict(sensor=sensor.id,bands=len(sensor.bands),coverage=out['coverage'].tolist(),band_values=out['values'].tolist(),weighting=out['weighting'],source_sha256=out['source']['sha256']))
    report=dict(platform=platform.platform(),python=platform.python_version(),numpy=np.__version__,package=rsp.__version__,timing_note='Median of three local runs, 64 rows; not a universal speedup claim',timings=timings,prosail_to_sensor_examples=sensors)
    path=Path(__file__).parent/'validation-report.json';path.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(timings=timings,validated_sensor_profiles=len(sensors)),indent=2))
if __name__=='__main__':main()
