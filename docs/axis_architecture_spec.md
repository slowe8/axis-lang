# Axis Architecture Specification (Draft 0.1)

## 1. Purpose

This document defines the v0.1 implementation architecture for Axis and explains how the architecture satisfies open-box requirements.

Primary inputs:
- axis_open_box_requirements.md
- axis_verification_test_plan.md

Primary outputs:
- Component boundaries
- Interface contracts
- Data flow
- Requirement satisfaction matrix

## 2. Architectural Principles

- Deterministic compile-time behavior for baseline semantics.
- Clear separation between parse, resolve, type/borrow analysis, and code generation.
- Explicit feature gating for deferred capabilities.
- Additive evolution without reinterpretation of valid baseline programs.

## 3. Top-Level Components

1. Frontend Lexer/Parser
- Responsibility: tokenize and parse source into AST according to v0.1 grammar.
- Owns: syntax validation, grammar rejection paths.
- Satisfies: AXIS-OB-070, AXIS-OB-071, AXIS-OB-072, AXIS-OB-032, AXIS-OB-050, AXIS-OB-060.

2. Module Graph and Name Resolver
- Responsibility: build module graph from files, resolve symbols, imports, visibility, and prelude.
- Owns: lexical/module/import/prelude lookup precedence.
- Satisfies: AXIS-OB-042, AXIS-OB-050, AXIS-OB-051, AXIS-OB-052.

3. Type Checker
- Responsibility: infer/check types, operator legality, mutability/place rules, question-mark semantics.
- Owns: numeric shape rules, Result/Option typing, await context typing.
- Satisfies: AXIS-OB-002, AXIS-OB-003, AXIS-OB-005, AXIS-OB-010, AXIS-OB-011, AXIS-OB-012, AXIS-OB-013, AXIS-OB-032, AXIS-OB-033, AXIS-OB-040, AXIS-OB-041, AXIS-OB-071.

4. Borrow Checker (v0.1 lexical model)
- Responsibility: ownership movement validity, shared-borrow lifetime checks, use-after-move rejection.
- Owns: lexical-lifetime analysis for safe v0.1 subset.
- Satisfies: AXIS-OB-001, AXIS-OB-004.

5. Async Structure Analyzer
- Responsibility: validate task tree constraints and scope policy declarations.
- Owns: explicit spawn/await structure legality and policy binding.
- Satisfies: AXIS-OB-030, AXIS-OB-031, AXIS-OB-032, AXIS-OB-033.

6. Runtime Task Scheduler
- Responsibility: structured task execution, join/cancel behavior, fail-fast and non-fail-fast policy realization.
- Owns: parent-child lifecycle and cooperative cancellation runtime behavior.
- Satisfies: AXIS-OB-030, AXIS-OB-031.

7. Numeric/Interop ABI Layer
- Responsibility: matrix storage layout, indexing translation utilities, interop conversion controls.
- Owns: column-major guarantees and explicit row-major conversion interfaces.
- Satisfies: AXIS-OB-020, AXIS-OB-021, AXIS-OB-022.

8. Decorator Planner
- Responsibility: advisory optimization intent routing to backend and diagnostics on unmet intent.
- Owns: trusted-surface gating hooks.
- Satisfies: AXIS-OB-060, AXIS-OB-061.

9. Backend/Codegen
- Responsibility: target code emission preserving frontend semantics.
- Owns: code emission and backend capability reporting for decorator planning.
- Satisfies: AXIS-OB-020, AXIS-OB-060.

10. Diagnostics Engine
- Responsibility: stable error categories and user-facing reporting.
- Owns: deterministic compile-time/runtime diagnostic normalization.
- Satisfies: AXIS-OB-004, AXIS-OB-013, AXIS-OB-060.

11. Feature Gate Manager
- Responsibility: mode and capability gating for deferred items.
- Owns: v0.1 baseline enablement and deferred feature rejection.
- Satisfies: AXIS-OB-090, AXIS-OB-091, AXIS-OB-092.

12. Conformance Harness
- Responsibility: execute AXIS-TV suites and publish pass/fail evidence.
- Owns: regression corpus and compatibility checks.
- Satisfies: architecture-level verification integration across all AXIS-OB IDs.

## 4. Core Data Model Contracts

AST:
- Parsed syntactic structure only.
- No resolved symbol bindings.

HIR (Resolved Intermediate Representation):
- Symbols resolved to canonical declarations.
- Visibility/import decisions finalized.

