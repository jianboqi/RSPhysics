"""Independent atmospheric conversions, coefficient audit, native NumPy scan."""
import hashlib
import json
import numpy as np
import rsphysics as rsp

coefficient='sentinel-2c.msi.B04.continental'
source=rsp.atmosphere.smac_coefficient_source(coefficient)
assert hashlib.sha256(source['text'].encode('ascii')).hexdigest()==source['sha256']
forward=rsp.atmosphere.smac(coefficient_id=coefficient,direction='surface_to_toa',reflectance=.3)
inverse=rsp.atmosphere.smac(coefficient_id=coefficient,reflectance=forward['result']['reflectance'])
np.testing.assert_allclose(inverse['result']['reflectance'],.3,atol=2e-12)
table=rsp.parameter_grid('atmosphere.smac',{'aot550':[.05,.1,.2,.4]})
batch=rsp.atmosphere.smac_batch(table,coefficient_id=coefficient,shared_axes=True)
assert batch['result']['reflectance'].shape==(4,)
assert batch['coordinates']['sweep_aot550'].shape==(21,)
print(json.dumps({'coefficient':coefficient,'source_sha256':source['sha256'],
                 'input_surface_reflectance':.3,'toa_reflectance':forward['result']['reflectance'],
                 'recovered_surface_reflectance':inverse['result']['reflectance'],
                 'aot550':table[:,5].tolist(),'corrected_surface_reflectance':batch['result']['reflectance'].tolist()},indent=2))
