"""Independent 6SV2.1 spectra, atmospheric correction, and original report export."""

import argparse
import json
from pathlib import Path
from rsphysics import sixs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=Path("target/sixs-example"))
    parser.add_argument("--plot", action="store_true")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    run = sixs.run(correction=sixs.Correction.from_reflectance(0.25))
    result = run["result"]
    (args.output_dir / "report.txt").write_text(result["report"], encoding="utf-8")
    (args.output_dir / "result.json").write_text(
        json.dumps(run, default=lambda x: x.tolist(), indent=2), encoding="utf-8"
    )
    scan = sixs.spectrum([0.45, 0.55, 0.65, 0.85, 1.6])
    print("TOA reflectance:", result["apparent_reflectance"])
    print("TOA radiance W/(m^2 sr um):", result["apparent_radiance_w_m2_sr_um"])
    print("Corrected reflectance:", result["quantities"]["lambertian_case"][0])
    print("Spectral reflectance:", scan["apparent_reflectance"])
    if args.plot:
        import matplotlib.pyplot as plt

        axis = sixs.plot_spectrum(scan, marker="o", color="#187a61")
        axis.figure.savefig(args.output_dir / "spectrum.png", dpi=160)
        plt.close(axis.figure)


if __name__ == "__main__":
    main()
