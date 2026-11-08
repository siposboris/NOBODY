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
printf '%s\n' '== Nobody Rust: opt-in evaluator smoke =='
ai_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml -- run ai rust/examples/eval_basic_ai.nbd)
sk_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml -- run sk rust/examples/eval_basic_sk.nbd)
expected=$(printf '9\n8')
if [[ "$ai_output" != "$expected" || "$sk_output" != "$expected" ]]; then
  printf 'Evaluator smoke output mismatch.\n' >&2
  exit 1
fi
if cargo run --locked --quiet --manifest-path rust/Cargo.toml -- run ai rust/examples/eval_bad_zero_ai.nbd; then
  printf 'Evaluator accepted a zero divisor unexpectedly.\n' >&2
  exit 1
fi
printf '%s\\n' '== Nobody native Python-free CLI smoke =='
cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- info | grep -F '"implementation": "Rust"'
native_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run sk rust/examples/eval_basic_sk.nbd)
if [[ "$native_output" != "$expected" ]]; then
  printf 'Native Nobody output mismatch.\\n' >&2
  exit 1
fi
printf '%s\n' 'Nobody Rust verification passed.'
