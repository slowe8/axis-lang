# Axis Design Questions

This document turns the current spec gaps into concrete design decisions.
The goal is to answer these in order and then fold the decisions back into the core spec.

## How to Use This

For each question, decide:
- the intended behavior
- whether it is part of v0.1 or explicitly deferred
- whether it is a language feature, standard library feature, or implementation detail

---

## 1. What Is In The Safe v0.1 Core?

The spec promises safety early, but some mechanisms are still marked planned.

Questions:
- Is `&T` part of v0.1, or deferred until a borrow checker exists?
- Is `&mut T` completely unavailable in v0.1?
- Are arenas available in safe code in v0.1, or only later?
- Is there any `unsafe` or trusted escape hatch in v0.1?

Decision needed:
- Define the exact set of constructs that are legal in safe v0.1 code.

Status:
- Decided: Option B.

Decision summary:
- Safe v0.1 includes owned values, `let`/`var`, shared immutable references (`&T`), and explicit `Result`/`Option` error handling.
- Deferred from safe v0.1: `&mut T`, safe arena semantics, arena promotion, and borrowed captures across `await`.

C-readiness constraints:
- Keep aliasing rules conservative in v0.1 so adding `&mut T` is additive.
- Treat arenas as a type-system feature (not only a runtime allocator detail).
- Keep async borrowing conservative until lifetime rules across suspension are specified.
- Require monotonic compatibility: valid v0.1 programs keep their meaning as C features arrive.

---

## 2. What Are The Numeric Operator Semantics?

The spec currently says `+`, `-`, `*` apply to vector and matrix operations, but does not define which meanings attach to which operand pairs.

Questions:
- Is `vec * vec` elementwise multiply or dot product?
- Is dot product spelled with a named function like `dot(a, b)` instead of `*`?
- Does `mat * vec` mean standard linear algebra multiplication?
- Does scalar broadcasting exist for `scalar * vec` and `scalar * mat`?
- Are `/` and unary `-` defined for vectors and matrices?

Decision needed:
- Define a complete operator table for scalar, vector, and matrix operands.

Status:
- Decided.

Decision summary:
- `vec * vec` is elementwise (Hadamard), not dot product.
- Dot product is explicit: `dot(a, b)`.
- Cross product is explicit: `cross(a, b)` and only valid for 3D vectors.
- `mat * vec` and `mat * mat` use standard linear algebra multiplication with dimension checks.
- Scalar multiply/divide with vectors and matrices is supported.
- Unary negation for vectors and matrices is elementwise.

v0.1 restrictions:
- `vec / vec` and `mat / mat` are not defined.
- No implicit broadcasting beyond scalar-to-vector and scalar-to-matrix scaling.
- Binary numeric operators require matching element types unless explicitly cast.
- Shape mismatches are compile-time errors when dimensions are statically known.

---

## 3. What Are The Matrix Layout And Convention Rules?

Known layout is a stated goal, but layout and multiplication convention are not yet specified.

Questions:
- Are matrices row-major or column-major in memory?
- Are vectors treated as row vectors or column vectors by default?
- Does `mat4x4` interoperate with GPU APIs using the same layout, or is explicit conversion required?
- Is layout part of the type, or a fixed language-wide convention?

Decision needed:
- Lock down the default memory layout and algebra convention.

Status:
- Decided.

Decision summary:
- Vectors are treated as column vectors.
- `mat * vec` follows standard linear algebra application (`y = A * x`).
- Matrix storage is column-major by default in v0.1.
- Indexing syntax is logical `m[row, col]` and does not expose storage order.

Interop and compatibility:
- Numeric APIs assume column-major buffers by default.
- Row-major interop requires explicit conversion.
- No implicit transpose at API or FFI boundaries.
- v0.1 keeps layout fixed; future layout-qualified matrix types may be added additively.

---

## 4. What Is The Mutability Model?

The spec defines immutable and mutable bindings, but not whether mutability belongs to bindings, places, references, or values.

Questions:
- Does `var` make the binding mutable, the value mutable, or both?
- Can fields of a struct be mutated through a `var` binding without `&mut`?
- Are arrays mutable by index only when bound with `var`?
- Will interior mutability exist, or is all mutation explicit through borrow rules?

