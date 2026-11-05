#!/usr/bin/env python3
"""Check documentation links and forbidden local paths.

Runs quickly enough to be the only job on a documentation-only pull request.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = [ROOT / "docs", ROOT / "README.md", ROOT / "AGENTS.md", ROOT / "THIRD-PARTY.md"]
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
FORBIDDEN = ("/home/", "file://")


def markdown_files() -> list[Path]:
    files: list[Path] = []
    for item in DOCS:
        if item.is_dir():
            files.extend(item.rglob("*.md"))
        elif item.exists():
            files.append(item)
    return files


def main() -> int:
    errors: list[str] = []
    for path in markdown_files():
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(ROOT)
        for needle in FORBIDDEN:
            if needle in text:
                errors.append(f"{rel}: forbidden local path {needle!r}")
        for match in LINK.finditer(text):
            target = match.group(1)
            if target.startswith(("#", "http://", "https://", "mailto:")):
                continue
            file_part = target.split("#", 1)[0]
            if not file_part:
                continue
            resolved = (path.parent / file_part).resolve()
            if not resolved.exists():
                errors.append(f"{rel}: missing link target {target!r}")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        print(f"{len(errors)} documentation error(s)", file=sys.stderr)
        return 1
    print(f"documentation OK ({len(markdown_files())} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
