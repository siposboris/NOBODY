#!/usr/bin/env python3
"""Compare the legacy Python and Rust Nobody runtimes on shared fixtures.

The reference runner is test-only. The Rust binary never calls Python.
This does NOT establish full language semantic equivalence.
"""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "tests" / "fixtures" / "runtime_parity"
RUST_BIN = ROOT / "rust" / "target" / "debug" / "nobody-rs"


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT,
        env={**os.environ, "PYTHONPATH": str(ROOT)},
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=15,
        check=False,
    )


def main() -> int:
    if not RUST_BIN.is_file():
        print("Missing Rust binary; run cargo build --locked --manifest-path rust/Cargo.toml --bin nobody-rs", file=sys.stderr)
        return 2
    cases = sorted(FIXTURES.glob("*.nbd"))
    if len(cases) != 6:
        print(f"Expected 6 checked-in differential fixtures, found {len(cases)}", file=sys.stderr)
        return 2
    failures = 0
    for fixture in cases:
        profile = fixture.name.split("_", 1)[0]
        reference = run([
            sys.executable, "-m", "nobody", "ast-run", str(fixture),
            "--lang", profile,
        ])
        rust = run([str(RUST_BIN), "run", profile, str(fixture)])
        same = (
            reference.returncode == 0
            and rust.returncode == 0
            and reference.stdout == rust.stdout
        )
        print(f"{'PASS' if same else 'FAIL'} {fixture.name}")
        if not same:
            failures += 1
            print(f"  Python: exit={reference.returncode} stdout={reference.stdout!r} stderr={reference.stderr!r}")
            print(f"  Rust:   exit={rust.returncode} stdout={rust.stdout!r} stderr={rust.stderr!r}")
    if failures:
        print(f"{failures}/{len(cases)} differential fixtures disagree", file=sys.stderr)
        return 1
    print(f"Python/Rust runtime parity passed for {len(cases)} explicitly scoped fixtures.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