Decision needed:
- Define the rules for assignment, field mutation, and indexed mutation.

Status:
- Decided.

Decision summary:
- Mutability is a property of bindings and places, not values.
- `let` bindings are immutable and not reassignable.
- `var` bindings are reassignable, subject to move rules.
- Field and indexed mutation are allowed only through mutable owned places.
- Shared references `&T` are read-only and cannot be used for mutation.
- `&mut T` is deferred from safe v0.1.

v0.1 restrictions and compatibility:
- Interior mutability is not part of the safe v0.1 core.
- No hidden aliasing loopholes in safe code.
- Mutable borrowing in C-stage must be additive and must not change valid v0.1 behavior.

---

## 5. What Is The Ownership And Borrowing Roadmap?

Move semantics are stated, but the operational model is not yet nailed down.

Questions:
- Which types are copyable by default, if any?
- Is `Copy` implicit for primitive scalars only, or trait-based like Rust?
- Can values be partially moved out of structs or tuples?
- Are borrows lexical only in the first implementation, or is non-lexical borrowing a goal from the start?

Decision needed:
- Specify the minimum ownership model needed for the first compiler.

Status:
- Decided.

Decision summary:
- Move-by-default semantics for non-trivial values.
- Use-after-move is a compile-time error.
- Implicit copy in v0.1 is limited to primitive scalars and `bool`.
- Non-scalar aggregates are move-only unless explicitly duplicated via library APIs.
- Shared immutable borrows are supported; mutable borrows are deferred.

v0.1 simplifications and compatibility:
- Partial moves out of structs/tuples are not supported in v0.1.
- Lexical borrow scopes are sufficient for the first compiler.
- Future non-lexical borrowing may relax rejections but must preserve valid v0.1 program meaning.
- Future mutable-borrow features are additive only.

---

## 6. What Exactly Is An Arena Value?

Arenas are central to the language identity, but the value model is still open.

Questions:
- Does `frame.alloc(T)` return `&T`, `&mut T`, a handle, or a distinct arena reference type?
- Can arena-allocated values contain borrows to stack values?
- Can non-arena values borrow from arena values?
- Are destructors run for arena values, or is arena teardown a bulk drop with restricted semantics?

Decision needed:
- Define the type shape and lifetime behavior of arena allocations.

Status:
- Decided.

Decision summary:
- `frame.alloc(T)` returns explicit region-typed `ArenaRef<'a, T>`.
- `ArenaRef` is read-only in safe v0.1.
- Mutable arena references are deferred with mutable borrowing.
- Arena refs are lifetime-bound and cannot escape without explicit promotion.

v0.1 constraints and compatibility:
- Arena values may not embed short-lived stack borrows that could dangle.
- Bulk arena teardown is the default reclamation behavior.
- v0.1 assumes trivial-drop arena payloads.
- Future mutable arena access and destructor policy must be additive and preserve valid v0.1 behavior.

---

## 7. What Does Arena Promotion Mean?

The spec says arena values cannot escape unless promoted, but promotion is not defined.

Questions:
- What operations count as promotion?
- Can a value be promoted from one arena to another?
- Is promotion shallow or deep?
- What happens if the promoted value contains references back into the old arena?
- Is promotion explicit syntax, an API call, or an inferred compiler action?

Decision needed:
- Define promotion as a type-checked operation with explicit constraints.

Status:
- Decided.

Decision summary:
- Promotion is always explicit.
- Promotion is deep in v0.1.
- Promotion is type-checked and fails compilation when safe rebinding is not possible.
- Promotion has explicit cost and is never inferred.

Safety and compatibility constraints:
- Promoted values must not retain references into the source arena.
- Escape across arena lifetime boundaries requires explicit promotion.
- Promotion semantics are deterministic and must remain additive as arena capabilities expand.

---

## 8. What Makes Tasks Structured?

The spec uses the term structured concurrency, but does not yet define the lifecycle rules.

Questions:
- Can a child task outlive its parent scope?
- How are child tasks spawned: implicit, explicit, or both?
- Is cancellation automatic when a parent exits early?
- How are failures propagated across sibling tasks?
- Can a task borrow stack or arena data from its parent?

