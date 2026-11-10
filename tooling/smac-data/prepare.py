"""Build one lossless coefficient resource from pinned upstream files (offline).

Original file bytes are retained inside the compressed resource for auditing and
export. No coefficient rounding, sensor substitution, or spectral interpolation.
"""
import hashlib
import json
import math
from pathlib import Path
import re
import zipfile
import zlib

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / '.reference-cache/smac/upstream'
DATA = ROOT / 'crates/models/atmosphere/smac/data'
AUTHOR = '77bf73dd1f2a2b6a33a97b8997da469210f9ce3e'
SEN2LIKE = '6dd3f0da82a7461b03a2914df56d305070e50d6d'
PREFIX = 'sen2like/sen2like/atmcor/smac/'
BANDS = ['B01','B02','B03','B04','B05','B06','B07','B08','B8A','B09','B10','B11','B12']

def sha(data):
    return hashlib.sha256(data).hexdigest()

def identity(name):
    match = re.fullmatch(r'coef_LANDSAT([457])_b(\d)_CONT.dat', name)
    if match:
        satellite, band = match.groups()
        sensor = 'etm-plus' if satellite == '7' else 'tm'
        return f'landsat-{satellite}.{sensor}.B{band}.continental', f'Landsat-{satellite} / {sensor.upper()}', f'B{band}', 'continental'
    match = re.fullmatch(r'Coef_LANDSAT8_(.*?)_1.dat', name)
    if match:
        band = {'440':'B1','490':'B2','560':'B3','660':'B4','860':'B5','1630':'B6','2250':'B7','PAN':'B8','1370':'B9'}[match[1]]
        return f'landsat-8.oli.{band}.continental', 'Landsat-8 / OLI', band, 'continental'
    match = re.fullmatch(r'coef_MODIS(\d+)_(CONT|DES).dat', name)
    if match:
        band, aerosol = match.groups(); aerosol = 'continental' if aerosol == 'CONT' else 'desert'
        return f'modis.B{band}.{aerosol}', 'MODIS (upstream platform unspecified)', f'B{band}', aerosol
    stem = name.removesuffix('.dat')
    # Keep legacy spelling/case in the stable ID: some original names differ only in case.
    aerosol = 'desert' if re.search(r'_DES(E)?$', stem) else 'continental' if re.search(r'_(CONT|1)$', stem) else 'unspecified'
    group = re.sub(r'^[Cc]oef_', '', stem).split('_')[0]
    return 'legacy.' + stem, group + ' / legacy', stem, aerosol

def build():
    sources = {}; records = []
    def source(key, raw, repository, commit, path):
        sources[key] = {'text':raw.decode('ascii'), 'sha256':sha(raw), 'url':f'https://raw.githubusercontent.com/{repository}/{commit}/{path}'}
    def record(ident, satellite, band, aerosol, key, values, column=None):
        if len(values) != 49 or not all(math.isfinite(x) for x in values):
            raise ValueError(f'invalid coefficient layout: {key}')
        records.append(dict(id=ident,satellite=satellite,band=band,aerosol=aerosol,source_key=key,column=column))
    archive = zipfile.ZipFile(CACHE/'source.zip')
    for path in sorted(archive.namelist()):
        if '/COEFS/' not in path or path.endswith('/'): continue
        name = path.split('/')[-1]
        # Prefer the maintained ESA copy for Landsat-8 (same named coefficients).
        if name.startswith('Coef_LANDSAT8_'): continue
        raw = archive.read(path); key = 'author/' + name
        source(key,raw,'olivierhagolle/SMAC',AUTHOR,'COEFS/'+name)
        values = [float(v) for v in raw.decode('ascii').split()]
        record(*identity(name),key,values)
    base = CACHE/'sen2like'/PREFIX
    for file in sorted((base/'COEFS').glob('*.dat')):
        raw = file.read_bytes(); key = 'sen2like/' + file.name
        source(key,raw,'senbox-org/sen2like',SEN2LIKE,PREFIX+'COEFS/'+file.name)
        if file.name.startswith('Coef_S2'):
            platform = file.name[7].lower()
            rows = [[float(v) for v in line.split()] for line in raw.decode().splitlines() if line.strip() and not line.lstrip().startswith('#')]
            if len(rows) != 49 or any(len(row) != 13 for row in rows): raise ValueError(file)
            for col, band in enumerate(BANDS):
                record(f'sentinel-2{platform}.msi.{band}.continental',f'Sentinel-2{platform.upper()} / MSI',band,'continental',key,[row[col] for row in rows],col)
        else:
            record(*identity(file.name),key,[float(v) for v in raw.decode().split()])
    records.sort(key=lambda r:r['id'])
    if len({r['id'] for r in records}) != len(records): raise ValueError('duplicate coefficient ID')
    payload = json.dumps({'format_version':1,'sources':sources,'records':records},ensure_ascii=True,separators=(',',':'),allow_nan=False).encode()
    packed = zlib.compress(payload,9)
    DATA.mkdir(parents=True,exist_ok=True)
    (DATA/'coefficients.json.zlib').write_bytes(packed)
    manifest = {'author_commit':AUTHOR,'sen2like_commit':SEN2LIKE,'pack_sha256':sha(packed),'payload_sha256':sha(payload),'records':len(records),'source_files':len(sources),'original_bytes':sum(len(s['text'].encode()) for s in sources.values()),'json_bytes':len(payload),'compressed_bytes':len(packed),'sources':{k:{p:v for p,v in s.items() if p!='text'} for k,s in sources.items()}}
    (ROOT/'tooling/smac-data/source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:v for k,v in manifest.items() if k!='sources'},indent=2))

if __name__ == '__main__': build()
