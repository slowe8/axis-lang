# Axis Open-Box Technical Requirements (Initial Translation)

## Scope

This document translates closed-box requirements into technical implementation requirements.
Each entry maps to closed-box requirement IDs and identifies enforcement layers.

Companion architecture mapping:
- docs/axis_architecture_spec.md

Requirement IDs:
- AXIS-OB-###

Enforcement layers:
- Parser
- Name Resolver
- Type Checker
- Borrow Checker
- Async Scheduler/Runtime
- Diagnostics
- Backend/Codegen

---

## 1. Ownership and Borrowing Subset

- AXIS-OB-001 (maps: AXIS-CB-001..006, 015): Type Checker and Borrow Checker SHALL reject use-after-move and invalid shared-borrow lifetime usage.
- AXIS-OB-002 (maps: AXIS-CB-003, 004, 044): Type Checker SHALL enforce read-only behavior for shared references and reject mutation through shared references.
- AXIS-OB-003 (maps: AXIS-CB-002, 005, 042): Type Checker SHALL model move semantics for non-trivial values and ownership transfer on assignment and argument passing.
- AXIS-OB-004 (maps: AXIS-CB-006): Diagnostics SHALL emit deterministic compile-time errors for ownership and borrowing violations.
- AXIS-OB-005 (maps: AXIS-CB-041..043): Type Checker SHALL enforce place mutability rules for reassignment and in-place field/index updates.

## 2. Numeric Semantics and Shape Rules

- AXIS-OB-010 (maps: AXIS-CB-020..027): Type Checker SHALL resolve numeric operators using explicit operand-shape rules and reject unsupported operator combinations.
- AXIS-OB-011 (maps: AXIS-CB-021, 022): Name Resolver and Type Checker SHALL bind dot and cross as explicit operations with dimensional constraints.
- AXIS-OB-012 (maps: AXIS-CB-023, 024): Type Checker SHALL enforce dimension compatibility for matrix-vector and matrix-matrix multiplication.
- AXIS-OB-013 (maps: AXIS-CB-025..027): Diagnostics SHALL provide explicit unsupported-operation errors for disallowed vector/matrix division and implicit broadcasting.

## 3. Matrix Layout and Interop

- AXIS-OB-020 (maps: AXIS-CB-030..033): Backend and standard numeric ABI layers SHALL encode matrix storage as column-major by default.
- AXIS-OB-021 (maps: AXIS-CB-032): Type Checker and runtime indexing helpers SHALL preserve logical row/column indexing independent of storage order.
- AXIS-OB-022 (maps: AXIS-CB-034, 035): Interop APIs SHALL require explicit conversion for row-major inputs and SHALL NOT apply implicit transpose.

## 4. Async and Structured Task Runtime

- AXIS-OB-030 (maps: AXIS-CB-060..064): Async Scheduler/Runtime SHALL represent task execution as explicit parent-child scopes with deterministic join/cancel semantics.
- AXIS-OB-031 (maps: AXIS-CB-065..067): Runtime and Type Checker SHALL support scope-level child-failure policy selection with fail-fast as default.
- AXIS-OB-032 (maps: AXIS-CB-068..071): Parser and Type Checker SHALL enforce await context legality and explicit task activation pathways.
- AXIS-OB-033 (maps: AXIS-CB-017): Type Checker SHALL reject borrowed captures that would cross await suspension in safe v0.1.

## 5. Error Propagation and Result/Option Handling

- AXIS-OB-040 (maps: AXIS-CB-080, 081): Type Checker SHALL enforce exact error-type compatibility for question-mark propagation in v0.1.
- AXIS-OB-041 (maps: AXIS-CB-082, 083): Type Checker SHALL constrain Option question-mark behavior to Option-returning contexts unless explicit adapters are used.
- AXIS-OB-042 (maps: AXIS-CB-013, 110..114): Name Resolver SHALL source Result/Option from prelude/library resolution rather than keyword treatment.

## 6. Modules and Visibility

- AXIS-OB-050 (maps: AXIS-CB-110, 111): Parser and Name Resolver SHALL construct module graphs from file hierarchy and explicit module/import declarations.
- AXIS-OB-051 (maps: AXIS-CB-112, 115): Name Resolver SHALL enforce private-by-default visibility and explicit public export boundaries.
- AXIS-OB-052 (maps: AXIS-CB-113, 114): Name Resolver SHALL apply deterministic lookup precedence across lexical, module, import, and prelude scopes.

## 7. Decorator Handling

- AXIS-OB-060 (maps: AXIS-CB-090..093): Parser and Backend SHALL treat advisory decorators as optimization intent and emit diagnostics on non-application.
- AXIS-OB-061 (maps: AXIS-CB-094, 122): Frontend SHALL gate trusted aliasing behind explicit trusted/unsafe boundaries outside safe v0.1.

## 8. Syntax Baseline Implementation

- AXIS-OB-070 (maps: AXIS-CB-100, 101): Parser SHALL implement block-expression tail-value semantics and statement-termination rules for baseline forms.
- AXIS-OB-071 (maps: AXIS-CB-102, 103, 104): Parser and Type Checker SHALL implement baseline literal, range, and match-pattern subsets.
- AXIS-OB-072 (maps: AXIS-CB-105): Parser evolution SHALL preserve parse meaning of valid baseline programs for additive syntax growth.

## 9. Deferred Technical Placeholders

- AXIS-OB-090 (maps: AXIS-CB-050..055): Arena technical implementation SHALL remain placeholder-gated until safe arena enablement milestone is approved.
- AXIS-OB-091 (maps: AXIS-CB-015, 016, 017, 045): Mutable borrowing, mutable arenas, cross-await borrowing, and interior mutability SHALL remain disabled in safe v0.1.
- AXIS-OB-092 (maps: AXIS-CB-018, 120..122): New features SHALL include compatibility validation proving additive behavior relative to baseline programs.

---

## Verification Hooks

- Each AXIS-OB requirement SHALL be linked to at least one compiler/runtime test category.
- Compile-time rejection requirements SHALL define canonical negative test cases.
- Behavioral requirements SHALL define deterministic expected outputs or runtime state assertions.

Primary mapping artifact:
- docs/axis_verification_test_plan.md
