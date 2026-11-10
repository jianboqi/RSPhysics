"""Fetch a fixed authors' release, retaining each original file and its SHA-256.

No downloaded code is executed. The selected release declares MIT in DESCRIPTION;
newer releases have different licensing and are not substituted automatically.
"""
import base64
import hashlib
import json
from pathlib import Path
import time
from urllib.request import Request, urlopen

COMMIT = "4649d5006ddf6bce609951765a88ba2498fcd8ab"
REPOSITORY = "https://github.com/jbferet/prospect"
FILES = ["DESCRIPTION", "R/Lib_PROSPECT.R", "data-raw/dataSpec_PRO.txt",
         "data-raw/create_sysdata.R", "R/SpecPROSPECT_FullRange-data.R", "README.md"]
ROOT = Path(__file__).resolve().parent
CACHE = ROOT.parents[2] / ".reference-cache" / "prospect-d"


def fetch(path):
    raw = f"https://raw.githubusercontent.com/jbferet/prospect/{COMMIT}/{path}"
    api = f"https://api.github.com/repos/jbferet/prospect/contents/{path}?ref={COMMIT}"
    last = None
    for url in [raw, api, raw, api]:
        try:
            with urlopen(Request(url, headers={"User-Agent": "RSPhysics-reference-intake"}), timeout=30) as response:
                data = response.read()
            if url == api:
                data = base64.b64decode(json.loads(data)["content"])
            if not data:
                raise ValueError("empty upstream file")
            return raw, data
        except Exception as error:
            last = error
            time.sleep(1)
    raise RuntimeError(f"could not fetch {path}: {last}")


def main():
    manifest = {"repository": REPOSITORY, "tag": "v1.7.0", "commit": COMMIT,
                "license_declaration": "MIT (DESCRIPTION at the pinned commit)", "files": {}}
    for name in FILES:
        url, data = fetch(name)
        target = CACHE / "upstream" / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        digest = hashlib.sha256(data).hexdigest()
        manifest["files"][name] = {"url": url, "sha256": digest, "bytes": len(data)}
        print(f"{digest}  {name}")
    (ROOT / "upstream-provenance.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
