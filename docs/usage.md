# Basic usage

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

Process larger tables in chunks:

```python
for chunk in rsp.iter_array('vegetation.prospect-d', table,
                            chunk_size=2, shared_axes=True):
    print(chunk['row_offset'], chunk['result']['reflectance'].shape)
```

## Input errors

Invalid inputs raise `rsp.ModelError`. Batch errors identify the zero-based row.
Parameter domains describe supported computations; they do not guarantee that
every combination represents a realistic material, canopy, or observing condition.

See [Models](README.md#models) for scientific scope and parameter meanings.
