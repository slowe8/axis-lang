# Axis LLVM Wrapper Strategy (Draft)

## Question: Should We Clone LLVM and Build a Wrapper?

Short answer: not yet.

Recommended path:
- Do not clone LLVM source into this repository as the default development path.
- Build a wrapper boundary in Axis first, then plug in one of:
- system LLVM installation via FFI (`llvm-sys` style)
- higher-level Rust binding layer (for faster iteration)
- optional external process mode (`llc`/`opt` toolchain) for early bring-up

## Why Not Clone LLVM First?

- Clone/build time and CI cost are high for early compiler iterations.
- It introduces large toolchain surface area before Axis SSA/backend contracts are stable.
- It can slow language-level progress while backend interfaces are still moving.

## Wrapper-First Benefits

- Axis backend code targets a stable internal adapter interface.
- We can test backend behavior with text/stub emitters before binding to real LLVM APIs.
- Binding choice can evolve without rewriting pass manager or SSA stages.

## Current Implementation Direction

Current backend shape in code:
- `LlvmBackendAdapter` trait defines backend emission boundary.
- `TextLlvmAdapter` is the initial implementation.
- `NativeLlvmAdapter` skeleton exists behind feature-aware behavior.
- `BackendLlvmPass` runs after SSA and can switch adapter implementations later.
- `lower_ssa_to_llvm_ir_with_preference` supports `Auto`/`Text`/`Native` selection.
- `Auto` currently reads `AXIS_LLVM_BACKEND` (set to `native` to opt in).
- Backend lowering is now fallible and enforces subset constraints instead of silently coercing unsupported SSA values.
- Requesting native mode without `llvm-native` feature now returns an explicit error.
- Current subset checks include rejecting void returns for i64 entry lowering and rejecting unsupported phi/assign/eval/branch payloads.
- Contract checking is extracted into a reusable module (`src/backend_contract.rs`) with table-driven tests across `SsaValue` variants per context.
- Phase-1 typed boundary is now in place: backend adapters accept `BackendInput { ssa, types }`, `backend_contract` receives `TypeStore`, and `BackendLlvmPass` requires persisted `backend_types` from type-check.
- Phase-2 typed SSA artifact is in place: `SsaTypeMap` is produced in SSA pass and threaded through backend input.
- First type-driven rule is active: branch conditions using SSA names now validate against mapped bool type and fail with a dedicated mismatch error.
- Type-driven consistency checks now also enforce known-type agreement for phi incoming values and assignments, and entry-return compatibility for known mapped types.
- Native path now emits through a dedicated module (`src/backend_native.rs`) with typed LLVM-like text for phi/assign/branch/return instead of reusing text-scaffold comments.
- Native wrapper now includes object emission support via `--emit-obj <path>` (feature `llvm-native`), invoking a locally built clang from the vendored LLVM checkout.
- LLVM is now tracked as a git submodule at `third_party/llvm-project`; bootstrap build script: `scripts/bootstrap_llvm.sh`.
- Default object-emission tool path is `third_party/llvm-build/bin/clang` (override with `AXIS_LLVM_CLANG`).

This allows introducing a real LLVM adapter behind the same trait.

## Planned Adapter Stack

1. `TextLlvmAdapter` (current)
- Purpose: deterministic snapshots and pass integration.
- Output: textual scaffold.

2. `NativeLlvmAdapter` (next)
- Purpose: real LLVM IR module construction.
- Output: verified LLVM module text/bitcode.
- Current status: skeleton with feature flag gate `llvm-native`.
- Dependency options (planned):
- dynamic link to installed LLVM
- containerized toolchain for CI reproducibility

3. `ToolchainAdapter` (optional)
- Purpose: invoke external LLVM tools for early assembly/object workflows.
- Output: object files and diagnostics from toolchain runs.

## Build and CI Strategy

Phase A:
- Keep repo dependencies small.
- Use text adapter by default.

Phase B:
- Add optional feature flag for native LLVM adapter (`--features llvm-native`).
- CI matrix includes one LLVM-enabled job; non-LLVM jobs remain fast.

Phase C:
- Promote native adapter as default once SSA and backend mapping stabilize.

## Immediate Next Steps

1. Keep strengthening SSA verifier until unresolved placeholders are eliminated for supported subsets.
2. Add a native adapter module skeleton behind feature-gated compilation.
3. Expand backend lowering coverage in the adapter boundary (constants, branches, returns, phi handling).
4. Add backend snapshot tests that remain adapter-agnostic where possible.