Decision needed:
- Define the task tree and the lifetime rules that make concurrency structured.

Status:
- Decided.

Decision summary:
- Tasks form a strict parent-child tree.
- Child tasks are explicitly spawned and cannot outlive parent scope.
- Parent scope must deterministically join or cancel children before exit.
- Parent-initiated early exit cancels remaining children cooperatively.

Child failure policy:
- Child failure handling is scope-configurable.
- Default policy is fail-fast: child failure propagates and cancels siblings.
- Non-fail-fast policy is valid for expected non-fatal child outcomes (for example option-like results).
- Under non-fail-fast policy, siblings continue and all children finish/join before parent continuation.

v0.1 constraints and compatibility:
- Child tasks in safe v0.1 use owned captures only.
- Borrowed parent stack or arena captures are deferred pending async lifetime rules.
- Detached task behavior is outside structured safe v0.1 unless explicitly introduced later.

---

## 9. What Is The `await` Model?

The spec shows `await`, but its expression behavior is not spelled out.

Questions:
- Is `await expr` an expression everywhere expressions are allowed?
- Can `await` appear inside loops, match arms, and nested expressions?
- Are tasks lazy futures, eager tasks, or a third model?
- What is the runtime contract for `task` return values?

Decision needed:
- Define whether `task` behaves more like a future constructor, coroutine, or lightweight thread.

Status:
- Decided.

Decision summary:
- `await` is an expression in `task` contexts.
- `await` in synchronous `fn` is a compile-time error.
- Calling a `task` function produces a task value.
- Execution starts via explicit `await` or explicit scoped spawn.
- No implicit detached/background task execution in safe v0.1.

Typing and policy interaction:
- If `expr` has type `Task<T>`, then `await expr` has type `T`.
- Structured cancellation/failure behavior follows the parent scope policy (fail-fast or configured non-fail-fast).

v0.1 constraints and compatibility:
- Awaited tasks use owned captures in safe v0.1.
- Borrowed captures across suspension are deferred pending async lifetime rules.
- Future async enhancements must be additive and preserve valid v0.1 await semantics.

---

## 10. How Does `?` Choose The Error Type?

Error propagation is shown, but conversion rules are not.

Questions:
- Must the error type match exactly for `?` to compile?
- Is there an implicit conversion trait or protocol?
- Can `Option<T>` be lifted into `Result<T, E>` with `?`, or only through explicit adapters?
- Is custom error conversion available in v0.1?

Decision needed:
- Define the coercion or non-coercion rules for `?`.

Status:
- Decided.

Decision summary:
- In v0.1, `?` requires exact error-type matching for `Result`.
- No implicit error conversion is performed by `?` in v0.1.
- Explicit adapters (for example `map_err(...)`) are used for conversion before `?`.

Option interaction:
- `?` on `Option<T>` is valid in `Option`-returning contexts.
- No implicit `Option` to `Result` lifting in v0.1.
- `Option` to `Result` conversion requires explicit adapter calls (for example `ok_or(...)`).

Compatibility note:
- Future implicit conversion traits/protocols may be added additively.
- Existing v0.1 `?` behavior must remain stable.

---

## 11. Are `Result` And `Option` Language Items Or Library Types?

They are listed as keywords and also shown as enums.

Questions:
- Are `Result` and `Option` reserved words, prelude types, or ordinary library enums?
- Can users shadow these names in local scope?
- Are `Ok`, `Err`, `Some`, and `None` language-known constructors or just enum variants?

Decision needed:
- Separate true syntax from standard library surface area.

Status:
- Decided.

Decision summary:
- `Result` and `Option` are standard library enums, not reserved keywords.
- `Ok`, `Err`, `Some`, and `None` are enum variants, not special syntax forms.
- Prelude exposure makes these names available by default.
- Compiler behavior (for example `?`) is type-driven, not keyword-driven.

Name resolution and compatibility:
- Local shadowing is technically possible but should be discouraged by linting.
- Fully qualified paths remain available for disambiguation.
- This model keeps language syntax minimal and preserves additive extensibility.

---

## 12. What Is The Decorator Contract?

