# Axis Milestone 0 Task List

## Purpose

This task list turns the Project Skeleton and Tooling milestone into executable work items.

Primary references:
- docs/axis_architecture_spec.md
- docs/axis_verification_test_plan.md
- docs/axis_implementation_roadmap.md

## Task List

### M0-T1: Stabilize Repository Entry Points

Goal:
- Ensure the Rust crate has a stable library root and minimal executable entry point.

Work items:
- Keep `src/main.rs` as the binary entry point.
- Keep `src/lib.rs` as the shared architecture module root.
- Ensure cargo builds the binary and library targets cleanly.

Acceptance:
- `cargo check` succeeds.
- The repository has a stable library root for future modules.

### M0-T2: Define Architecture-Aligned Module Skeleton

Goal:
- Create the top-level module files for the planned compiler/runtime components.

Work items:
- Maintain module files for frontend, resolution, types, borrow, runtime, backend, and diagnostics.
- Keep module names aligned with the architecture spec.
- Preserve a flat scaffold until deeper component directories are needed.

Acceptance:
- Each architecture component has a matching Rust module stub.
- Module names map directly to architecture terminology.

### M0-T3: Establish Diagnostic and Test Conventions

Goal:
- Define stable naming conventions for diagnostics and tests.

Work items:
- Use requirement IDs AXIS-CB-### and AXIS-OB-###.
- Use test IDs AXIS-TV-###.
- Reserve diagnostic IDs in a consistent future format.

Acceptance:
- Written conventions are available in the rules and verification docs.
- Future tests can be named without renaming the framework.

### M0-T4: Create Verification Harness Entry Points

Goal:
- Prepare the repository for requirement-linked verification suites.

Work items:
- Establish a place for parser, type, borrow, runtime, interop, and compatibility suites.
- Keep suite names aligned with AXIS-TV-001..011.
- Ensure failing cases can be tracked deterministically.

Acceptance:
- Verification plan can be executed against a known suite inventory.
- Future test files have a stable naming scheme.

### M0-T5: Document Release-Gate Rules

Goal:
- Make it explicit when Milestone 0 is considered complete.

Work items:
- Link milestone completion to AXIS-TV-010 and AXIS-TV-011 readiness.
- Require that baseline scaffolding exist before parser implementation work begins.
- Preserve additive-only evolution for future milestones.

Acceptance:
- Milestone 0 exit criteria are written and visible.
- Milestone 1 can start without reworking repository structure.

## Sequence

1. M0-T1
2. M0-T2
3. M0-T3
4. M0-T4
5. M0-T5

## Completion Criteria

- `cargo check` passes.
- Module scaffold is present.
- Naming conventions are documented.
- Verification hooks are ready.
- Milestone 1 can begin with parser work.
