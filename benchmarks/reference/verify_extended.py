"""Offline byte identity and finite-number checks for complete forward references."""
from pathlib import Path
import hashlib
import json
import math
import zlib

ROOT=Path(__file__).resolve().parent


def finite(value):
    if isinstance(value,float):assert math.isfinite(value)
    elif isinstance(value,list):
        for item in value:finite(item)
    elif isinstance(value,dict):
        for item in value.values():finite(item)


if __name__=='__main__':
    records=json.loads((ROOT/'extended-manifest.json').read_text(encoding='utf-8'))
    for relative,want in records.items():
        path=(ROOT/relative).resolve();assert path.is_relative_to(ROOT.resolve())
        raw=path.read_bytes();assert hashlib.sha256(raw).hexdigest()==want,relative
        if path.suffix=='.zlib':raw=zlib.decompress(raw)
        if '.json' in path.name:finite(json.loads(raw))
    print(f'OK {len(records)} extended reference/data/provenance files; all JSON numbers finite')
