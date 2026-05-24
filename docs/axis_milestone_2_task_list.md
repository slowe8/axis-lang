# Axis Milestone 2 Task List

## Purpose

This task list turns the module resolution milestone into executable work items.

Primary references:
- docs/axis_implementation_roadmap.md
- docs/axis_verification_test_plan.md
- docs/axis_architecture_spec.md

## Task List

### M2-T1: Formalize Module Graph Types

Goal:
- Define stable types for module paths, modules, and symbols.

Work items:
- Keep module hierarchy explicit.
- Keep symbol ownership tied to module paths.
- Preserve prelude storage separate from normal module storage.

Acceptance:
- Module graph types compile and remain stable for later resolver work.

### M2-T2: Implement Name Resolution Precedence

Goal:
- Resolve names in deterministic order.

Work items:
- Lexical scope.
- Current module scope.
- Explicit imports.
- Prelude.

Acceptance:
- Resolver chooses the expected binding for each lookup source.

### M2-T3: Enforce Visibility Rules

Goal:
- Preserve private-by-default semantics and explicit export boundaries.

Work items:
- Public symbols are importable.
- Private symbols remain hidden across module boundaries.

Acceptance:
- Resolver rejects private imports and accepts exported symbols.

### M2-T4: Add Module Graph Tests

Goal:
- Lock resolution semantics with test coverage.

Work items:
- Child module construction.
- Import resolution.
- Prelude fallback.
- Private import rejection.

Acceptance:
- Resolver tests cover the main lookup branches.

### M2-T5: Keep Resolver API Additive

Goal:
- Make future resolver expansion additive.

Work items:
- Avoid renaming the core graph and symbol types once published.
- Preserve compatibility with later AST/module-declaration expansion.

Acceptance:
- The resolver scaffold can be extended without breaking the current tests.

## Sequence

1. M2-T1
2. M2-T2
3. M2-T3
4. M2-T4
5. M2-T5

## Completion Criteria

- Module graph and resolver types exist.
- Lookup precedence is implemented and tested.
- Visibility rules are enforced.
- Milestone 3 type-system work can consume resolved symbols.
