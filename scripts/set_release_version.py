#!/usr/bin/env python3
"""Update workspace package and internal crate dependency versions for release."""

from __future__ import annotations

import re
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: set_release_version.py <version>", file=sys.stderr)
        return 2

    version = sys.argv[1].strip()
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:[.-][0-9A-Za-z.-]+)?", version):
        print(f"invalid release version: {version}", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parents[1]
    manifest = root / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")

    text = re.sub(
        r'(\[workspace\.package\]\n(?:[^\[]*\n)*?version = ")[^"]+(")',
        rf"\g<1>{version}\2",
        text,
        count=1,
    )
    text = re.sub(
        r'(sora-[A-Za-z0-9_-]+ = \{ path = "crates/sora-[^"]+", version = ")[^"]+(" \})',
        rf"\g<1>{version}\2",
        text,
    )

    manifest.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
