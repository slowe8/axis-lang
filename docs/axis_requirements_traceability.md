# Axis Requirements Traceability Matrix

## Purpose

This matrix maps closed-box requirement IDs to resolved design questions and source anchors.

Sources:
- docs/axis_core_spec.md
- docs/axis_design_questions.md

---

## Matrix

| Closed-Box ID | Decision Source | Primary Topic | Source Anchors |
|---|---|---|---|
| AXIS-CB-001..006 | Q1, Q4, Q5 | Safe core and ownership/borrowing baseline | axis_core_spec: v0.1 scope, mutability, ownership roadmap; axis_design_questions: Q1, Q4, Q5 |
| AXIS-CB-010..018 | Q1 | Feature availability and deferral boundaries | axis_core_spec: v0.1 scope and notes; axis_design_questions: Q1 |
| AXIS-CB-020..027 | Q2 | Numeric operator semantics | axis_core_spec: numeric operator semantics; axis_design_questions: Q2 |
| AXIS-CB-030..035 | Q3 | Matrix layout and interop behavior | axis_core_spec: matrix layout and convention; axis_design_questions: Q3 |
| AXIS-CB-040..045 | Q4 | Binding/place mutability and mutation constraints | axis_core_spec: mutability model; axis_design_questions: Q4 |
| AXIS-CB-050..055 | Q6, Q7 | Arena value and promotion behavior | axis_core_spec: arena value model and promotion semantics; axis_design_questions: Q6, Q7 |
| AXIS-CB-060..071 | Q8, Q9 | Structured task lifecycle and await model | axis_core_spec: structured task lifecycle and await model; axis_design_questions: Q8, Q9 |
| AXIS-CB-080..083 | Q10 | Question-mark propagation and conversion rules | axis_core_spec: error handling and question-mark rules; axis_design_questions: Q10 |
| AXIS-CB-090..094 | Q12 | Decorator contract and trusted boundary | axis_core_spec: decorator contract; axis_design_questions: Q12 |
| AXIS-CB-100..105 | Q13 | Minimal syntax baseline and additive syntax growth | axis_core_spec: syntax roadmap; axis_design_questions: Q13 |
| AXIS-CB-110..115 | Q14 | Module and visibility model | axis_core_spec: modules and visibility; axis_design_questions: Q14 |
| AXIS-CB-120..122 | Q1..Q14 | Evolution and compatibility policy | axis_core_spec: forward-compatibility notes; axis_design_questions: finalization status |

---

## Coverage Check

Resolved decisions covered:
- Q1 through Q14 all mapped to at least one closed-box requirement range.

Closed-box coverage:
- All AXIS-CB requirements in axis_closed_box_requirements.md are represented in the matrix by ID range.

---

## Open-Box Linkage

Open-box technical requirements in axis_open_box_requirements.md must reference at least one AXIS-CB ID.

Verification linkage:
- Verification suites in axis_verification_test_plan.md must map to both AXIS-CB and AXIS-OB IDs.
