# Axis LLVM Integration Plan (Draft 0.1)

## Purpose

This document captures the staged integration strategy from current MIR/SSA scaffolding to executable LLVM IR generation.

## Current Baseline

Implemented today:
- Pass-managed pipeline with Parse -> Resolve -> TypeCheck -> Lower -> Ssa.
- MIR with explicit blocks, branch/goto/return terminators, place-based assignments, and join markers.
- CFG analysis utilities (predecessors, successors, reverse post-order, dominator sets).
- SSA scaffolding pass that versions places and materializes phi placeholders.

Not implemented yet:
- Full SSA rename and use-rewrite across CFG joins.
- Concrete LLVM backend module/function/type/value emission.

## Integration Principles

- LLVM lowering SHALL consume finalized SSA form, not pre-SSA MIR.
- Backend SHALL not perform language-level semantics checks that belong in earlier passes.
- Diagnostics SHALL remain deterministic and stable across profiles.
- Initial backend SHALL target correctness-first IR, with optimization quality deferred.

## Staged Plan

1. Finalize SSA conversion
- Replace scaffolding placeholders with fully linked phi inputs by SSA name.
- Ensure all value uses in statements and terminators are SSA names or constants.
- Add SSA verifier checks (dominance, definition-before-use, phi predecessor coverage).

2. Define LLVM backend contract
- Introduce backend input contract over finalized SSA program.
- Define mapping for:
- scalar primitives (`int`, `float`, `bool`, `char`)
- unit and never handling
- function signatures and calling convention assumptions
- control-flow blocks and terminator mapping

3. Add backend pass shell
- Add `BackendLlvmPass` after SSA conversion in pipeline config.
- Produce an in-memory LLVM module representation (or text IR string if bridge not yet linked).
- Emit backend diagnostics into shared `PassContext`.

4. Implement minimum viable codegen
- Function declaration + definition emission.
- Basic blocks and branches from SSA CFG.
- Return lowering for scalar and unit cases.
- Integer/boolean constant materialization.

5. Expand value and operation coverage
- Arithmetic and comparison operations.
- Phi lowering from SSA merge nodes.
- Calls and argument passing.
- Memory model for locals if required by non-SSA operations.

6. Introduce backend verification and snapshots
- Run LLVM verifier per-module/per-function.
- Add textual IR snapshot tests for canonical control-flow shapes.
- Add end-to-end compile smoke tests from Axis source to LLVM IR output.

## Pass Pipeline Target (Near-Term)

1. ParsePass
2. ResolvePass
3. TypeCheckPass
4. LowerPass
5. SsaPass (scaffold -> final SSA)
6. BackendLlvmPass

## Key Technical Risks

- Incomplete SSA use rewriting may force backend-side patch logic (must avoid).
- Type representation drift between TIR and backend may create unstable IR signatures.
- Phi placement mistakes in loops can produce verifier failures or miscompiles.

## Immediate Next Tasks

1. Upgrade SSA scaffold to finalized SSA names in terminators and expression uses.
2. Add SSA verifier pass with dominance and phi coverage checks.
3. Create backend module skeleton (`src/backend/llvm.rs`) and backend pass placeholder.
4. Add `--llvm` CLI dump flag once backend output artifact exists.
