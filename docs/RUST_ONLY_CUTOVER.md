# Direct Rust cutover — unverified candidate

The active Python implementation and Python test scripts have been removed from this copy. The original uploaded ZIP preserves them. Existing Rust implementations remain unchanged except for catalogue relocation and the regression harness. The 249-entry catalogue is now in `rust/data/`.

The 6 runtime and 13 AST reference outputs were captured from the original Python CLI before deletion. Rust CI compares against those frozen outputs without invoking Python. Existing Rust test suites remain.

This is not a complete feature port. Legacy closures and captured mutable state, first-class functions, list/index operations, floating-point semantics and some builtins and legacy test cases are not established as equivalent in Rust. Historical documentation may refer to removed paths. Do not treat this as proof of feature parity or production readiness. No roadmap completion claims were increased.

Validation: the original reference CLI successfully produced all 19 outputs. Rust compilation, formatting, Clippy and test execution were not possible in the editing environment (no Rust toolchain, network DNS unavailable). Run `bash scripts/verify-rust.sh` and both `parity-harness` modes via the Rust workflow before merging.
