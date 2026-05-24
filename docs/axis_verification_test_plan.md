# Axis Verification and Test Plan (Draft 0.1)

## 1. Purpose

This plan defines how Axis requirements are verified.
It maps closed-box requirements (AXIS-CB-###) and open-box requirements (AXIS-OB-###) to test suites, evidence, and exit criteria.

## 2. Scope

- In scope: v0.1 baseline requirements and deferred-gate checks.
- Out of scope: full implementation of deferred features.

## 3. Verification Levels

- V1: Parser and syntax validation tests.
- V2: Name resolution and module visibility tests.
- V3: Type system and ownership/borrow checking tests.
- V4: Async task lifecycle and policy tests.
- V5: Runtime/interop behavior tests.
- V6: Conformance and compatibility regression tests.

## 4. Test Artifact Rules

- Each test suite has an ID: AXIS-TV-###.
- Each suite defines positive tests, negative tests, and acceptance criteria.
- Negative tests must validate deterministic diagnostic categories.
- Every AXIS-CB requirement must map to at least one AXIS-TV suite.
- Every AXIS-OB requirement must map to at least one AXIS-TV suite.

## 5. Requirement-to-Test Matrix

| Test Suite | Verification Level | Requirement Mapping | Test Method | Acceptance Criteria | Evidence |
|---|---|---|---|---|---|
| AXIS-TV-001 Safe Core Ownership | V3 | CB: 001..006, 015, 040..045; OB: 001..005, 091 | Compile positive and negative ownership/borrow samples | 100% expected accept/reject outcomes; deterministic diagnostic class for each negative case | Compiler test logs and baseline snapshots |
| AXIS-TV-002 Feature Gating | V6 | CB: 010..018, 121, 122; OB: 090..092 | Capability manifest checks and compile gating tests | Deferred features rejected in v0.1 mode; baseline features available | Feature-gate report |
| AXIS-TV-003 Numeric Semantics | V3/V5 | CB: 020..027; OB: 010..013 | Operator resolution tests and runtime numeric oracle comparisons | All legal operations type-check and match expected results; illegal ops rejected | Type-check logs and runtime assertions |
| AXIS-TV-004 Matrix Layout and Interop | V5 | CB: 030..035; OB: 020..022 | Memory layout fixtures, indexing checks, FFI boundary tests | Column-major layout confirmed; no implicit transpose; explicit conversion required for row-major | Layout inspection outputs and interop tests |
| AXIS-TV-005 Async Structured Lifecycle | V4 | CB: 060..071; OB: 030..033 | Structured concurrency scenario tests (join/cancel/fail policy/await legality) | Parent-child lifecycle rules always enforced; fail-fast default and non-fail-fast policy behave as specified | Runtime event traces and policy outcome logs |
| AXIS-TV-006 Error Propagation | V3 | CB: 080..083; OB: 040..042 | Question-mark typing and adapter tests | Exact-match error typing enforced; Option/Result adapter behavior matches spec | Type-check diagnostics and success cases |
| AXIS-TV-007 Decorator Contract | V3/V5 | CB: 090..094; OB: 060..061 | Advisory decorator behavior and trusted-boundary gating tests | Advisory decorators never alter observable safe semantics; unsupported requests emit diagnostics; trusted path gated | Compiler diagnostics and behavioral diffs |
| AXIS-TV-008 Syntax Baseline | V1 | CB: 100..105; OB: 070..072 | Grammar conformance and parsing ambiguity tests | Baseline grammar accepted; unsupported syntax rejected; additive compatibility snapshots remain stable | Parser snapshots and rejection logs |
| AXIS-TV-009 Module and Visibility | V2 | CB: 110..115; OB: 050..052 | Multi-module package fixtures with visibility and import cases | Name resolution precedence is deterministic; private-by-default enforced | Resolver traces and compile outcomes |
| AXIS-TV-010 Compatibility Regression | V6 | CB: 018, 105, 120..122; OB: 072, 092 | Golden corpus recompile across versions/modes | Valid baseline corpus behavior remains unchanged; additive features do not reinterpret baseline programs | Golden corpus diff report |
| AXIS-TV-011 Deferred Arena Readiness Gate | V6 | CB: 050..055; OB: 090, 091 | Deferred-requirement gate checks and placeholder API probes | Arena semantics remain disabled in safe v0.1 mode while design placeholders stay consistent | Gate checklist |

## 6. Coverage Summary

- Closed-box coverage target: 100% of AXIS-CB IDs mapped.
- Open-box coverage target: 100% of AXIS-OB IDs mapped.
- Negative-test target: at least one negative case for every rejection requirement.
- Determinism target: stable diagnostic category for repeated runs.

## 7. Exit Criteria for v0.1 Verification

- All suites AXIS-TV-001 through AXIS-TV-011 executed.
- No open critical failures on baseline requirements.
- Deferred-gate suites pass with expected rejections/disabled behavior.
- Compatibility regression suite confirms no reinterpretation of valid baseline corpus.

## 8. Architecture Handoff Inputs

This plan provides architecture inputs by identifying where each requirement is enforced:

- Parser-focused: AXIS-TV-008
- Name Resolver-focused: AXIS-TV-009
- Type Checker/Borrow Checker-focused: AXIS-TV-001, 003, 006
- Async Runtime-focused: AXIS-TV-005
- Backend/Interop-focused: AXIS-TV-004, 007
- Cross-cutting compatibility and release governance: AXIS-TV-002, 010, 011

These inputs should be used to define component boundaries and responsibility allocation in the architecture phase.

Architecture output artifact:
- docs/axis_architecture_spec.md

Implementation roadmap artifact:
- docs/axis_implementation_roadmap.md