Decorators are introduced as performance hints and trusted low-level escape hatches, but their semantic weight is unclear.

Questions:
- Are `@simd`, `@parallel_for`, and `@gpu_kernel` mandatory requests, best-effort hints, or compile-time errors when unsupported?
- Can decorators change observable behavior, or only performance?
- Is `@trusted_aliasing` effectively an `unsafe` feature?
- What proof obligations fall on the programmer when using trusted decorators?

Decision needed:
- Define which decorators are advisory and which introduce a trusted or unsafe contract.

Status:
- Decided.

Decision summary:
- `@simd`, `@parallel_for`, and `@gpu_kernel` are advisory performance decorators.
- Advisory decorators express optimization intent and do not change observable safe-code semantics.
- Advisory decorators may be honored, ignored, or rejected based on target/backend capability.
- Clear diagnostics are required when advisory requests are not applied.

Strictness and trusted boundary:
- Default mode keeps advisory decorators best-effort.
- Optional strict-performance mode may upgrade unmet advisory requests to compile errors.
- `@trusted_aliasing(...)` is a trusted boundary outside safe v0.1.
- Trusted misuse may cause undefined behavior and carries programmer proof obligations.

Compatibility and portability:
- Unsupported targets must emit explicit diagnostics.
- Future decorators may be added, but semantic-affecting decorators must be explicitly categorized as trusted/unsafe.

---

## 13. What Is In The Minimal Surface Syntax For v0.1?

Several examples imply syntax that is not yet specified.

Questions:
- Are blocks expressions that return the last value?
- Are semicolons required after all statements except tail expressions?
- What are the literal forms for integers, floats, strings, chars, and booleans?
- What range syntax exists beyond `0..10`, if any?
- What pattern syntax is supported in `match`?

Decision needed:
- Define the minimal grammar surface required for a useful first implementation.

Status:
- Decided.

Decision summary:
- v0.1 grammar is intentionally minimal and implementation-focused.
- Blocks are expressions with tail-expression return semantics.
- `let`/`var` and expression statements require semicolons; tail expressions do not.
- v0.1 literals include decimal ints/floats, booleans, strings, and chars.
- v0.1 range support is limited to half-open `a..b`.
- v0.1 match patterns are limited to literals, wildcard, and simple enum-variant payload binding.

Full conceptual syntax plan (beyond v0.1):
- Define the full syntax surface now as a roadmap category set, even when implementation is deferred.
- Planned categories include richer patterns, extended literal forms, broader ranges, module/visibility syntax, and expanded generic/method forms.
- Async and arena-driven syntax additions are expected but must be explicitly specified when introduced.

Compatibility constraint:
- Future syntax is additive and must not reinterpret valid v0.1 programs.

---

## 14. What Is The Module And Visibility Story?

The examples use paths like `fs::read_to_string` and `runtime::block_on`, but the spec does not define modules.

Questions:
- What is the file-to-module mapping?
- Is there `mod`, `use`, `pub`, or a different model?
- Is the standard library always in scope, or imported through a prelude?
- How does name resolution work across packages and modules?

Decision needed:
- Define the smallest viable module system so examples become legal language, not just notation.

Status:
- Decided.

Decision summary:
- File and directory structure define module hierarchy.
- `mod` declares submodules, `use` imports, and `pub` controls visibility.
- Items are private by default unless explicitly exported.
- Name resolution proceeds through lexical scope, current module, explicit imports, then prelude.
- Standard prelude is in scope by default in v0.1.

Package and compatibility rules:
- Cross-package visibility is explicit.
- Public API is defined through exported `pub` module surfaces.
- Future namespace features must be additive and must not reinterpret valid v0.1 paths or visibility behavior.

---

## Suggested Discussion Order

If the goal is to stabilize the language core quickly, answer these first:

1. Safe v0.1 core
2. Numeric operator semantics
3. Matrix layout and convention
4. Mutability model
5. Ownership and borrowing roadmap
6. Arena value model
7. Arena promotion
8. Structured tasks

The remaining questions can follow once the core execution and memory model are stable.

## Finalization Status

All 14 core design questions in this document are resolved for the current draft pass.