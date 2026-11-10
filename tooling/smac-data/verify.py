"""Offline audit of the compressed SMAC resource and original-reference fixture."""
import hashlib
import json
from pathlib import Path
import zlib

ROOT=Path(__file__).resolve().parents[2]
def main():
    manifest=json.loads((Path(__file__).parent/'source-manifest.json').read_text())
    packed=(ROOT/'crates/models/atmosphere/smac/data/coefficients.json.zlib').read_bytes()
    digest=lambda b:hashlib.sha256(b).hexdigest()
    assert digest(packed)==manifest['pack_sha256']
    payload=zlib.decompress(packed);assert digest(payload)==manifest['payload_sha256']
    lib=json.loads(payload);assert len(lib['records'])==manifest['records']==367
    assert len(lib['sources'])==manifest['source_files']==331
    assert len({r['id'] for r in lib['records']})==367
    for key,source in lib['sources'].items():
        assert digest(source['text'].encode('ascii'))==source['sha256']==manifest['sources'][key]['sha256']
        assert source['url']==manifest['sources'][key]['url']
    for record in lib['records']:
        rows=[[float(v) for v in row.split()] for row in lib['sources'][record['source_key']]['text'].splitlines() if row.strip() and not row.startswith('#')]
        values=[v for row in rows for v in row] if record['column'] is None else [row[record['column']] for row in rows]
        assert len(values)==49
    ref=ROOT/'benchmarks/reference/smac';data=(ref/'expected.json').read_bytes()
    assert digest(data)==(ref/'fixture-sha256.txt').read_text().split()[0]
    suite=json.loads(data);assert suite['coefficient_pack_sha256']==digest(packed)
    assert len(suite['cases'])==839
    print(f"OK SMAC: {len(lib['records'])} coefficient sets, 331 exact original files, 839 reference cases; {len(packed)} compressed bytes")

if __name__=='__main__':main()
