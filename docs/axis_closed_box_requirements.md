# Axis Closed-Box Product Requirements (Draft 0.1)

## Scope

This document defines non-technical, externally testable product requirements for Axis.
Requirements use SHALL and SHALL NOT statements.

Requirement IDs:
- AXIS-CB-###

Scope tags:
- v0.1 Baseline
- Deferred
- Additive Future

---

## 1. Safety and Correctness

- AXIS-CB-001 (v0.1 Baseline): Safe Axis programs SHALL prevent use-after-free behavior.
- AXIS-CB-002 (v0.1 Baseline): Safe Axis programs SHALL reject use of moved non-trivial values.
- AXIS-CB-003 (v0.1 Baseline): Safe Axis programs SHALL allow shared read-only references.
- AXIS-CB-004 (v0.1 Baseline): Safe Axis programs SHALL NOT allow mutable references.
- AXIS-CB-005 (v0.1 Baseline): Safe Axis programs SHALL enforce explicit ownership transfer for non-trivial values.
- AXIS-CB-006 (v0.1 Baseline): Safe Axis programs SHALL enforce compile-time errors for invalid ownership and borrowing operations.

## 2. Feature Availability and Scope

- AXIS-CB-010 (v0.1 Baseline): Axis SHALL provide immutable and mutable bindings.
- AXIS-CB-011 (v0.1 Baseline): Axis SHALL provide synchronous functions.
- AXIS-CB-012 (v0.1 Baseline): Axis SHALL provide task-based asynchronous functions under structured execution rules.
- AXIS-CB-013 (v0.1 Baseline): Axis SHALL provide Result and Option as prelude-available standard types.
- AXIS-CB-014 (v0.1 Baseline): Axis SHALL provide fixed-size vector and matrix numeric types.
- AXIS-CB-015 (Deferred): Axis SHALL NOT expose safe mutable borrowing in v0.1.
- AXIS-CB-016 (Deferred): Axis SHALL NOT expose safe mutable arena references in v0.1.
- AXIS-CB-017 (Deferred): Axis SHALL NOT expose borrowed captures across await suspension in v0.1.
- AXIS-CB-018 (Additive Future): Deferred features SHALL be introduced additively without changing the meaning of valid v0.1 programs.

## 3. Observable Numeric Behavior

- AXIS-CB-020 (v0.1 Baseline): Vector-vector multiplication SHALL be elementwise.
- AXIS-CB-021 (v0.1 Baseline): Dot product SHALL be expressed via an explicit dot operation.
- AXIS-CB-022 (v0.1 Baseline): Cross product SHALL be expressed via an explicit cross operation and SHALL be limited to 3D vectors.
- AXIS-CB-023 (v0.1 Baseline): Matrix-vector and matrix-matrix multiplication SHALL follow standard linear algebra semantics.
- AXIS-CB-024 (v0.1 Baseline): Scalar scaling of vectors and matrices SHALL be supported.
- AXIS-CB-025 (v0.1 Baseline): Vector-vector division SHALL NOT be defined.
- AXIS-CB-026 (v0.1 Baseline): Matrix-matrix division SHALL NOT be defined.
- AXIS-CB-027 (v0.1 Baseline): Implicit broadcasting beyond scalar-to-vector and scalar-to-matrix scaling SHALL NOT be defined.

## 4. Matrix Layout and Interop Behavior

- AXIS-CB-030 (v0.1 Baseline): Axis matrix storage SHALL be column-major by default.
- AXIS-CB-031 (v0.1 Baseline): Axis vector interpretation in matrix math SHALL use column-vector convention.
- AXIS-CB-032 (v0.1 Baseline): Matrix indexing SHALL be logical row/column indexing independent of storage order.
- AXIS-CB-033 (v0.1 Baseline): Default numeric API interop SHALL assume column-major buffers.
- AXIS-CB-034 (v0.1 Baseline): Row-major interop SHALL require explicit conversion.
- AXIS-CB-035 (v0.1 Baseline): API and FFI boundaries SHALL NOT apply implicit transpose.

## 5. Mutability and State Update Behavior

- AXIS-CB-040 (v0.1 Baseline): Mutability SHALL be controlled by binding and place rules.
- AXIS-CB-041 (v0.1 Baseline): Immutable bindings SHALL NOT be reassignable.
- AXIS-CB-042 (v0.1 Baseline): Mutable bindings SHALL be reassignable subject to ownership rules.
- AXIS-CB-043 (v0.1 Baseline): In-place field and index mutation SHALL require a mutable owned root place.
- AXIS-CB-044 (v0.1 Baseline): Mutation through shared references SHALL NOT be allowed.
- AXIS-CB-045 (Deferred): Interior mutability SHALL NOT be part of safe v0.1.

## 6. Arena Lifecycle and Promotion Behavior

