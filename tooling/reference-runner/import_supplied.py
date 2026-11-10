"""Import exact supplied reference sources into a disposable, verified cache.

Examples:
  python tooling/reference-runner/import_supplied.py lmeb --source PATH/LMEB.rar
  python tooling/reference-runner/import_supplied.py gort-gpp --source PATH/GPP
Only recorded source files are copied; observation spreadsheets are not imported.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('model',choices=['lmeb','gort-gpp'])
    parser.add_argument('--source',required=True,type=Path)
    args=parser.parse_args()
    source=args.source.resolve(strict=True)
    cache=(ROOT/'.reference-cache'/args.model).resolve()
    if not cache.is_relative_to((ROOT/'.reference-cache').resolve()):
        raise ValueError('reference cache resolves outside designated root')
    cache.mkdir(parents=True,exist_ok=True)
    record=json.loads((ROOT/'benchmarks/reference'/args.model/'upstream-provenance.json').read_text())
    with tempfile.TemporaryDirectory(prefix='import-',dir=cache) as tmp:
        if source.is_file():
            if args.model!='lmeb':raise ValueError('GORT-GPP requires the GPP source directory')
            names=subprocess.check_output(['tar','-tf',str(source)],text=True).splitlines()
            if any(Path(n).is_absolute() or '..' in Path(n).parts or ':' in n
                   or not (n.rstrip('/')=='LMEB' or n.startswith('LMEB/')) for n in names):
                raise ValueError('archive has an unexpected or unsafe path')
            subprocess.run(['tar','-xf',str(source),'-C',tmp],check=True)
            source_root=Path(tmp)/'LMEB'
        else:
            source_root=source
        target=cache/('LMEB' if args.model=='lmeb' else 'source')
        for entry in record['files']:
            rel=Path(entry['path'])
            src=(source_root/rel).resolve(strict=True)
            if not src.is_relative_to(source_root.resolve()):raise ValueError('redirected source path')
            raw=src.read_bytes()
            if len(raw)!=entry['size_bytes'] or hashlib.sha256(raw).hexdigest()!=entry['sha256']:
                raise ValueError(f'original source differs from recorded reference: {rel}')
            dst=target/rel
            if not dst.resolve().is_relative_to(cache):raise ValueError('redirected target path')
            dst.parent.mkdir(parents=True,exist_ok=True)
            shutil.copyfile(src,dst)
    print(f'Imported and verified {len(record["files"])} files into {cache}')

if __name__=='__main__':main()
