#!/usr/bin/env bash
# Deterministic, read-only verification entry point for the Rust migration.
# No dependency installation, source rewriting or Python fallback.
set -Eeuo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if ! command -v cargo >/dev/null 2>&1; then
  printf 'Verification prerequisite missing: cargo is not installed. Install the Rust toolchain and rerun this script.\n' >&2
  exit 2
fi
if ! command -v rustfmt >/dev/null 2>&1; then
  printf 'Verification prerequisite missing: rustfmt is missing. Install the rustfmt component and rerun.\n' >&2
  exit 2
fi
if ! command -v cargo-clippy >/dev/null 2>&1; then
  printf 'Verification prerequisite missing: Clippy is missing. Install the clippy component and rerun.\n' >&2
  exit 2
fi
printf '%s\n' '== Nobody Rust: locked dependency resolution =='
cargo metadata --locked --manifest-path rust/Cargo.toml --format-version 1 --no-deps > /dev/null
printf '%s\n' '== Nobody Rust: format check =='
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
printf '%s\n' '== Nobody Rust: compile and test =='
cargo test --locked --manifest-path rust/Cargo.toml --all-targets
printf '%s\n' '== Nobody Rust: strict Clippy =='
cargo clippy --locked --manifest-path rust/Cargo.toml --all-targets -- -D warnings
printf '%s\n' '== Nobody Rust: CLI smoke =='
cargo run --locked --quiet --manifest-path rust/Cargo.toml -- typecheck ai rust/examples/typecheck_good_ai.nbd
cargo run --locked --quiet --manifest-path rust/Cargo.toml -- typecheck ai rust/examples/arity_good_ai.nbd
for bad in rust/examples/typecheck_bad_ai.nbd rust/examples/arity_bad_ai.nbd; do
  if cargo run --locked --quiet --manifest-path rust/Cargo.toml -- typecheck ai "$bad"; then
    printf 'Verification failed: expected typecheck failure for %s but command succeeded.\n' "$bad" >&2
    exit 1
  fi
done
printf '%s\n' 'Nobody Rust verification passed.'
