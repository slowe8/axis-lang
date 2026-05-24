# Axis Milestone 1 Task List

## Purpose

This task list turns the parser milestone into executable work items.

Primary references:
- docs/axis_implementation_roadmap.md
- docs/axis_verification_test_plan.md
- docs/axis_architecture_spec.md

## Task List

### M1-T1: Split Frontend Layers

Goal:
- Separate lexer, AST, and parser responsibilities.

Work items:
- Keep lexer logic isolated from parsing logic.
- Keep syntax trees isolated from tokenization logic.
- Keep parser logic focused on grammar and tree construction.

Acceptance:
- Frontend modules are independently readable and testable.

### M1-T2: Implement Baseline Tokens

Goal:
- Support baseline lexical tokens needed for v0.1 syntax.

Work items:
- Identifier tokens.
- Integer tokens.
- Semicolon tokens.
- End-of-file handling.

Acceptance:
- Tokenization is deterministic for baseline inputs.

### M1-T3: Implement Baseline AST Shapes

Goal:
- Define the initial syntax tree shapes used by the parser.

Work items:
- Expression nodes.
- Statement nodes.
- Program root node.

Acceptance:
- Parser can construct a stable baseline AST.

### M1-T4: Implement Baseline Parser Entry Point

Goal:
- Parse a source string into a program AST.

Work items:
- Program-level parse entry.
- Statement parsing.
- Expression parsing for identifiers and integers.

Acceptance:
- Valid minimal baseline inputs parse into AST nodes.

### M1-T5: Add Parser Regression Tests

Goal:
- Lock parser behavior with positive and negative tests.

Work items:
- Add acceptance cases for valid baseline syntax.
- Add rejection cases for unsupported forms.

Acceptance:
- Parser behavior is stable and predictable.

## Sequence

1. M1-T1
2. M1-T2
3. M1-T3
4. M1-T4
5. M1-T5

## Completion Criteria

- Lexer, AST, and parser modules exist.
- Parser can build a simple AST from minimal input.
- The milestone has a matching test plan.
