# Compiler milestone acceptance gates

This page tracks **verifiable partial progress**, not a new estimate of overall language completeness. The top-level roadmap count remains the only source for the README percentage. A gate passes only with a linked merged commit and a successful relevant CI run; do not infer completion from a drafted PR or a test file alone.

## Rust compiler and bytecode VM
- [ ] Confirm source-to-bytecode function recursion on main with a passing Rust integration test and linked CI run.
- [ ] Confirm source-to-bytecode forward calls and mutual recursion with verified diagnostics or execution tests.
- [ ] Confirm executable imported modules and consistent results across all supported syntax profiles.
- [ ] Confirm bytecode round-trip and execution parity for all documented supported constructs.

## Native Linux x86-64 ELF backend
- [ ] Confirm typed direct recursion in a standalone ELF with factorial and bounded failure tests on main.
- [ ] Confirm branching recursion (e.g. Fibonacci) preserves intermediate values in a standalone ELF, with CI.
- [ ] Confirm forward function calls and mutual recursion, or document precise unsupported diagnostics and acceptance gaps.
- [ ] Confirm all currently documented supported native constructs via real ELF execution and evaluator comparison.

## Python to Rust migration
- [ ] Inventory every remaining Python command in regular build, test and release workflows.
- [ ] Replace routine Python-based verification with Rust commands, with equivalent failure cases.
- [ ] Verify legacy Python frontend fixture parity with Rust equivalents before removing each dependency.
- [ ] Run the supported default build, test, and release validation paths without Python.

## Self-hosting
- [ ] Compile a nontrivial Nobody-authored compiler component with the Rust bootstrap compiler.
- [ ] Execute that component and compare outputs against the Rust implementation.
- [ ] Bootstrap the compiler from Nobody sources and reproduce equivalent artifacts.
- [ ] Verify repeatable bootstrap in CI without requiring Python or a Rust compiler for the final stage.

**How to report:** For each section, report `verified gates / total gates` and cite the exact merged commit plus CI run. Unknown gates remain unchecked. Do not convert these counts to an overall project percentage. Update this file only after evidence is available; avoid claiming that tests already merged are unverified without checking their CI status.
