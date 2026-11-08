# Nobody — feature status

**Prototype** means implemented in earlier local bundles but not yet re-audited on GitHub. **Planned** means not implemented. A configuration switch is *not* proof a feature works.

| Capability | Status | Notes |
|---|---|---|
| Basic SK/EN/CS/DE syntax | Prototype | One language per file |
| Lowercase ASCII keyword/identifier policy | Prototype | Strings/comments may use Unicode |
| Variables, conditions, loops, functions | Prototype | Limited core |
| Nobody → Nobody source translation | Prototype | .nbd output |
| Nobody-written compiler core | Prototype | Python bootstrap still required |
| Basic inferred/explicit type checks | Prototype | Limited coverage |
| Restricted immutable-only mode | Prototype | Limited constructs |
| Catalog of 22 option families | Prototype | Many unimplemented |
| Formal parser, AST/IR, modules, generics | Planned | |
| Rust-style ownership, GC and manual memory profiles | Planned | |
| Native x86-64/ARM64 and WASM backends | Planned | |
| GPU/tensors, scientific math, SIMD | Planned | |
| Async, threads, actors, distribution | Planned | |
| Hygienic macros and formal proof modes | Planned | |
| LSP, debugger, package manager, reproducible builds | Planned | |
| Native self-hosted toolchain without Python | Planned | |
| Hard resource isolation and security sandbox | Planned | |

See [full roadmap](ROADMAP.md).


## Experimental 0.8 progress

A local prototype now adds strict short foreign-keyword checks and validates both ends of Nobody-to-Nobody source translation. 49 tests passed locally, but the complete code has not yet been imported here. See [0.8 notes](PROTOTYPE_0_8.md). This does not change the status of native compilation, runtime, memory safety or full self-hosting.
