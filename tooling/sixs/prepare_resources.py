"""Command-line entry for the source-packaged 6SV2.1 build tools."""
from pathlib import Path
import runpy,sys
directory=Path(__file__).resolve().parents[2]/'components/6sv2.1/build_tools'
sys.path.insert(0,str(directory))
globals().update(runpy.run_path(str(directory/'prepare_resources.py'),run_name=__name__))
