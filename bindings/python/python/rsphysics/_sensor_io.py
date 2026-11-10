"""Official-table IO only; no spectral integration. Standard library, no Excel runtime.

XLSX is read as XML without executing formulas/macros or extracting archive files.
Negative measured USGS response noise is explicitly zeroed and counted during
asset preparation; custom responses are validated strictly by the Rust kernel.
"""
import math
import re
import zipfile
import xml.etree.ElementTree as ET
from pathlib import PurePosixPath

_NS = {'m':'http://schemas.openxmlformats.org/spreadsheetml/2006/main'}

def worksheets(path):
    with zipfile.ZipFile(path) as archive:
        if sum(x.file_size for x in archive.infolist()) > 128*1024**2:
            raise ValueError('response workbook exceeds the supported uncompressed size')
        shared=[]
        if 'xl/sharedStrings.xml' in archive.namelist():
            for node in ET.fromstring(archive.read('xl/sharedStrings.xml')):
                shared.append(''.join(node.itertext()))
        relations={n.attrib['Id']:n.attrib['Target'] for n in ET.fromstring(archive.read('xl/_rels/workbook.xml.rels'))}
        workbook=ET.fromstring(archive.read('xl/workbook.xml'))
        for sheet in workbook.findall('m:sheets/m:sheet',_NS):
            rel=sheet.attrib['{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id']
            target=relations[rel]
            member=target.lstrip('/') if target.startswith('/') else str(PurePosixPath('xl')/target)
            if '..' in PurePosixPath(member).parts or not member.startswith('xl/'):
                raise ValueError('unsupported worksheet relationship')
            root=ET.fromstring(archive.read(member));rows=[]
            for row in root.findall('m:sheetData/m:row',_NS):
                values=[]
                for cell in row.findall('m:c',_NS):
                    letters=re.match(r'[A-Z]+',cell.attrib['r']).group();column=0
                    for char in letters:column=column*26+ord(char)-64
                    while len(values)<column:values.append(None)
                    text=cell.find('m:v',_NS);value=None
                    if cell.attrib.get('t')=='inlineStr':value=''.join(cell.find('m:is',_NS).itertext())
                    elif text is not None and text.text is not None:
                        if cell.attrib.get('t')=='s':value=shared[int(text.text)]
                        elif cell.attrib.get('t')=='str':value=text.text
                        elif cell.attrib.get('t')=='e':raise ValueError('formula/error cell in response workbook')
                        else:value=float(text.text)
                    values[column-1]=value
                rows.append(values)
            yield sheet.attrib['name'],rows

def _band(identifier,name,x,y,sheet):
    reordered=any(a>b for a,b in zip(x,x[1:]))
    if reordered:
        pairs=sorted(zip(x,y));x=[p[0] for p in pairs];y=[p[1] for p in pairs]
    if len(x)<2 or len(x)!=len(y) or any(not math.isfinite(v) for v in x+y) or any(a>=b for a,b in zip(x,x[1:])) or any(v<=0 for v in x):
        raise ValueError('invalid response samples: '+identifier)
    negative=sum(v<0 for v in y);minimum=min(y);y=[max(0.,v) for v in y]
    positive=[i for i,v in enumerate(y) if v>0]
    if not positive:raise ValueError('zero-area response: '+identifier)
    a=max(0,positive[0]-1);b=min(len(x),positive[-1]+2)
    return dict(id=identifier,name=name,wavelengths_nm=x[a:b],response=y[a:b],source_sheet=sheet,negative_samples_clipped=negative,minimum_original_response=minimum,source_rows_reordered=reordered)

def parse(path,entry):
    parser=entry['parser'];bands=[]
    if parser=='obpg':
        lines=path.read_text().splitlines();fields=next(line.split('=',1)[1].split(',') for line in lines if line.startswith('/fields='))
        rows=[[float(v) for v in line.split()] for line in lines if line.strip() and line.lstrip()[0] not in '/!#']
        if any(len(r)!=len(fields) for r in rows):raise ValueError('OBPG table column mismatch')
        for i,field in enumerate(fields[1:],1):
            # Keep provider wavelength labels, not inferred MODIS band numbering.
            bands.append(_band(field,field,[r[0] for r in rows],[r[i] for r in rows],'OBPG band-average RSR'))
    else:
        sheets=dict(worksheets(path))
        if parser=='s2':
            sheet=f"Spectral Responses ({entry['platform_code']})";rows=sheets[sheet];header=rows[0]
            data=[r for r in rows[1:] if r and isinstance(r[0],(int,float))]
            for i in range(1,14):
                identifier=header[i].rsplit('_',1)[-1]
                bands.append(_band(identifier,identifier,[r[0] for r in data],[r[i] for r in data],sheet))
        elif parser=='tirs8':
            sheet='TIRS BA RSR';rows=sheets[sheet][1:];rows=[r for r in rows if r and isinstance(r[0],(int,float))]
            for i in [1,2]:bands.append(_band('B'+str(i+9),'Thermal '+str(i+9),[r[0]*1000 for r in rows],[r[i] for r in rows],sheet))
        elif parser=='tirs9':
            for band in [10,11]:
                sheet=f'TIRS Band {band} BA RSR';rows=[r for r in sheets[sheet][1:] if r and isinstance(r[0],(int,float))]
                bands.append(_band('B'+str(band),'Thermal '+str(band),[r[0]*1000 for r in rows],[r[1] for r in rows],sheet))
        elif parser=='landsat':
            oli={'CoastalAerosol':1,'Blue':2,'Green':3,'Red':4,'NIR':5,'SWIR1':6,'SWIR2':7,'Pan':8,'Cirrus':9}
            for sheet,rows in sheets.items():
                if not rows or len(rows[0])<2 or str(rows[0][0]).lower()!='wavelength':continue
                match=re.search(r'Band\s*(\d+)',str(rows[0][1]))
                band=int(match.group(1)) if match else oli[sheet]
                data=[r for r in rows[1:] if r and isinstance(r[0],(int,float)) and len(r)>1 and isinstance(r[1],(int,float))]
                bands.append(_band('B'+str(band),sheet,[r[0] for r in data],[r[1] for r in data],sheet))
            bands.sort(key=lambda b:int(b['id'][1:]))
        else:raise ValueError('unknown official parser')
    if not bands or any(b['id'] not in entry['band_ids'] for b in bands) or [b['id'] for b in bands]!=entry['band_ids']:
        raise ValueError('official response band list differs from pinned registry')
    return dict(id=entry['id'],name=entry['name'],version=entry['version'],scope=entry['scope'],source=entry['source'],parser_version=1,bands=bands,preprocessing='Convert wavelength units to nm; sort out-of-order provider rows and record that fact; reject duplicate wavelengths; replace negative measured responses with zero and record counts/minima; trim only exact zero tails, retaining one zero shoulder. No Gaussian approximation, wavelength resampling or peak/area normalization. Response is not inferred outside the provider tabulation.')
