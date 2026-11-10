"""Versioned official SRFs and custom bandpasses. All numeric integration is Rust.

USGS/NASA profiles are bundled and work offline. Sentinel-2 v5 A/B/C metadata
is registered, but its response samples must be fetched/imported locally due
to the provider's redistribution restrictions. load() never accesses a network.
"""
from dataclasses import dataclass
from pathlib import Path
from importlib.resources import files
from urllib.request import Request, urlopen
import copy
import hashlib
import json
import os
import tempfile
import time
import numpy as np
from ._api import ModelError, _call
from ._numeric import axis, real_array
from . import _native, spectral, _sensor_io

_RESOURCES=files('rsphysics').joinpath('data/sensors')

def _registry():
    return json.loads(_RESOURCES.joinpath('registry.json').read_text(encoding='utf-8'))

def _entry(sensor_id):
    for item in _registry():
        if item['id']==sensor_id:return item
    raise ModelError('invalid_parameter', f'unknown sensor {sensor_id!r}; use sensors.list_sensors()')

def _cache(cache_dir=None):
    if cache_dir is not None:return Path(cache_dir)
    if os.environ.get('RSPHYSICS_DATA_DIR'):return Path(os.environ['RSPHYSICS_DATA_DIR'])
    if os.name=='nt':return Path(os.environ.get('LOCALAPPDATA',Path.home()/'AppData/Local'))/'rsphysics/sensors'
    return Path(os.environ.get('XDG_CACHE_HOME',Path.home()/'.cache'))/'rsphysics/sensors'

def list_sensors(*, cache_dir=None):
    """Metadata for 12 pinned profiles; available means bundled or locally cached.

    MODIS includes the 16 NASA OBPG reflective responses, not all 36 bands.
    Landsat TM/ETM+ profiles include the supplied reflective-band subsets.
    """
    return [{**item,'available':item['distribution']=='bundled' or (_cache(cache_dir)/item['asset']).is_file()} for item in _registry()]

@dataclass(frozen=True)
class Bandpass:
    """One tabulated relative response; no Gaussian or out-of-table extrapolation."""
    id: str
    wavelengths_nm: np.ndarray
    response: np.ndarray

    def __post_init__(self):
        x=axis(self.wavelengths_nm).copy();r=real_array(self.response,ndim=1,name='response').copy()
        _call(_native.spectral_convolve,x,np.zeros(x.size),x,r,np.empty(0),np.empty(0),False)
        x.flags.writeable=False;r.flags.writeable=False
        object.__setattr__(self,'wavelengths_nm',x);object.__setattr__(self,'response',r)

    def convolve(self,wavelengths_nm,values,**kwargs):
        return spectral.convolve(wavelengths_nm,values,self.wavelengths_nm,self.response,**kwargs)

class Sensor:
    """An ordered set of independent band responses with source provenance."""
    def __init__(self,profile):
        self.id=profile['id'];self.name=profile['name'];self.version=profile['version']
        self._profile=copy.deepcopy(profile)
        self.bands=tuple(Bandpass(b['id'],b['wavelengths_nm'],b['response']) for b in profile['bands'])

    @property
    def metadata(self):return copy.deepcopy(self._profile)

    @property
    def band_ids(self):return tuple(b.id for b in self.bands)

    def select(self,band_ids):
        """Explicit ordered subset; no guessing or silently dropping uncovered bands."""
        if isinstance(band_ids,str):band_ids=[band_ids]
        band_ids=list(band_ids)
        if not band_ids or len(set(band_ids))!=len(band_ids):raise ModelError('invalid_parameter','band selection must be nonempty and unique')
        lookup={b['id']:b for b in self._profile['bands']}
        try:bands=[lookup[key] for key in band_ids]
        except KeyError as e:raise ModelError('invalid_parameter',f'unknown band {e.args[0]} for {self.id}') from e
        return Sensor(self._profile|dict(bands=bands))

    def convolve(self,wavelengths_nm,values,*,irradiance=None,irradiance_wavelengths_nm=None,allow_partial=False):
        """Result values (...,band); coverage/centroids are per-band arrays.

        Pass irradiance for an illumination-weighted reflectance summary.
        Response-only averaging never adds atmospheric/solar/geometry corrections.
        Partial support must be explicit and is reported, never silently filled.
        """
        if not isinstance(allow_partial,(bool,np.bool_)):raise ModelError('invalid_parameter','allow_partial must be boolean')
        x=axis(wavelengths_nm);ix,iy=spectral._illumination(x,irradiance,irradiance_wavelengths_nm)
        curves=[(b.wavelengths_nm.tolist(),b.response.tolist()) for b in self.bands]
        out=_call(_native.spectral_convolve_many,x,real_array(values),curves,ix,iy,bool(allow_partial))
        out.update(bands=self.band_ids,sensor=self.id,version=self.version,source=copy.deepcopy(self._profile['source']),weighting='response' if irradiance is None else 'response_times_irradiance')
        return out