TIR (Typed Intermediate Representation):
- Types, mutability places, operator resolutions, and question-mark legality encoded.

BIR (Borrow-Checked Intermediate Representation):
- Ownership movement and borrow validity constraints validated.
- Rejected paths are diagnostics-only outputs.

AIR (Async Intermediate Representation):
- Task scope tree and policy descriptors normalized for runtime scheduling.

LIR/Backend IR:
- Lowered executable representation preserving TIR and AIR semantics.

## 5. Phase Pipeline

1. Parse
2. Resolve modules and names
3. Type check
4. Borrow check
5. Async structure analysis
6. Decorator planning
7. Lower and generate code
8. Emit diagnostics and artifacts

Pipeline invariants:
- A later phase must not reinterpret previously accepted semantics.
- Deferred features fail in feature-gate checks before semantic commitment.

## 6. Interface Contracts

Parser -> Resolver
- Input: source files
- Output: AST plus module declarations

Resolver -> Type Checker
- Input: resolved HIR
- Output: symbol-bound expressions and declarations

Type Checker -> Borrow Checker
- Input: TIR
- Output: ownership-sensitive typed operations

Borrow Checker -> Async Analyzer
- Input: ownership-valid BIR
- Output: borrow-safe async candidate IR

Async Analyzer -> Runtime Scheduler Metadata
- Input: AIR
- Output: task scope and policy metadata

Decorator Planner -> Backend
- Input: optimization intents and target capabilities
- Output: applied or unmet decorator decisions

All phases -> Diagnostics Engine
- Input: phase-local errors/warnings/info
- Output: normalized diagnostics

## 7. Requirement Satisfaction Matrix

| Open-Box Requirement | Primary Component Owner | Secondary Components | Verification Suites |
|---|---|---|---|
| AXIS-OB-001..005 | Borrow Checker, Type Checker | Diagnostics Engine | AXIS-TV-001 |
| AXIS-OB-010..013 | Type Checker | Diagnostics Engine | AXIS-TV-003 |
| AXIS-OB-020..022 | Numeric/Interop ABI Layer | Backend/Codegen | AXIS-TV-004 |
| AXIS-OB-030..033 | Async Structure Analyzer, Runtime Task Scheduler | Type Checker | AXIS-TV-005 |
| AXIS-OB-040..042 | Type Checker, Module Graph and Name Resolver | Diagnostics Engine | AXIS-TV-006 |
| AXIS-OB-050..052 | Module Graph and Name Resolver | Parser | AXIS-TV-009 |
| AXIS-OB-060..061 | Decorator Planner | Backend/Codegen, Diagnostics Engine | AXIS-TV-007 |
| AXIS-OB-070..072 | Frontend Lexer/Parser | Type Checker | AXIS-TV-008 |
| AXIS-OB-090..092 | Feature Gate Manager | All semantic phases | AXIS-TV-002, AXIS-TV-010, AXIS-TV-011 |

## 8. Architecture-to-Verification Mapping

- Parser-focused checks: AXIS-TV-008
- Resolver-focused checks: AXIS-TV-009
- Type/Borrow checks: AXIS-TV-001, AXIS-TV-003, AXIS-TV-006
- Async lifecycle checks: AXIS-TV-005
- Backend/interop checks: AXIS-TV-004, AXIS-TV-007
- Compatibility and gating checks: AXIS-TV-002, AXIS-TV-010, AXIS-TV-011

## 9. Deferred Capability Integration Strategy

Deferred capabilities are integrated through explicit extension points without changing baseline behavior:

- Mutable borrowing: extend Borrow Checker rules and TIR mutability forms.
- Safe arena enablement: add arena lifetime subsystem and promotion validator behind feature gates.
- Borrow-across-await: extend Async Analyzer and Borrow Checker cross-phase contract.
- Trusted surfaces: isolate in separate trusted compilation mode.

All deferred integrations must satisfy AXIS-OB-092 before baseline merge.

## 10. Architecture Exit Criteria

- Every AXIS-OB requirement has a primary component owner.
- Every AXIS-OB requirement maps to at least one AXIS-TV suite.
- Feature-gate boundaries are explicit for all deferred capabilities.
- No architecture interface implies semantic reinterpretation of valid v0.1 programs.

## 11. Implementation Roadmap Reference

- docs/axis_implementation_roadmap.md provides the execution sequence derived from this architecture.
- Milestone ordering in the roadmap must preserve the component dependency graph defined above.
