# GOMS C references

Thirty-one cases cover structural parameters, 1–10 bands, absolute solar/view
azimuths, terrain slope/aspect and both numerical conventions. Each case has a
171-direction principal plane and a 648-direction hemisphere, including masks
for nonfinite or below-local-horizon geometry. The C harness copies the three
mixing weights into output globals without changing their calculation.

`original-c.json` retains the unmodified selected-point values.
`expected.json.zlib` contains the full grid for the convention selected by each
input. `provenance.json` records source/archive hashes, compiler settings and
the explicit area/angular corrections. `manifest.json` verifies fixture bytes.

At nadir and normalized density 0.1, the original general-overlap expression
gives ground weight 0.5895955307385816. Removing its inconsistent division by
pi gives exp(-0.1*pi) = 0.7304026910486456, consistent with the principal-plane
overlap and the Poisson gap probability. Terrain corrections update projected
tangents after rotation, fold azimuth separation, use atan2 at the normal
direction and apply all nonzero slopes.

Recompute with the separately supplied pinned source and a C compiler:

```sh
python tooling/reference-runner/verify_goms_source.py --source PATH/Goms.c
```

On Windows, run in a Visual Studio x64 developer prompt and add `--cc cl`.
The original source and compiled reference binaries are not distributed.
