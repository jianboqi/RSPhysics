"""Fetch fixed-version original SMAC materials into ONE disposable reference cache."""
import hashlib
import json
from pathlib import Path
import urllib.request
import zipfile
import io

ROOT=Path(__file__).resolve().parents[2]
CACHE=ROOT/'.reference-cache/smac/upstream'
MANIFEST=json.loads((Path(__file__).parent/'source-manifest.json').read_text())

def fetch(url):
    with urllib.request.urlopen(url,timeout=90) as response:return response.read()

def main():
    CACHE.mkdir(parents=True,exist_ok=True)
    author=MANIFEST['author_commit'];esa=MANIFEST['sen2like_commit']
    archive=fetch(f'https://codeload.github.com/olivierhagolle/SMAC/zip/{author}')
    (CACHE/'source.zip').write_bytes(archive)
    zipped=zipfile.ZipFile(io.BytesIO(archive))
    for name in ['smac.py','readme.md']:(CACHE/name).write_bytes(zipped.read(f'SMAC-{author}/{name}'))
    for key,source in MANIFEST['sources'].items():
        if key.startswith('author/'):
            raw=zipped.read(f'SMAC-{author}/COEFS/'+key.split('/',1)[1])
            assert hashlib.sha256(raw).hexdigest()==source['sha256']
        else:
            raw=fetch(source['url']);assert hashlib.sha256(raw).hexdigest()==source['sha256']
            path=CACHE/'sen2like/sen2like/sen2like/atmcor/smac/COEFS'/key.split('/',1)[1]
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
    for file in ['sen2like/LICENSE.txt','sen2like/sen2like/atmcor/smac/smac.py','sen2like/sen2like/atmcor/smac/copyright.txt','sen2like/sen2like/atmcor/smac/readme.md']:
        path=CACHE/'sen2like'/file;path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes(fetch(f'https://raw.githubusercontent.com/senbox-org/sen2like/{esa}/{file}'))
    source=CACHE/'sen2like/sen2like/sen2like/atmcor/smac/smac.py'
    expected=json.loads((ROOT/'benchmarks/reference/smac/expected.json').read_text())['source_sha256']
    assert hashlib.sha256(source.read_bytes()).hexdigest()==expected
    print('Fetched and verified pinned original materials in',CACHE)

if __name__=='__main__':main()
