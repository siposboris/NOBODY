#!/usr/bin/env python3
"""Update or verify the README roadmap checklist progress marker.

Counts top-level checkbox bullets in docs/ROADMAP.md. This metric measures
checklist items, NOT compiler completeness or release readiness.
"""
from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ROADMAP = ROOT / "docs" / "ROADMAP.md"
README = ROOT / "README.md"
BEGIN = "<!-- roadmap-progress:start -->"
END = "<!-- roadmap-progress:end -->"
CHECKBOX = re.compile(r"^- \[([ x])\] ", re.MULTILINE)


def render() -> str:
    marks = CHECKBOX.findall(ROADMAP.read_text(encoding="utf-8"))
    if not marks:
        raise ValueError("No roadmap checkboxes found; refusing to report 100%.")
    done = marks.count("x")
    total = len(marks)
    percent = (100 * done + total // 2) // total
    filled = (20 * done + total // 2) // total
    bar = "█" * filled + "░" * (20 - filled)
    return (
        f"{BEGIN}\n"
        f"### Roadmap checklist progress — {percent}%\n\n"
        f"**{done} / {total} roadmap checklist items marked complete** "
        f"({total - done} still open)\n\n"
        f"`{bar}` **{percent}%**\n\n"
        "This measures **completed checklist entries**, not the percentage of "
        "a working compiler, feature parity, quality, or production readiness. "
        "Some checked items are historical prototypes or partial milestones. "
        "Nobody is experimental; its Rust runtime, native compiler and "
        "self-hosting are not complete.\n\n"
        "See the [full roadmap](docs/ROADMAP.md) and "
        "[feature status](docs/FEATURES.md). "
        "Regenerate with `cargo run --locked --manifest-path rust/Cargo.toml --bin nobody-roadmap -- --write`.\n"
        f"{END}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if README marker is stale")
    args = parser.parse_args()
    data = README.read_text(encoding="utf-8")
    if data.count(BEGIN) != 1 or data.count(END) != 1:
        raise ValueError("README must have exactly one ordered progress marker pair.")
    start = data.index(BEGIN)
    end = data.index(END)
    if end < start:
        raise ValueError("README progress markers are reversed.")
    revised = data[:start] + render() + data[end + len(END):]
    if args.check:
        if data != revised:
            print("Roadmap progress is stale. Run: cargo run --locked --manifest-path rust/Cargo.toml --bin nobody-roadmap -- --write")
            return 1
        print("Roadmap progress is up to date.")
        return 0
    if data != revised:
        README.write_text(revised, encoding="utf-8")
    print("Roadmap progress updated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
