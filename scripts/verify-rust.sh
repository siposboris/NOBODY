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
printf '%s\n' '== Nobody native Python-free CLI smoke =='
cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- info | grep -F '"implementation": "Rust"'
native_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run sk rust/examples/eval_basic_sk.nbd)
if [[ "$native_output" != "$expected" ]]; then
  printf 'Native Nobody output mismatch.\n' >&2
  exit 1
fi
printf '%s\n' '== Nobody Phase 4: verified in-memory bytecode VM smoke =='
vm_ai=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode ai rust/examples/eval_basic_ai.nbd)
vm_sk=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode sk rust/examples/eval_basic_sk.nbd)
if [[ "$vm_ai" != "$expected" || "$vm_sk" != "$expected" ]]; then
  printf 'Bytecode VM differs from existing Rust interpreter output.\n' >&2
  exit 1
fi
if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode ai rust/examples/bytecode_bad_unsupported_ai.nbd; then
  printf 'Bytecode VM unexpectedly accepted an unsupported control-flow statement.\n' >&2
  exit 1
fi
printf '%s\n' '== Nobody Phase 4: control-flow bytecode parity =='
for lang in ai sk en; do
  source="rust/examples/bytecode_control_${lang}.nbd"
  ast_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run "$lang" "$source")
  vm_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode "$lang" "$source")
  if [[ "$ast_output" != "$vm_output" || "$vm_output" != "$(printf '8\ntrue')" ]]; then
    printf 'VM control-flow parity failed: %s\n' "$source" >&2
    exit 1
  fi
done
if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode ai rust/examples/bytecode_infinite_ai.nbd; then
  printf 'VM did not terminate an infinite loop within its instruction budget.\n' >&2
  exit 1
fi
printf '%s\n' '== Nobody Phase 4: signed for-range bytecode parity =='
for lang in ai sk en; do
  source="rust/examples/bytecode_for_"$lang".nbd"
  tree_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run "$lang" "$source")
  vm_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode "$lang" "$source")
  if [[ "$tree_output" != "$vm_output" || "$vm_output" != "6" ]]; then
    printf 'For-range VM output mismatch: %s\n' "$source" >&2
    exit 1
  fi
done
if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode ai rust/examples/bytecode_for_bad_step_ai.nbd; then
  printf 'Zero-step range was accepted by bytecode VM.\n' >&2
  exit 1
fi
printf '%s\n' '== Nobody Phase 4: standalone bytecode function parity =='
for lang in ai sk en; do
  source="rust/examples/bytecode_functions_"$lang".nbd"
  tree_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run "$lang" "$source")
  vm_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode "$lang" "$source")
  if [[ "$tree_output" != "$vm_output" || "$vm_output" != "$(printf '13\n6')" ]]; then
    printf 'Function VM output mismatch: %s\n' "$source" >&2
    exit 1
  fi
done
if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode ai rust/examples/bytecode_function_bad_arity_ai.nbd; then
  printf 'Bytecode VM unexpectedly accepted wrong function arity.\n' >&2
  exit 1
fi
printf '%s\n' '== Nobody Phase 4: compiled portable bytecode smoke =='
bytecode_temp=$(mktemp -d)
trap 'rm -rf "$bytecode_temp"' EXIT
cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-bytecode ai rust/examples/bytecode_functions_ai.nbd "$bytecode_temp/program.nbc"
cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
  verify-bytecode "$bytecode_temp/program.nbc"
disk_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
  run-compiled "$bytecode_temp/program.nbc")
if [[ "$disk_output" != "$(printf '13\n6')" ]]; then
  printf 'Persisted bytecode VM output differed from original source.\n' >&2
  exit 1
fi
if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-bytecode ai rust/examples/bytecode_functions_ai.nbd "$bytecode_temp/program.nbc"; then
  printf 'Compiled bytecode unexpectedly overwrote an existing file.\n' >&2
  exit 1
fi
cp "$bytecode_temp/program.nbc" "$bytecode_temp/damaged.nbc"
printf 'extra' >> "$bytecode_temp/damaged.nbc"
if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
  run-compiled "$bytecode_temp/damaged.nbc"; then
  printf 'Persisted bytecode accepted trailing corruption.\n' >&2
  exit 1
fi
printf '%s\n' '== Nobody Phase 4: independent native Linux x86-64 ELF smoke =='
if [[ "$(uname -s)" == "Linux" && "$(uname -m)" == "x86_64" ]]; then
  for tool in as ld readelf; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      printf 'Native Linux backend prerequisite missing: %s (GNU binutils).\n' "$tool" >&2
      exit 2
    fi
  done
  native_temp=$(mktemp -d)
  trap 'rm -rf "$bytecode_temp" "$native_temp"' EXIT
  for language in ai sk en; do
    source="rust/examples/native_flow_"$language".nbd"
    executable="$native_temp/program-$language"
    cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
      compile-native "$language" "$source" "$executable"
    readelf -h "$executable" | grep -F 'ELF64'
    readelf -h "$executable" | grep -F 'Advanced Micro Devices X86-64'
    interpreted=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
      run "$language" "$source")
    native_output=$("$executable")
    if [[ "$interpreted" != "$native_output" || "$native_output" != "$(printf '8\n-42')" ]]; then
      printf 'Native ELF output does not match Rust evaluator for: %s\n' "$source" >&2
      exit 1
    fi
  done
  printf '%s\n' '== Nobody Phase 4: native signed for-range ELF parity =='
  for language in ai sk en; do
    source="rust/examples/bytecode_for_"$language".nbd"
    executable="$native_temp/range-$language"
    cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
      compile-native "$language" "$source" "$executable"
    tree_output=$(cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
      run "$language" "$source")
    native_output=$("$executable")
    if [[ "$tree_output" != "$native_output" || "$native_output" != "6" ]]; then
      printf 'Native signed range differs from interpreter: %s\n' "$source" >&2
      exit 1
    fi
  done
  cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
    compile-native ai rust/examples/native_range_zero_ai.nbd "$native_temp/range-zero"
  set +e
  "$native_temp/range-zero"
  range_zero_status=$?
  set -e
  if [[ "$range_zero_status" -ne 6 ]]; then
    printf 'Native zero-step range should exit with status 6, got: %s\n' "$range_zero_status" >&2
    exit 1
  fi
  cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
    compile-native ai rust/examples/native_overflow_ai.nbd "$native_temp/overflow"
  set +e
  "$native_temp/overflow"
  overflow_status=$?
  set -e
  if [[ "$overflow_status" -ne 3 ]]; then
    printf 'Native signed overflow did not exit with code 3.\n' >&2
    exit 1
  fi
  if cargo run --locked --quiet --manifest-path rust/Cargo.toml --bin nobody -- \
    compile-native ai rust/examples/bytecode_functions_ai.nbd "$native_temp/unsupported"; then
    printf 'Native backend unexpectedly accepted unsupported function syntax.\n' >&2
    exit 1
  fi
fi
printf '%s\n' 'Nobody Rust verification passed.'