def load(sensor_id,*,cache_dir=None):
    """Load an integrity-checked pinned profile offline; never downloads implicitly."""
    entry=_entry(sensor_id)
    if entry['distribution']=='bundled':data=_RESOURCES.joinpath(entry['asset']).read_bytes()
    else:
        path=_cache(cache_dir)/entry['asset']
        if not path.is_file():raise ModelError('missing_data',f'{sensor_id} requires a local official table: call sensors.fetch({sensor_id!r}) or sensors.import_file(...). No network is used by load().')
        data=path.read_bytes()
    if hashlib.sha256(data).hexdigest()!=entry['processed_sha256']:raise ModelError('missing_data',f'processed response checksum mismatch for {sensor_id}; re-import the pinned official source')
    return Sensor(json.loads(data))

def _atomic(path,data):
    path.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent,delete=False) as temp:
        name=Path(temp.name);temp.write(data)
    try:os.replace(name,path)
    finally:
        if name.exists():name.unlink()

def import_file(sensor_id,path,*,cache_dir=None):
    """Import the exact official source identified by registry SHA-256.

    Unknown versions are rejected instead of being mislabeled. Use Bandpass for
    custom response arrays. Importing the shared S2 v5 workbook installs A/B/C.
    """
    entry=_entry(sensor_id);path=Path(path);data=path.read_bytes()
    if hashlib.sha256(data).hexdigest()!=entry['source']['sha256']:raise ModelError('missing_data','official source checksum mismatch; this is not the pinned response version')
    if entry['distribution']=='bundled':return load(sensor_id,cache_dir=cache_dir)
    _atomic(_cache(cache_dir)/'raw'/entry['source']['filename'],data)
    for item in _registry():
        if item['source']['sha256']!=entry['source']['sha256']:continue
        try:profile=_sensor_io.parse(path,item)
        except (ValueError,KeyError,IndexError) as e:raise ModelError('missing_data','official response parser failed: '+str(e)) from e
        prepared=(json.dumps(profile,ensure_ascii=False,separators=(',',':'))+'\n').encode()
        if hashlib.sha256(prepared).hexdigest()!=item['processed_sha256']:raise ModelError('missing_data','processed official response differs from the pinned asset')
        _atomic(_cache(cache_dir)/item['asset'],prepared)
    return load(sensor_id,cache_dir=cache_dir)

def fetch(sensor_id,*,cache_dir=None):
    """Explicitly download a pinned official source, verify it, and cache locally.

    Bundled profiles require no download. No data is sent except an ordinary
    request to the provider. A changed upstream table fails checksum validation.
    """
    entry=_entry(sensor_id)
    if entry['distribution']=='bundled':return load(sensor_id,cache_dir=cache_dir)
    try:return load(sensor_id,cache_dir=cache_dir)
    except ModelError:pass
    source=entry['source'];path=_cache(cache_dir)/'raw'/source['filename']
    if path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest()==source['sha256']:
        return import_file(sensor_id,path,cache_dir=cache_dir)
    last=None
    for attempt in range(3):
        try:
            with urlopen(Request(source['url'],headers={'User-Agent':'RSPhysics-SRF/0.1'}),timeout=30) as response:data=response.read(source['bytes']+1)
            if hashlib.sha256(data).hexdigest()!=source['sha256']:raise ModelError('missing_data','downloaded official source checksum mismatch')
            _atomic(path,data)
            return import_file(sensor_id,path,cache_dir=cache_dir)
        except ModelError:raise
        except OSError as error:
            last=error
            if attempt<2:time.sleep(.3)
    raise ModelError('missing_data',f'official download failed; use import_file with an existing official table: {last}') from last
