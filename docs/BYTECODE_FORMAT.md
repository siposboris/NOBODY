# Nobody experimental binary bytecode v1 (.nbc)

**Status:** experimental platform-neutral wire format for the supported Rust VM subset. Not a cross-version compatibility promise, native machine-code compiler, or self-hosting.

## CLI usage

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-bytecode sk rust/examples/bytecode_functions_sk.nbd program.nbc
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  verify-bytecode program.nbc
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  run-compiled program.nbc
```

The compiler creates a new `.nbc` file and refuses to overwrite existing paths. The verification command validates the file without running it. The execution command needs neither the original `.nbd` source, language-profile setting, nor Python.

## Binary layout

All multi-byte fields are little-endian. Counts, slots, indexes and jump targets are unsigned 32-bit integers. Numeric literals are signed 128-bit two's-complement values. No Rust struct layouts or native addresses are serialized.

| Field | Bytes | Meaning |
|---|---:|---|
| Magic | 8 | ASCII `NOBBC001` |
| Format version | 2 | u16, exactly 1 |
| Reserved flags | 2 | u16, exactly 0 |
| Root slot count | 4 | u32 |
| Root instruction count | 4 | u32 |
| Function count | 4 | u32 |
| Root instructions | variable | exactly the recorded instruction count |
| Function entries | variable | exactly the recorded function count |
| Checksum | 8 | u64 FNV-1a of all preceding bytes |

Each function entry consists of a u16 UTF-8 function-name byte length, that many name bytes, a u32 parameter count, u32 slot count, u32 instruction count and the corresponding instruction stream. The first parameter-count slots of a function are initialized by the caller.

Every instruction starts with one opcode byte, optionally followed by fixed-size operands.

| Opcode | Operation | Operand bytes |
|---:|---|---|
| 0 | Push | i128 (16) |
| 1 | PushBool | u8 (must be 0 or 1) |
| 2 | PushNull | none |
| 3 | Load | slot u32 |
| 4 | Define | slot u32 |
| 5 | Store | slot u32 |
| 6–16 | Add, Sub, Mul, Neg, Not, Eq, Ne, Lt, Le, Gt, Ge | none |
| 17 | Jump | target u32 |
| 18 | JumpIfFalse | target u32 |
| 19 | RangeCheck | current, stop, step, exit: four u32 |
| 20 | RangeAdvance | current, step, exit: three u32 |
| 21 | Drop | none |
| 22 | Print | argument count u32 |
| 23 | Call | function ID u32, argument count u32 |
| 24 | Return | none |
| 25 | Halt | none |

## Limits and validation

- 2 MiB maximum total file size, including checksum.
- Up to 1,024 function entries, each with at most 10,000 instructions.
- Up to 1,024 local variable slots per unit; 1–256 UTF-8 bytes per function name.
- Unknown versions/flags/opcodes, noncanonical booleans, truncation, trailing data, invalid jumps and damaged checksums are rejected.
- The existing control-flow verifier checks stack shape, definite slot initialization, call arity, return discipline and legal termination in each unit.
- No fallback to source interpretation or Python when decoding fails.

**Security:** FNV-1a is a noncryptographic checksum for accidental damage, not authentication or a signature; it can be recomputed trivially. A passing verifier is not an operating-system sandbox. Do not execute untrusted bytecode without external isolation.

The format is deterministic: equivalent supported Nobody source programs in Slovak, English and `ai` can generate identical bytes. Bytecode has no embedded source or language profile. It remains an *experimental subset*: closures, recursion, all types, stable cross-version ABI, native machine code and self-hosting are still missing.