- AXIS-CB-050 (Deferred): Arena allocations SHALL use explicit region-bound reference semantics when enabled.
- AXIS-CB-051 (Deferred): Arena-backed values SHALL NOT escape their source arena without explicit promotion.
- AXIS-CB-052 (Deferred): Arena promotion SHALL be explicit.
- AXIS-CB-053 (Deferred): Arena promotion SHALL be deep in first safe arena release.
- AXIS-CB-054 (Deferred): Arena promotion SHALL be rejected when safe lifetime rebinding cannot be proven.
- AXIS-CB-055 (Deferred): Arena scope exit SHALL reclaim arena memory in bulk.

## 7. Async and Structured Task Behavior

- AXIS-CB-060 (v0.1 Baseline): Task execution SHALL be structured as a parent-child tree.
- AXIS-CB-061 (v0.1 Baseline): Child tasks SHALL NOT outlive the parent scope that spawned them.
- AXIS-CB-062 (v0.1 Baseline): Child spawning SHALL be explicit and scoped.
- AXIS-CB-063 (v0.1 Baseline): Parent scope completion SHALL deterministically observe child completion through join or configured cancellation.
- AXIS-CB-064 (v0.1 Baseline): Parent early-exit failure SHALL trigger cooperative cancellation of remaining child tasks.
- AXIS-CB-065 (v0.1 Baseline): Child failure policy SHALL be configurable per scope.
- AXIS-CB-066 (v0.1 Baseline): Default child failure policy SHALL be fail-fast.
- AXIS-CB-067 (v0.1 Baseline): Non-fail-fast policy SHALL permit sibling completion and join before parent continuation.
- AXIS-CB-068 (v0.1 Baseline): Await SHALL be an expression in task contexts.
- AXIS-CB-069 (v0.1 Baseline): Await in synchronous functions SHALL be rejected.
- AXIS-CB-070 (v0.1 Baseline): Task execution SHALL begin only through explicit await or explicit scoped spawn.
- AXIS-CB-071 (v0.1 Baseline): Detached background task execution SHALL NOT occur implicitly in safe v0.1.

## 8. Error Handling Behavior

- AXIS-CB-080 (v0.1 Baseline): Result propagation with question-mark SHALL require exact error-type matching.
- AXIS-CB-081 (v0.1 Baseline): Question-mark SHALL NOT perform implicit error conversion in v0.1.
- AXIS-CB-082 (v0.1 Baseline): Option question-mark usage SHALL be limited to Option-returning contexts.
- AXIS-CB-083 (v0.1 Baseline): Option-to-Result lifting SHALL require explicit adapter operations.

## 9. Decorator and Trusted Surface Behavior

- AXIS-CB-090 (v0.1 Baseline): Performance decorators SHALL be advisory by default.
- AXIS-CB-091 (v0.1 Baseline): Advisory decorators SHALL NOT change observable safe-code semantics.
- AXIS-CB-092 (v0.1 Baseline): Unsupported advisory decorator requests SHALL produce explicit diagnostics.
- AXIS-CB-093 (v0.1 Baseline): Strict performance mode MAY elevate unmet advisory requests to compile errors.
- AXIS-CB-094 (Deferred): Trusted aliasing SHALL remain outside safe v0.1.

## 10. Syntax and Language Surface Behavior

- AXIS-CB-100 (v0.1 Baseline): Blocks SHALL evaluate to the final expression value when no trailing statement terminator is present.
- AXIS-CB-101 (v0.1 Baseline): Statement forms requiring termination SHALL require statement terminators.
- AXIS-CB-102 (v0.1 Baseline): v0.1 literal support SHALL include decimal integers, decimal floats, booleans, strings, and chars.
- AXIS-CB-103 (v0.1 Baseline): v0.1 range support SHALL include half-open range form only.
- AXIS-CB-104 (v0.1 Baseline): v0.1 match pattern support SHALL include literals, wildcard, and simple enum payload binding.
- AXIS-CB-105 (Additive Future): Future syntax expansion SHALL be additive and SHALL NOT reinterpret valid v0.1 programs.

## 11. Modules, Visibility, and Name Resolution

- AXIS-CB-110 (v0.1 Baseline): File and directory structure SHALL define module hierarchy.
- AXIS-CB-111 (v0.1 Baseline): Module and import declarations SHALL support explicit namespace construction and use.
- AXIS-CB-112 (v0.1 Baseline): Visibility SHALL be private by default unless explicitly exported.
- AXIS-CB-113 (v0.1 Baseline): Name resolution SHALL follow deterministic precedence across lexical, module, import, and prelude scopes.
- AXIS-CB-114 (v0.1 Baseline): Standard prelude SHALL be in scope by default.
- AXIS-CB-115 (v0.1 Baseline): Cross-package access SHALL require explicit public exposure and explicit reference.

## 12. Compatibility and Evolution Rules

- AXIS-CB-120 (v0.1 Baseline): Product evolution SHALL preserve behavior of valid v0.1 programs unless an explicit versioned breaking change policy is adopted.
- AXIS-CB-121 (v0.1 Baseline): Deferred capabilities SHALL include explicit activation criteria before becoming baseline.
- AXIS-CB-122 (v0.1 Baseline): New trusted or unsafe surfaces SHALL be explicitly marked and isolated from safe defaults.
