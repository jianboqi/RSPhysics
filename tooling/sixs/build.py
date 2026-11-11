"""Command-line entry for the source-packaged 6SV2.1 build tools."""
from pathlib import Path
import runpy,sys
directory=Path(__file__).resolve().parents[2]/'crates/models/atmosphere/sixs/fortran/build_tools'
sys.path.insert(0,str(directory))
globals().update(runpy.run_path(str(directory/'build.py'),run_name=__name__))
