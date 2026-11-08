#!/usr/bin/env python3
"""Exact Python-reference vs native Rust AST JSON comparison for one-line inputs.

Only compatible en/sk reference fixtures are included, including multiline source. This is a test tool;
the Rust Nobody executable itself never imports or launches Python.
"""
from pathlib import Path
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "tests" / "fixtures" / "ast_json_parity"
RUST = ROOT / "rust" / "target" / "debug" / "nobody"


def invoke(args):
    return subprocess.run(
        args,
        cwd=ROOT,
        env={**os.environ, "PYTHONPATH": str(ROOT)},
        capture_output=True,
        text=True,
        timeout=15,
        check=False,
    )


def main():
    if not RUST.is_file():
        print("Native Rust nobody executable missing: build with cargo --locked", file=sys.stderr)
        return 2
    fixtures = sorted(FIXTURES.glob("*.nbd"))
    if len(fixtures) != 13:
        print(f"Expected 13 AST JSON fixtures, found {len(fixtures)}", file=sys.stderr)
        return 2
    errors = 0
    for fixture in fixtures:
        profile = fixture.name.split("_", 1)[0]
        python = invoke([
            sys.executable, "-m", "nobody",
            "ast-json", str(fixture), "--lang", profile,
        ])
        rust = invoke([str(RUST), "ast-json", profile, str(fixture)])
        ok = (
            python.returncode == 0
            and rust.returncode == 0
            and python.stdout == rust.stdout
        )
        print(f"{'PASS' if ok else 'FAIL'} {fixture.name}")
        if not ok:
            errors += 1
            print(f"  Python: rc={python.returncode} out={python.stdout!r} err={python.stderr!r}")
            print(f"  Rust: rc={rust.returncode} out={rust.stdout!r} err={rust.stderr!r}")
    if errors:
        print(f"AST JSON parity mismatch: {errors}/{len(fixtures)} fixtures", file=sys.stderr)
        return 1
    print("Python vs Rust single-line AST JSON parity passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
