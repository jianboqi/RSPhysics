"""Explicit downloads of hash-pinned official response sources into one cache."""
import argparse
import hashlib
import json
from pathlib import Path
from urllib.request import Request,urlopen
ROOT=Path(__file__).resolve().parents[2]
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--include-esa',action='store_true',help='also download the non-redistributable SentiWiki table for local use');args=parser.parse_args()
    entries=json.loads((Path(__file__).parent/'source-manifest.json').read_text())
    seen=set();folder=ROOT/'.reference-cache/sensors/upstream';folder.mkdir(parents=True,exist_ok=True)
    for entry in entries:
        if entry['distribution']!='bundled' and not args.include_esa:continue
        source=entry['source'];name=source['filename']
        if name in seen:continue
        seen.add(name);path=folder/name
        data=path.read_bytes() if path.exists() else urlopen(Request(source['url'],headers={'User-Agent':'RSPhysics-response-intake'}),timeout=45).read(source['bytes']+1)
        if hashlib.sha256(data).hexdigest()!=source['sha256']:raise ValueError('source changed: '+name)
        path.write_bytes(data);print(name,source['sha256'])
if __name__=='__main__':main()
