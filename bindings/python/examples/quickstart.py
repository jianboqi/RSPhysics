"""Discover a model, simulate a leaf spectrum, and export a NumPy parameter sweep.

Install rsphysics first. Add --plot after installing rsphysics[examples] to save
a figure. All calculations use packaged resources; no downloads are required.
"""

import argparse
from pathlib import Path

import numpy as np
import rsphysics as rsp


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=Path("rsphysics-output"))
    parser.add_argument("--plot", action="store_true", help="also save a PNG figure")
    args = parser.parse_args()

    print(f"RSPhysics {rsp.__version__}")
    print("Models:", ", ".join(model["metadata"]["id"] for model in rsp.list_models()))

    model_id = "vegetation.prospect-d"
    description = rsp.describe(model_id)
    chlorophyll = description["input_schema"]["properties"]["chlorophyll"]
    print("Default chlorophyll:", chlorophyll["default"])

    leaf = rsp.vegetation.prospect_d(chlorophyll=45., water=0.015)
    spectrum = leaf["result"]
    wavelengths = spectrum["wavelengths_nm"]
    print("Single reflectance spectrum:", spectrum["reflectance"].shape)

    # Cartesian product: three chlorophyll contents and two water contents.
    table = rsp.parameter_grid(model_id, {
        "chlorophyll": [20., 40., 60.],
        "water": [0.01, 0.02],
    })
    batch = rsp.run_array(model_id, table, shared_axes=True)
    reflectance = batch["result"]["reflectance"]
    print("Batch reflectance:", reflectance.shape)

    args.output_dir.mkdir(parents=True, exist_ok=True)
    np.savetxt(args.output_dir / "leaf-spectrum.csv",
               np.column_stack((wavelengths, spectrum["reflectance"], spectrum["transmittance"])),
               delimiter=",", header="wavelength_nm,reflectance,transmittance", comments="")
    np.savetxt(args.output_dir / "parameters.csv", table, delimiter=",",
               header=",".join(rsp.parameters_for(model_id)), comments="")
    np.savetxt(args.output_dir / "reflectance-sweep.csv",
               np.column_stack((batch["coordinates"]["wavelengths_nm"], reflectance.T)),
               delimiter=",", header="wavelength_nm," + ",".join(f"sample_{i}" for i in range(len(table))),
               comments="")

    if args.plot:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt

        columns = rsp.parameters_for(model_id)
        fig, ax = plt.subplots(figsize=(8, 4.5))
        for row, values in zip(table, reflectance):
            cab = row[columns.index("chlorophyll")]
            water = row[columns.index("water")]
            ax.plot(wavelengths, values, label=f"Cab={cab:g}, Cw={water:g}")
        ax.set(xlabel="Wavelength (nm)", ylabel="Leaf reflectance", ylim=(0, 1))
        ax.legend(title="Cab: µg/cm²; Cw: cm", fontsize=8, ncols=2)
        ax.grid(alpha=0.2)
        fig.tight_layout()
        fig.savefig(args.output_dir / "reflectance-sweep.png", dpi=160)
        plt.close(fig)
    print("Saved outputs to", args.output_dir.resolve())


if __name__ == "__main__":
    main()
