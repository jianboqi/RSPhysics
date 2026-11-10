"""Remove only the project's disposable original-reference cache (dry-run by default)."""
import argparse
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[2]
MODELS=['prospect-d','prosail-d','ross-li','tau-omega','water-cloud','sensors','smac','lmeb','gort-gpp']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--remove", action="store_true", help="actually remove the verified reference cache")
    parser.add_argument('--model',choices=MODELS+['all'],default='all')
    args = parser.parse_args()
    for model in (MODELS if args.model=='all' else [args.model]):
        clean(model,args.remove)


def clean(model,remove):
    TARGET=ROOT/'.reference-cache'/model
    base = (ROOT / ".reference-cache").resolve()
    target = TARGET.resolve()
    if model not in MODELS or target != base / model or not target.is_relative_to(ROOT.resolve()):
        raise ValueError("refusing to clean a path outside the designated project cache")
    if TARGET.is_symlink() or (TARGET.exists() and getattr(TARGET, "is_junction", lambda: False)()):
        raise ValueError("refusing a redirected cache directory")
    if not TARGET.exists():
        print(f"No reference cache: {target}")
        return
    size = sum(p.stat().st_size for p in TARGET.rglob("*") if p.is_file())
    print(f"{'Removing' if remove else 'Would remove'} {target} ({size / 1024**2:.1f} MiB)")
    if remove:
        shutil.rmtree(target)
        print("Removed reference downloads/runtime; model coefficients and offline regression fixtures retained")


if __name__ == "__main__":
    main()
