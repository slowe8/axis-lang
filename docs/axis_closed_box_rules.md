# Axis Closed-Box Requirement Writing Rules

## Purpose

This document defines mandatory writing rules for closed-box requirements.
Closed-box requirements must stay non-technical and externally testable.

---

## 1. Normative Language Rules

- Use SHALL for mandatory behavior.
- Use SHALL NOT for prohibited behavior.
- Use MAY only for explicitly optional behavior.
- Do not use should, could, ideally, typically, or similar non-normative terms in requirement statements.

## 2. Requirement Structure Rules

Each requirement entry must include:
- Requirement ID.
- Scope tag.
- One normative statement.

Recommended format:
- AXIS-CB-### (Scope): Product SHALL ...

## 3. Scope Tag Rules

Allowed tags:
- v0.1 Baseline: required now.
- Deferred: approved concept but not available in baseline behavior.
- Additive Future: future-approved direction that must not break baseline semantics.

Rules:
- Every requirement must include exactly one scope tag.
- Deferred requirements must not be interpreted as baseline product obligations.

## 4. Closed-Box Content Boundary

Closed-box requirements must describe what the product guarantees, not how internals are implemented.

Allowed:
- Observable behavior.
- Contractual constraints.
- Versioned availability and compatibility promises.

Not allowed in closed-box statements:
- Internal algorithm names.
- Data-structure choices.
- Compiler phase details.
- Backend implementation details.

Those details belong in open-box technical requirements.

## 5. Prohibition and Exception Rules

- Use SHALL NOT for disallowed behavior.
- If exceptions exist, define them in separate requirement IDs.
- Do not embed multiple exception branches inside one requirement sentence.

## 6. Measurability Rules

Every requirement must be testable as pass/fail through one of:
- Build-time acceptance/rejection behavior.
- Runtime observable behavior.
- API/contract conformance.

If a requirement cannot be tested, rewrite it.

## 7. Compatibility Rules

- New requirements must not reinterpret valid baseline behavior unless explicitly version-gated.
- Additive Future requirements must preserve baseline semantics by default.
- Breaking changes require explicit version policy outside this document.

## 8. Naming and ID Rules

- Closed-box IDs use AXIS-CB-###.
- IDs are immutable once published.
- If wording changes but intent remains, keep ID and revise text.
- If intent changes materially, deprecate old ID and add a new ID.

## 9. Traceability Rules

- Each closed-box requirement must map to at least one source decision.
- Mappings are recorded in axis_requirements_traceability.md.
- No closed-box requirement should be orphaned from decision rationale.

## 10. Review Checklist

Before accepting a closed-box update:
- Requirement uses SHALL or SHALL NOT.
- Requirement has one scope tag.
- Statement is externally verifiable.
- Statement avoids internal implementation mechanisms.
- Statement has traceability mapping.
