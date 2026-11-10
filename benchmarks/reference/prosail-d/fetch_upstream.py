"""Fetch only the author's fixed MIT-declared PROSAIL forward-model sources/data."""
import base64
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import time
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent
CACHE = ROOT.parents[2] / ".reference-cache/prosail-d/upstream"
COMMIT = "46549958295b605ac1e211a693d4b2287c3a5a13"
FILES = ["DESCRIPTION", "LICENSE", "R/prosail.R", "R/fourSAIL.R", "R/campbell.R", "R/volscatt.R",
         "R/dcum.R", "R/dladgen.R", "R/jfunc1.R", "R/jfunc2.R", "R/jfunc3.R", "R/jfunc4.R",
         "R/conservative_scattering.R", "R/non_conservative_scattering.R", "R/spec_soil-data.R",
         "R/adjust_prospect_to_sail.R", "data-raw/dataSpec_SOIL_ATM.txt"]


def fetch(name):
    raw = f"https://raw.githubusercontent.com/jbferet/prosail/{COMMIT}/{name}"
    api = f"https://api.github.com/repos/jbferet/prosail/contents/{name}?ref={COMMIT}"
    failure = None
    for url in (raw, api, raw, api):
        try:
            with urlopen(Request(url, headers={"User-Agent": "RSPhysics-reference-intake"}), timeout=30) as response:
                data = response.read()
            if url == api:
                data = base64.b64decode(json.loads(data)["content"])
            target = CACHE / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            return name, dict(url=raw, sha256=hashlib.sha256(data).hexdigest(), bytes=len(data))
        except Exception as error:
            failure = error
            time.sleep(1)
    raise RuntimeError(f"{name}: {failure}")


if __name__ == "__main__":
    with ThreadPoolExecutor(max_workers=4) as pool:
        files = dict(pool.map(fetch, FILES))
    result = dict(repository="https://github.com/jbferet/prosail", version="3.1.1", commit=COMMIT,
                  license_declaration="MIT + file LICENSE (fixed DESCRIPTION; forward-model subset only)", files=files)
    (ROOT / "upstream-provenance.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    for name, record in files.items():
        print(record["sha256"], name)
