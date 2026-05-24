# Axis Pass Pipeline and IR Strategy (Draft 0.1)

## 1. Decisions Captured

This document records pipeline decisions made after parser and initial type checker bring-up.

Confirmed decisions:
- Axis SHALL use a pass-manager architecture with explicit pass ordering.
- v0.1 SHALL prefer statically registered passes over dynamic runtime plugin loading.
- Compiler passes SHALL share unified diagnostics and pass-level logging through a single pass context.
- The runner SHALL support pass-profile configuration (`dev`, `test`, `release`) that controls pass selection.
- Axis SHALL use distinct IR stages: AST, HIR (resolved), TIR (typed), and later MIR/lowering IR.
- LLVM IR generation SHALL be performed after semantic passes and MIR-style lowering.

## 2. Pass Architecture

Core objects:
- `CompilerPass`: unit of work with deterministic input/output expectations.
- `PassManager`: ordered pass runner.
- `PipelineState`: evolving artifacts (source, AST, HIR, TIR, MIR, SSA scaffold).
- `PassContext`: cross-pass services (module root, diagnostics, pass logs).

v0.1 initial registered passes:
1. ParsePass
2. ResolvePass
3. TypeCheckPass
4. LowerPass
5. SsaPass (scaffolding)
6. BackendLlvmPass (scaffold artifact)

Pass logging contract:
- Every pass execution SHALL emit a pass-log entry with pass name and elapsed time.

## 3. IR Layer Clarification

AST:
- Parser-produced syntax tree.
- Source-structural representation.

HIR:
- Name-resolution-oriented representation.
- Symbol references and module ownership collected for semantic passes.

TIR:
- Type-annotated representation used by mutability, ownership, and numeric legality checks.

Planned next IR:
- MIR/lowering IR with explicit control flow and place assignments before SSA conversion.

SSA scaffolding IR:
- Versioned names derived from MIR places.
- Join markers translated into phi placeholders with incoming block metadata.
- Intended as the immediate boundary before full SSA rename/normalization and LLVM lowering.

Current note:
- v0.1 now emits concrete MIR with CFG-aware verification, place-based assignments, SSA scaffolding, and a backend LLVM scaffold artifact (`--llvm`); full SSA rename and real LLVM instruction lowering are the next milestones.

## 4. Planned LLVM Translation Sequence

Target sequence:
1. AST generation
2. HIR resolution
3. TIR type checking
4. Borrow and async legality validation
5. MIR/lowering transformation
6. SSA scaffolding and final SSA conversion
7. LLVM IR generation
8. Optimization passes
9. Emit stage

Rationale:
- Keeps LLVM generation focused on low-level code emission rather than high-level language semantics.

## 5. Optimization Pass Priorities (Initial)

Initial optimization ordering target:
1. Constant folding
2. Dead code elimination
3. CFG simplification
4. Optional small-function inlining

All optimizations SHALL preserve v0.1 safe semantics and deterministic diagnostics behavior.