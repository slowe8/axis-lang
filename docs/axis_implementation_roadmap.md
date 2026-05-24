# Axis Implementation Roadmap (Draft 0.1)

## 1. Purpose

This roadmap turns the architecture, closed-box requirements, open-box requirements, and verification plan into an implementation sequence.

Primary references:
- docs/axis_closed_box_requirements.md
- docs/axis_open_box_requirements.md
- docs/axis_architecture_spec.md
- docs/axis_verification_test_plan.md

## 2. Roadmap Principles

- Build by dependency order.
- Establish verification gates as early as possible.
- Keep deferred capabilities disabled until their gates are explicitly approved.
- Preserve valid v0.1 behavior across all later milestones.

Cross-cutting architecture decision:
- The compiler SHALL execute through an explicit pass manager with shared diagnostics and pass logging.
- IR progression SHALL be AST -> HIR -> TIR -> MIR/lowering IR -> LLVM IR.

## 3. Milestone Sequence

### Milestone 0: Project Skeleton and Tooling

Goal:
- Establish repository structure, build entry points, test harness layout, and diagnostic conventions.

Primary work:
- Create compiler component directories and module boundaries.
- Set up baseline parser/type-checker test harnesses.
- Establish requirement ID, test ID, and diagnostic ID conventions.

Owners:
- Conformance Harness
- Diagnostics Engine

Dependencies:
- None.

Verification suites:
- AXIS-TV-010
- AXIS-TV-011

Exit criteria:
- Repository has stable build/test entry points.
- Baseline conformance and gating scaffolding exists.

### Milestone 1: Parser and Grammar Baseline

Goal:
- Parse v0.1 grammar and reject unsupported syntax deterministically.

Primary work:
- Lexer/tokenizer.
- AST construction.
- Tail-expression and statement-termination grammar.
- Literal/range/match baseline parsing.

Owners:
- Frontend Lexer/Parser
- Diagnostics Engine

Dependencies:
- Milestone 0.

Verification suites:
- AXIS-TV-008

Exit criteria:
- Valid baseline programs parse.
- Invalid grammar is rejected with stable diagnostics.

Milestone 1 task list reference:
- docs/axis_milestone_1_task_list.md

### Milestone 2: Module Resolution and Visibility

Goal:
- Resolve files, modules, imports, visibility, and prelude names.

Primary work:
- Build module graph.
- Implement lexical/module/import/prelude lookup order.
- Enforce private-by-default visibility.

Owners:
- Module Graph and Name Resolver

Dependencies:
- Milestone 1.

Verification suites:
- AXIS-TV-009

Exit criteria:
- Multi-module packages resolve deterministically.
- Visibility boundaries are enforced.

Milestone 2 task list reference:
- docs/axis_milestone_2_task_list.md

### Milestone 3: Type System Core

Goal:
- Enforce v0.1 type rules for numerics, mutability, Result/Option, and question-mark semantics.

Primary work:
- Type inference and checking.
- Numeric operator resolution and dimensional checks.
- Mutability/place rules.
- Result/Option resolution and `?` typing.

Owners:
- Type Checker
- Diagnostics Engine

Dependencies:
- Milestones 1 and 2.

Verification suites:
- AXIS-TV-003
- AXIS-TV-006

Exit criteria:
- Numeric, mutability, and error-propagation rules are enforced.
- Rejection diagnostics are deterministic.

### Milestone 4: Ownership and Borrowing Core

Goal:
- Enforce move semantics, shared borrowing, and lexical lifetime checks.

Primary work:
- Use-after-move rejection.
- Shared reference read-only enforcement.
- Lexical borrow validation.

Owners:
- Borrow Checker
- Type Checker

Dependencies:
- Milestones 1 through 3.

Verification suites:
- AXIS-TV-001

Exit criteria:
- Safe ownership subset is enforced.
- Mutable borrowing remains disabled.

### Milestone 5: Numeric ABI and Interop

Goal:
- Lock matrix layout and interop behavior.

Primary work:
- Column-major representation.
- Logical row/column indexing support.
- Explicit row-major conversion interfaces.

Owners:
- Numeric/Interop ABI Layer
- Backend/Codegen

Dependencies:
- Milestone 3.

Verification suites:
- AXIS-TV-004

Exit criteria:
- Layout and interop behavior match closed-box requirements.
- No implicit transpose at boundaries.

### Milestone 6: Structured Async Core

Goal:
- Implement structured task execution and await legality.

Primary work:
- Parent-child task tree validation.
- Scope-level failure policy.
- Await context legality.
- Owned-capture-only enforcement.

Owners:
- Async Structure Analyzer
- Runtime Task Scheduler
- Type Checker

Dependencies:
- Milestones 1, 3, and 4.

Verification suites:
- AXIS-TV-005

Exit criteria:
- Join/cancel semantics are deterministic.
- Fail-fast and non-fail-fast policies behave as specified.

### Milestone 7: Decorator and Trusted Surface Routing

Goal:
- Add advisory decorator handling and trusted-boundary gating.

Primary work:
- Advisory decorator diagnostics.
- Strict-performance mode hooks.
- Trusted aliasing gating.

Owners:
- Decorator Planner
- Backend/Codegen
- Diagnostics Engine

Dependencies:
- Milestone 3.

Verification suites:
- AXIS-TV-007

Exit criteria:
- Advisory decorators never alter safe semantics.
- Trusted surfaces remain outside safe v0.1.

### Milestone 8: Compatibility and Deferred Gates

Goal:
- Prove additive evolution behavior and keep deferred features disabled.

Primary work:
- Feature-gate manager.
- Golden corpus regression checks.
- Deferred-feature rejection paths.

Owners:
- Feature Gate Manager
- Conformance Harness
- Diagnostics Engine

Dependencies:
- Milestones 1 through 7.

Verification suites:
- AXIS-TV-002
- AXIS-TV-010
- AXIS-TV-011

Exit criteria:
- Deferred features remain disabled in safe v0.1.
- Baseline behavior remains stable across regression runs.

## 4. Architecture Delivery Sequence

1. Parser, diagnostics, and test harness foundation.
2. Resolver and module graph.
3. Type checker and mutability/numeric rules.
4. Borrow checker and ownership enforcement.
5. Async analyzer/runtime.
6. Interop and backend capability routing.
7. Decorator/trusted-surface gating.
8. Compatibility and regression hardening.

## 5. Requirement Coverage Summary

- Baseline closed-box requirements are covered by Milestones 1 through 8.
- Open-box technical requirements are covered by the same milestones with explicit owners.
- Deferred requirements are represented as gated milestones rather than active baseline work.

## 6. Implementation Readiness Gates

- No milestone may begin without the preceding milestone's verification suite definitions.
- No deferred feature may be implemented without explicit gate criteria.
- Any architecture change must preserve the requirement satisfaction matrix.

## 7. Milestone 0 Task List Reference

- docs/axis_milestone_0_task_list.md contains the concrete task breakdown for Milestone 0.
- Milestone 0 tasks should be completed before Milestone 1 parser work begins.
