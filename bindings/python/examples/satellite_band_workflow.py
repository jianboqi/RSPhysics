"""PROSAIL -> official sensor bands, explicit SRF weighting, chunked parameter grid.

Run after pip install rsphysics. No imagery, inversion or model coupling.
Use --sentinel for the explicitly requested official S2 download/cache operation.
"""
import argparse
import json
import numpy as np
import rsphysics as rsp

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--sentinel',action='store_true',help='explicitly fetch/load ESA S2 v5 into the local cache');args=parser.parse_args()
    model='vegetation.prosail-d'
    table=rsp.parameter_grid(model,{'leaf_area_index':[1.,3.,5.],'chlorophyll':[20.,40.,60.]})
    sensors=[rsp.sensors.load('landsat-8.oli').select(['B2','B3','B4','B5','B6','B7']),rsp.sensors.load('landsat-9.oli-2').select(['B2','B3','B4','B5','B6','B7'])]
    if args.sentinel:sensors.append(rsp.sensors.fetch('sentinel-2a.msi'))
    rows=[]
    for chunk in rsp.iter_array(model,table,chunk_size=4,shared_axes=True):
        x=chunk['coordinates']['wavelengths_nm'];brf=chunk['result']['brf']
        for sensor in sensors:
            band=sensor.convolve(x,brf)
            rows.append(dict(row_offset=chunk['row_offset'],sensor=sensor.id,bands=band['bands'],response_weighted_brf=band['values'].tolist(),coverage=band['coverage'].tolist(),source_sha256=band['source']['sha256']))
    # Thermal response functions require a thermal spectrum; no optical extrapolation.
    calibrated=rsp.radiometry.linear_calibrate([0,10000,65535],gain=.0003342,offset=.1,unit='W m^-2 sr^-1 um^-1',nodata=0,saturation=65535)
    brightness=rsp.radiometry.radiance_to_brightness_temperature(calibrated['values'],k1=774.8853,k2=1321.0789,valid_mask=calibrated['valid'])
    print(json.dumps(dict(model=model,samples=len(table),weighting='response-only spectral summary; no solar spectrum or atmosphere has been assumed',bands=rows,illustrative_calibration=dict(valid=calibrated['valid'].tolist(),brightness_temperature_k=[float(v) if np.isfinite(v) else None for v in brightness['values']])),indent=2,allow_nan=False))
if __name__=='__main__':main()
