"""Run every installed model once with its documented default parameters.

Default runs demonstrate the common interface. Model guides describe the
physical assumptions and parameter choices needed for a particular study.
"""

import numpy as np
import rsphysics as rsp


def main():
    for description in rsp.list_models():
        model_id = description["metadata"]["id"]
        simulation = rsp.run(model_id)
        result = simulation["result"]
        summary = ", ".join(
            f"{name}: {np.shape(value) or 'scalar'}"
            for name, value in list(result.items())[:3]
        )
        print(f"{model_id}\n  {summary}")


if __name__ == "__main__":
    main()
