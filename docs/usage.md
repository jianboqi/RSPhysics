# Basic usage

## First steps

Install RSPhysics following the [installation guide](installation.md). The
examples below run locally using the resources included in the package.

## Run a model

Model-specific functions accept named physical parameters. Unspecified inputs
use the model's defaults.

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
result = leaf['result']
print(result['wavelengths_nm'].shape)  # (2101,)
print(result['reflectance'].shape)    # (2101,)
```

Each call returns `result` and `manifest`. `result` contains physical outputs;
`manifest` records the model, versions, and parameters used. Units and result
definitions are described on the individual model pages.

## Discover models and parameters

```python
model_ids = [model['metadata']['id'] for model in rsp.list_models()]
description = rsp.describe('vegetation.prospect-d')
parameters = description['input_schema']['properties']
print(parameters['chlorophyll']['default'])
print(parameters['chlorophyll']['x-unit'])  # ug cm^-2

# The same model through the common interface.
leaf = rsp.run('vegetation.prospect-d', {'chlorophyll': 45})
```

## NumPy batches

Use a parameter grid for a small experiment:

```python
table = rsp.parameter_grid('vegetation.prospect-d', {
    'chlorophyll': [20., 40., 60.],
    'water': [0.01, 0.02],
})
batch = rsp.run_array('vegetation.prospect-d', table, shared_axes=True)
wavelengths = batch['coordinates']['wavelengths_nm']
reflectance = batch['result']['reflectance']
print(reflectance.shape)  # (6, 2101)
```

For an existing NumPy matrix, obtain the column order with
`rsp.parameters_for(model_id)`. The input shape is `(samples, parameters)`.
With `shared_axes=True`, fixed wavelength/angle coordinates are returned once
under `coordinates`; output arrays keep the sample dimension first.

Discrete selectors, such as a correlation function or optical coefficient set,
are passed in `fixed`. They are not numeric matrix columns. For example:

```python
roughness = rsp.parameter_grid('microwave.i2em', {
    'observation_zenith_deg': [20., 30., 40.],
})
radar = rsp.run_array('microwave.i2em', roughness,
                      fixed={'correlation_function': 'gaussian'})
print(radar['result']['sigma0_vv_db'].shape)  # (3,)
```

For parameter mappings that contain strings or multi-band vectors, use
`rsp.run_batch(model_id, [parameters1, parameters2])`. Individual model guides
describe dedicated numeric batch functions and their array shapes.

Process larger tables in chunks:

```python
for chunk in rsp.iter_array('vegetation.prospect-d', table,
                            chunk_size=2, shared_axes=True):
    print(chunk['row_offset'], chunk['result']['reflectance'].shape)
```

## Save and plot results

NumPy results can be exported directly. This saves one spectrum with its
wavelength and dimensionless reflectance/transmittance:

```python
from pathlib import Path
import numpy as np

output_dir = Path('rsphysics-output')
output_dir.mkdir(exist_ok=True)
np.savetxt(output_dir / 'leaf-spectrum.csv',
           np.column_stack((result['wavelengths_nm'],
                            result['reflectance'], result['transmittance'])),
           delimiter=',', header='wavelength_nm,reflectance,transmittance',
           comments='')
```

Matplotlib is optional. Install it with `pip install "rsphysics[examples]"`,
then create a figure:

```python
import matplotlib
matplotlib.use('Agg')  # Save a figure without requiring a graphical desktop.
import matplotlib.pyplot as plt

fig, ax = plt.subplots()
for values in reflectance:
    ax.plot(wavelengths, values)
ax.set(xlabel='Wavelength (nm)', ylabel='Leaf reflectance', ylim=(0, 1))
fig.tight_layout()
fig.savefig(output_dir / 'reflectance-sweep.png', dpi=160)
plt.close(fig)
```

The complete [quickstart script](../bindings/python/examples/quickstart.py)
also exports the parameter table so each curve can be traced to its inputs:

```sh
python bindings/python/examples/quickstart.py --plot --output-dir rsphysics-output
```

The command assumes a repository checkout. Alternatively, download the script
and run it with the installed package. [All models](../bindings/python/examples/all_models.py)
runs every installed model once through the common interface. Model defaults
demonstrate the API; choose inputs appropriate to the scientific application.

## Input errors

Invalid inputs raise `rsp.ModelError`. Batch errors identify the zero-based row.
Parameter domains describe supported computations; they do not guarantee that
every combination represents a realistic material, canopy, or observing condition.

See [Models](README.md#models) for scientific scope and parameter meanings.
