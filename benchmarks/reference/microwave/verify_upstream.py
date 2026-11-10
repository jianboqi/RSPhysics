"""Offline fixture/provenance verification; original cache is optional."""
import argparse
import hashlib
import json
from pathlib import Path
from prepare import FILES,ROOT
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--require-cache',action='store_true');args=parser.parse_args()
 hashes=json.loads((Path(__file__).parent/'fixture-hashes.json').read_text())
 for slug,(name,url,sha) in FILES.items():
  folder=ROOT/'benchmarks/reference'/slug
  for filename,expected in hashes[slug].items():
   assert hashlib.sha256((folder/filename).read_bytes()).hexdigest()==expected,(slug,filename)
  provenance=json.loads((folder/'upstream-provenance.json').read_text());assert provenance['url']==url and provenance['sha256']==sha
  path=ROOT/'.reference-cache'/slug/'upstream'/name
  if args.require_cache:assert path.is_file(),path
  if path.exists():assert hashlib.sha256(path.read_bytes()).hexdigest()==sha,path
  print(slug+': pinned fixture/source provenance verified; cache '+('present' if path.exists() else 'not required'))
if __name__=='__main__':main()
