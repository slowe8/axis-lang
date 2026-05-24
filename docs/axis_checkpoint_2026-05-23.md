# Axis Checkpoint - 2026-05-23

## Scope of Checkpoint

This checkpoint summarizes progress through the initial pass-managed compiler pipeline bring-up and the first concrete MIR lowerer iteration.

## Completed Progress

1. Pass architecture foundation
- Pass runner is in place with explicit phase ordering and shared context.
- Pass profiles are supported (`dev`, `test`, `release`) and wired through CLI.
- Pipeline state includes AST, HIR, TIR, and MIR artifacts.

2. HIR progression
- HIR now carries stable `SymbolId` values.
- Name-use tracking includes resolution scope and optional symbol linkage.
- Lexical bindings (function parameters, local lets, loop bindings) are promoted into first-class HIR symbol entries.

3. TIR progression
- Type checker can run from HIR input (`check_hir`).
- Typed expressions now carry optional symbol linkage (`symbol_id`) sourced from HIR name-use records.

4. MIR progression
- MIR is no longer summary-only.
- Lowered artifacts now include basic blocks, statements, terminators, temporary tracking, and value forms.
- Lowering supports concrete paths for literals, identifiers/paths, blocks, calls, tuples, unary/binary/range scaffolding, and `if` control-flow branching.

5. Runner and diagnostics visibility
- CLI can dump pass logs and stage artifacts (`--passes`, `--hir`, `--typed`, `--mir`).
- Diagnostics continue to flow through shared pass context.

6. Validation status
- Unit/integration suite currently green (45 tests passing at this checkpoint).

## Major Backlog Items (Next)

1. MIR control-flow completeness
- Replace placeholder lowering for `while`, `for`, and `match` with stable block/edge construction.
- Introduce explicit block cursor model so branch-producing expressions do not overwrite active block terminators unsafely.

2. Declaration-site symbol identity
- Thread declaration `SymbolId` onto typed declarations (`let`/params) so MIR local stores target declaration IDs rather than inferred use IDs.

3. Deterministic HIR-to-TIR symbol mapping
- Replace name-sequence consumption with explicit expression node IDs to avoid order sensitivity.

4. MIR SSA readiness
- Add local/place abstraction and assign semantics required for SSA or SSA-like lowering to LLVM IR.

5. Type-checker semantic depth
- Expand TIR legality for numeric shape rules, `?` propagation, and Option/Result exactness with deterministic diagnostics.

6. Borrow and async passes
- Start borrow-check pass contract over TIR/MIR symbols.
- Add async structure legality pass before backend lowering.

7. Backend planning
- Define LLVM-lowering contract from MIR (control-flow, values, calls, symbols, and type mapping).
- Specify optimization pass gates and ordering constraints in pass profile config.

## Immediate Work Queue

1. Add declaration-side symbol IDs to typed lets/params so MIR StoreLocal targets declaration identity directly.
2. Introduce an explicit MIR block cursor model so nested control-flow lowering does not overwrite active terminators.
3. Extend MIR coverage from scaffolding to normalized CFG suitable for SSA preparation.