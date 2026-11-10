"""Execute public Python examples and check local Markdown links offline."""

from pathlib import Path
from contextlib import chdir
import re
from tempfile import TemporaryDirectory


def verify_pages(pages):
    blocks = 0
    for page in pages:
        text = page.read_text(encoding="utf-8")
        namespace = {"__name__": "__documentation__"}
        for index, code in enumerate(re.findall(r"```python\r?\n(.*?)```", text, re.S), 1):
            exec(compile(code, f"{page}:example-{index}", "exec"), namespace)
            blocks += 1
        for destination in re.findall(r"\]\(([^)]+)\)", text):
            if not destination.startswith(("https:", "http:", "#", "mailto:")):
                target = page.parent / destination.split("#")[0]
                if not target.exists():
                    raise ValueError(f"Broken local link in {page}: {destination}")
    print(f"{blocks} Python examples passed; local links checked in {len(pages)} pages")


def main():
    root = Path(__file__).resolve().parents[2]
    pages = [root / "README.md", root / "bindings/python/README.md", *sorted((root / "docs").rglob("*.md"))]
    with TemporaryDirectory(prefix="rsphysics-docs-") as directory, chdir(directory):
        verify_pages(pages)


if __name__ == "__main__":
    main()
