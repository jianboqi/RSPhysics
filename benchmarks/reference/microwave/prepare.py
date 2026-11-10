"""Re-download immutable references only into the disposable .reference-cache."""
import hashlib
import json
from pathlib import Path
from urllib.request import Request,urlopen
ROOT=Path(__file__).resolve().parents[3]
COMMIT='f9bde397c32f8759f7a410cf3d55febac9034aef'
FILES={
 'tau-omega':('SMAP_ATBD_RevB.pdf','https://smap.jpl.nasa.gov/system/internal_resources/details/original/316_L2_SM_P_ATBD_v7_Sep2015.pdf','56f7a568e6efd0e92b7e3e939ea1fb4e31bbaa8dcc4c9cfeda4901e3a78ec207'),
 'water-cloud':('model.py',f'https://raw.githubusercontent.com/mcwhity/sense/{COMMIT}/sense/model.py','1131ace6d3d0c1ec8776fe2fb03bb226de3d956ae00c3cc5b0d095d449fc6fa3'),
}
def main():
 for slug,(name,url,sha) in FILES.items():
  path=ROOT/'.reference-cache'/slug/'upstream'/name;path.parent.mkdir(parents=True,exist_ok=True)
  data=path.read_bytes() if path.exists() else urlopen(Request(url,headers={'User-Agent':'RSPhysics-reference-intake'}),timeout=40).read()
  if hashlib.sha256(data).hexdigest()!=sha:raise ValueError('Reference hash mismatch: '+slug)
  path.write_bytes(data)
  record=dict(url=url,sha256=sha,bytes=len(data),reference_kind='official published equations, not operational code' if slug=='tau-omega' else 'SenSE developer implementation; not Attema/Ulaby 1978 original code',commit=COMMIT if slug=='water-cloud' else None,license_note='Test-only cache; not redistributed or linked. SenSE documentation declares GPL; scientific Rust solver independently implements published equations.')
  folder=ROOT/'benchmarks/reference'/slug;folder.mkdir(parents=True,exist_ok=True)
  (folder/'upstream-provenance.json').write_text(json.dumps(record,indent=2)+'\n')
  print(slug,sha)
if __name__=='__main__':main()
