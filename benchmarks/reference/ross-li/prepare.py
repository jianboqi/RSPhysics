"""Fetch immutable original AMBRALS sources into the disposable cache; record provenance."""
import base64,hashlib,json,time
from pathlib import Path
from urllib.request import Request,urlopen
ROOT=Path(__file__).resolve().parents[3]
COMMIT='b43d9171ea9f6dc76854cac34037e2af7903d02e'
REPO='arthurelmes/ambrals_forward'
HASHES={'ambralsfor.c':'96ee82999c66a7e7e64e9c6167ddb7b34e46ac62fcaad0844aab56462f155f28','ambralsfor.h':'f562ddb308a82c748f497d81f31c10696143b5a18deb91cbf0fbd22fad0a0e92','readme.txt':'d7c2507264c7f2f8e80b02568a948973d7139dd8fcee9d7e5398b6372ccfeed8'}
def fetch(name):
    raw=f'https://raw.githubusercontent.com/{REPO}/{COMMIT}/{name}'
    api=f'https://api.github.com/repos/{REPO}/contents/{name}?ref={COMMIT}'
    for url in [raw,api,raw]:
        try:
            content=urlopen(Request(url,headers={'User-Agent':'RSPhysics-reference-intake'}),timeout=25).read()
            if url==api:content=base64.b64decode(json.loads(content)['content'])
            return content,raw
        except Exception:
            time.sleep(1)
    raise RuntimeError('cannot obtain pinned source '+name)
def main():
    cache=ROOT/'.reference-cache/ross-li/upstream';cache.mkdir(parents=True,exist_ok=True)
    records={}
    for name in ['ambralsfor.c','ambralsfor.h','readme.txt']:
        path=cache/name
        if path.exists():data=path.read_bytes();url=f'https://raw.githubusercontent.com/{REPO}/{COMMIT}/{name}'
        else:data,url=fetch(name);path.write_bytes(data)
        if hashlib.sha256(data).hexdigest()!=HASHES[name]:raise ValueError('pinned source hash mismatch: '+name)
        records[name]=dict(url=url,sha256=hashlib.sha256(data).hexdigest(),bytes=len(data))
    result=dict(repository='https://github.com/'+REPO,commit=COMMIT,reference_identity='Original AMBRALS forward C routines in a public mirror; comments trace to ambmodels.c.reciprocal. Not asserted to be a NASA operational checkout.',original_download='ftp://rsftp.eeos.umb.edu/data01/Website/brdf_forward.tar.gz',official_tools_page='https://www.umb.edu/spectralmass/terra-aqua-modis/modis-user-tools/',original_download_status='FTP/HTTP/HTTPS timed out at intake',license_status='No explicit redistribution license found; original C stays in disposable test-only cache and is not distributed in packages.',files=records)
    (Path(__file__).resolve().parent/'upstream-provenance.json').write_text(json.dumps(result,indent=2)+'\n')
    print('Pinned original AMBRALS sources: '+COMMIT)
if __name__=='__main__':main()
