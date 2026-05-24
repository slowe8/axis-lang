
# Axis Language Specification – Draft 0.1

> Axis is a Rust-flavored, no-GC systems language with first-class linear algebra types, arenas, and structured async.

---

## 1. Design Goals

Axis aims to be:

1. **Safe by default**
   - Ownership / borrowing with compile-time checks.
   - No data races or use-after-free in safe code.
   - Result-based error handling; no exceptions.

2. **Systems-capable**
   - No garbage collector.
   - Direct control over memory, threads, and FFI.
   - Predictable performance.

3. **Numerics-native**
   - Built-in vector/matrix types with known layout.
   - SIMD / parallel / GPU hints via decorators.
   - Designed for high-performance linear algebra and low-level math.

4. **Arena-centric**
   - Arenas / regions are first-class, with syntax and type-system support.
   - Ideal for per-request, per-frame, or scratch allocation patterns.

5. **Modern concurrency**
   - Clear distinction between synchronous `fn` and asynchronous `task`.
   - Structured concurrency.
   - Channels for safe message passing.

### 1.1 v0.1 Scope and Forward Compatibility

Axis v0.1 adopts a conservative safe core (Option B):
- Owned values and move semantics.
- Immutable and mutable bindings (`let` / `var`).
- Shared immutable references (`&T`).
- Result/Option-based error handling.

The following are explicitly deferred from safe v0.1:
- Mutable references (`&mut T`).
- Safe arena allocation semantics and promotion.
- Trusted aliasing escape hatches in safe code.
- Borrowed captures across `await` suspension points.

Forward-compatibility rule:
- Future Option C features must be additive and must not change the meaning of valid v0.1 programs.
- v0.1 rules are therefore chosen to preserve a monotonic path to mutable borrowing, arenas, and richer async lifetimes.

---

## 2. Lexical Structure

### 2.1. Source files
- UTF-8 text.
- Unix line endings recommended.

### 2.2. Comments
- `//` for line comments.
- `/* ... */` for block comments.

### 2.3. Identifiers
- ASCII letters or `_` to start, then alphanumeric or `_`.
- Case-sensitive.

Reserved keywords:
```
fn task let var arena
if else while for match
true false return
struct enum type impl
```

---

## 3. Types

Axis is statically, strongly typed with local inference.

### 3.1. Primitive types
- `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `bool`

### 3.2. Composite types
#### Arrays
```
let xs: [i32; 4] = [1, 2, 3, 4];
```

#### Tuples
```
let pair: (i32, f32) = (1, 2.0);
```

#### Structs
```
struct Point { x: f32, y: f32 }
```

#### Enums (sum types)
```
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

### 3.3. Linear algebra types
#### Fixed-size vectors
```
let v: vec4<f32> = [1.0, 2.0, 3.0, 4.0];
```

#### Fixed-size matrices
```
let m: mat4x4<f32> = mat4x4::identity();
```

### 3.3.1 Numeric operator semantics (v0.1)

Axis defines numeric operators with explicit shape checks.

- `scalar + scalar`, `scalar - scalar`, `scalar * scalar`, `scalar / scalar` are allowed.
- `vecN<T> + vecN<T>` and `vecN<T> - vecN<T>` are elementwise.
- `vecN<T> * scalar` and `scalar * vecN<T>` are elementwise scaling.
- `vecN<T> / scalar` is elementwise scaling by reciprocal.
- Unary `-vecN<T>` is elementwise negation.
- `vecN<T> * vecN<T>` is elementwise (Hadamard) multiplication.

Dot and cross are explicit functions:
- `dot(a, b)` for vector dot product.
- `cross(a, b)` for 3D vectors only.

Matrix operators:
- `matMxN<T> + matMxN<T>` and `matMxN<T> - matMxN<T>` are elementwise.
- `matMxN<T> * scalar` and `scalar * matMxN<T>` are scaling.
- `matMxN<T> / scalar` is scaling by reciprocal.
- Unary `-matMxN<T>` is elementwise negation.
- `matMxN<T> * vecN<T>` is linear algebra multiplication and produces `vecM<T>`.
- `matMxN<T> * matNxP<T>` is linear algebra multiplication and produces `matMxP<T>`.

v0.1 restrictions:
- `vec / vec` is not defined.
- `mat / mat` is not defined.
- No implicit broadcasting beyond scalar-to-vector and scalar-to-matrix scaling.
- Binary numeric operators require matching element types unless an explicit cast is used.
- Shape mismatches are compile-time errors when dimensions are known.

### 3.3.2 Matrix layout and convention (v0.1)

Axis uses a fixed matrix convention in v0.1:

- Vectors are treated as column vectors.
- Matrix-vector multiplication follows linear algebra form `y = A * x`.
- Matrix storage is column-major in memory.
- Indexing syntax is logical `m[row, col]` and is independent of storage order.

Interop rules:
- Standard library and FFI-facing numeric APIs should assume column-major buffers by default.
- Row-major interop requires explicit conversion.
- No implicit transpose is performed at API or FFI boundaries.

Forward-compatibility note:
- v0.1 keeps matrix layout fixed to avoid semantic drift.
- Future layout-qualified matrix types may be added as an additive extension.

#### Generic matrices (planned)
```
Matrix<T, M, N>
Vector<T, N>
```

### 3.4. Result/Option
Primary error/optional system.

v0.1 library status:
- `Result` and `Option` are standard library enums provided by the prelude.
- `Ok`, `Err`, `Some`, and `None` are enum variants, not reserved syntax forms.
- Compiler behavior such as `?` is type-driven and does not require keyword status for these names.

---

## 4. Bindings & Mutability

- `let` → immutable
- `var` → mutable

```
let x = 10;
var y = 0;
y += 1;
```

### 4.1 Mutability model (v0.1)

Axis v0.1 treats mutability as a property of bindings and places, not values.

- `let` bindings are not reassignable.
- `var` bindings are reassignable, subject to move rules.
- In-place mutation is allowed only through mutable owned places.

Examples:
- If `p` is a `var` binding to a struct value, `p.field = ...` is allowed.
- If `xs` is a `var` binding to an array/vector/matrix value, indexed element assignment is allowed.
- The same mutations are rejected when the root place is bound with `let`.

Reference behavior in v0.1:
- Shared references `&T` are read-only.
- Mutation through `&T` is not allowed.
- `&mut T` is deferred from safe v0.1.

Interior mutability:
- Not part of the safe v0.1 core.
- Any future interior mutability or trusted mutation primitives must be explicit and isolated from safe defaults.

---

## 5. Functions

### 5.1. Synchronous functions
```
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Generics:
```
fn max<T: Ord>(a: T, b: T) -> T { ... }
```

### 5.2. Methods (planned)
```
impl Vector2 {
    fn length(self) -> f32 { ... }
}
```

---

## 6. Expressions & Statements

- Binary ops: `+`, `-`, `*`, `/`
- Calls: `foo(x)`
- Field access: `p.x`
- Indexing: `arr[i]`
- Enum variants: `Ok(42)`

Statements:
- `let`, `var`
- `expr;`
- `return expr;`

### 6.1 Syntax roadmap

Axis defines syntax in two layers:

- v0.1 minimal syntax: required for the first stable compiler.
- Full conceptual syntax: the intended long-term surface, including deferred features.

### 6.2 v0.1 minimal syntax

Expression and block model:

- Blocks are expressions.
- A trailing expression without `;` is the block value.
- Adding `;` makes the expression statement-like and yields unit.

Statement termination:

- `let`/`var` statements require `;`.
- Expression statements require `;`.
- Tail expressions in blocks omit `;`.

Literal forms:

- Integers: decimal literals.
- Floats: decimal literals.
- Booleans: `true`, `false`.
- Strings: double-quoted UTF-8.
- Chars: single-quoted Unicode scalar values.

Ranges and loops:

- `a..b` (half-open) is the only guaranteed range form in v0.1.

Pattern baseline:

- Literal patterns.
- Wildcard `_`.
- Enum variant patterns with simple payload binding.
- Advanced pattern features are deferred.

### 6.3 Full conceptual syntax (planned)

The full language syntax is expected to include the following categories over time:

- Modules and visibility declarations.
- Type aliases, trait-like bounds, and richer generic constraints.
- Method receiver forms including mutable receivers when mutable borrowing is enabled.
- Extended literal syntax (numeric bases, suffixes, and separators).
- Expanded range syntax (inclusive and open-ended forms).
- Rich patterns (destructuring, nested patterns, OR-patterns, guards).
- Additional control-flow constructs and expression forms as needed by async/arena ergonomics.
- Attribute/decorator surface for advisory and trusted features under explicit rules.

Compatibility rule:

- New syntax must be additive and must not reinterpret valid v0.1 programs.

---

## 7. Control Flow

### 7.1. If
```
if cond { ... } else { ... }
```

### 7.2. While
```
while i < 10 { ... }
```

### 7.3. For
```
for i in 0..10 { ... }
```

### 7.4. Match
```
match result {
    Ok(v) => ...
    Err(e) => ...
}
```

---

## 8. Error Handling

Axis uses `Result<T, E>` and `Option<T>` with `?`.

```
fn load_config(p: &str) -> Result<Config, IoError> {
    let txt = fs::read_to_string(p)?;
    let cfg = parse_config(&txt)?;
    Ok(cfg)
}
```

### 8.1 `?` conversion rules (v0.1)

In v0.1, `?` uses exact error-type matching by default.

- For `Result<T, E>`, `?` requires `E` to match the enclosing function/task error type.
- No implicit error conversion is performed by `?` in v0.1.
- Explicit adapters (for example `map_err(...)`) may be used before `?`.

Option interaction:

- `?` on `Option<T>` is valid in `Option`-returning contexts.
- No implicit `Option` to `Result` lifting is performed in v0.1.
- Converting `Option` to `Result` requires explicit adapters (for example `ok_or(...)`).

Forward-compatibility note:

- Future conversion traits/protocols may be added as an additive extension.
- Valid v0.1 `?` behavior must remain stable.

---

## 9. Ownership & Borrowing

- Move semantics by default.
- References: `&T` in safe v0.1, mut refs planned: `&mut T`.

Borrow checker (planned) enforces:
- 1 mutable borrow OR
- many immutable borrows.

v0.1 note:
- Safe v0.1 supports shared immutable borrowing only.
- Exclusive mutable borrowing (`&mut T`) is deferred until aliasing rules are fully specified.

### 9.1 Ownership roadmap (v0.1)

Axis v0.1 adopts a conservative ownership model intended to remain compatible with later C-stage borrowing.

- Non-trivial values move by default.
- Using a moved value is a compile-time error.
- Implicit copy in v0.1 is limited to primitive scalars and `bool`.
- Non-scalar aggregates are move-only unless explicitly duplicated through library APIs.

v0.1 simplifications:
- Partial moves from structs/tuples are not supported.
- Lexical borrow scopes are sufficient for the first compiler.
- Future non-lexical borrow analysis may relax rejections but must not change the meaning of valid v0.1 programs.

Forward-compatibility note:
- Mutable borrowing and richer ownership features are additive extensions and must preserve existing v0.1 behavior.

---

## 10. Arenas (planned but core concept)

```
fn process() -> Result<(), Error> {
    arena frame {
        let buf = frame.alloc_array<f32>(1024);
        let mat = frame.alloc(Matrix::zero());
    }
    Ok(())
}
```

Values allocated in arenas cannot escape their scope unless promoted.

### 10.1 Arena value model (v0.1 design target)

Arena allocation is modeled as an explicit region-typed reference:

- `frame.alloc(T)` returns `ArenaRef<'frame, T>`.
- `ArenaRef<'a, T>` is read-only in safe v0.1.
- Mutable arena references are deferred until mutable borrowing is available.

Lifetime and escape behavior:

- `ArenaRef<'a, T>` is tied to region lifetime `'a`.
- Values containing `ArenaRef<'a, _>` cannot escape `'a` without an explicit promotion step.
- Escaping references or containers with embedded arena refs is a compile-time error unless promoted.

Interaction constraints:

- Arena-allocated values may contain owned data and arena references that are valid for at least `'a`.
- Arena-allocated values must not contain short-lived stack borrows that could dangle.
- Non-arena values may read through `ArenaRef` while in scope but may not persist arena-backed references beyond `'a`.

Teardown and drop policy:

- Arena scope exit performs bulk reclamation.
- v0.1 assumes trivial-drop arena payloads.
- Non-trivial destructor behavior inside arenas is deferred until destructor policy is specified.

### 10.2 Arena promotion semantics (v0.1 design target)

Promotion is the explicit operation that rebinds arena-backed data from a source arena lifetime to a destination arena lifetime.

Promotion rules:

- Promotion is always explicit. No implicit promotion is performed in assignment, return, or argument passing.
- Promotion is deep in v0.1. The promoted result must not retain references into the source arena.
- Promotion is type-checked. If any reachable component cannot be safely rebound to the destination arena, compilation fails.
- Promotion is cost-visible. Promotion may allocate and copy data and should be treated as a non-trivial operation.

Lifetime and direction constraints:

- The promoted value is tied to the destination arena lifetime.
- Promotion must not create references that outlive their valid region.
- Arena-to-arena promotion must preserve lifetime soundness; values cannot be promoted into arenas that cannot satisfy required lifetimes.

Escape behavior:

- Arena-backed values cannot escape their source arena unless explicitly promoted.
- Returning or storing arena-backed values beyond source lifetime requires explicit promotion.

v0.1 note:
- Arena syntax and semantics are design targets but are not yet part of the guaranteed safe core.
- Arena promotion rules will be specified before arenas are enabled in safe code.

---

## 11. Async & Tasks (planned)

### Distinction:
- `fn` → synchronous
- `task` → asynchronous, can `await`

```
task fetch(url: &str) -> Result<Response, NetError> {
    let res = await http::get(url)?;
    Ok(res)
}
```

```
fn main() -> Result<(), AppError> {
    runtime::block_on(main_task())
}
```

v0.1 note:
- If tasks are available in early implementations, they should operate on owned inputs/outputs.
- Borrowed captures that survive `await` are deferred until borrow and lifetime rules for async are finalized.

### 11.1 Structured task lifecycle (v0.1 design target)

Structured task rules:

- Tasks form a strict parent-child tree.
- Child tasks cannot outlive the parent scope that spawned them.
- Child spawning is explicit and scoped.
- Parent scope must observe deterministic completion of children before exit (join or configured cancellation path).

Parent exit behavior:

- If a parent scope exits early due to its own failure, remaining children in the scope are cancelled.
- Cancellation is cooperative and applies to descendants in the cancelled subtree.

Child failure behavior:

- Child failure handling is policy-configurable at the task scope.
- Default policy is fail-fast: a child error fails the parent scope and triggers sibling cancellation.
- Non-fail-fast policies are allowed, for example when child outcomes are modeled as optional or non-fatal results.
- Under non-fail-fast policy, sibling tasks continue running and all children are joined before the parent continues.

v0.1 capture constraints:

- Child tasks in safe v0.1 use owned captures only.
- Borrowed captures from parent stack or arena are deferred until async lifetime rules are finalized.

### 11.2 Await model (v0.1 design target)

Core semantics:

- `await expr` is an expression and is valid anywhere expressions are valid in `task` contexts.
- `await` in synchronous `fn` is a compile-time error.
- Synchronous entry to async execution remains explicit (for example `runtime::block_on(...)`).

Task execution model:

- Calling a `task` function produces a task value.
- Task execution is activated by explicit `await` or explicit scoped spawn.
- No implicit detached/background task execution occurs in safe v0.1.

Typing:

- If `expr` has task output type `Task<T>`, `await expr` has type `T`.
- Error composition through `Result` follows standard expression typing and `?` rules.

Policy interaction:

- Awaited child tasks participate in the structured cancellation and failure policies of their parent task scope.
- Fail-fast and non-fail-fast behavior is determined by the configured scope policy.

v0.1 capture constraints:

- Awaited tasks in safe v0.1 use owned captures only.
- Borrowed captures across suspension points are deferred until async lifetime rules are finalized.

---

## 12. Decorators

### Performance hints
- `@simd`
- `@parallel_for`
- `@gpu_kernel`

### Trusted low-level code
```
@trusted_aliasing("explanation")
fn copy_nonoverlapping(...) { ... }
```

### 12.1 Decorator contract (v0.1)

Decorator categories:

- Advisory performance decorators: `@simd`, `@parallel_for`, `@gpu_kernel`.
- Trusted decorators: `@trusted_aliasing(...)`.

Advisory decorator semantics:

- Advisory decorators express optimization intent.
- They must not change observable semantics of safe code.
- Backends may honor, ignore, or reject advisory decorators based on target capability and compilation mode.
- When advisory decorators are not applied, compilers should emit clear diagnostics.

Strictness mode:

- Default mode permits advisory decorators to be best-effort.
- Optional strict-performance mode may treat unmet advisory decorators as compile errors.

Trusted decorator semantics:

- Trusted decorators are outside the safe v0.1 core.
- `@trusted_aliasing` introduces an explicit trusted boundary with programmer proof obligations.
- Misuse in trusted contexts may cause undefined behavior.

Portability rule:

- Unsupported targets must produce explicit diagnostics rather than silently changing semantics.

v0.1 note:
- Trusted aliasing features are outside the safe v0.1 core.
- Any future trusted/unsafe surface must be explicit and isolated from safe defaults.

---

## 13. Standard Library (minimal v0)

- `println`
- `fs::read_to_string`
- Basic math: `sqrt`, etc.
- Core types: `vecN`, `mat4x4`, `Result`, `Option`

---

## 14. Modules & Visibility (v0.1 design target)

Module mapping:

- Each source file defines one module.
- Directory hierarchy defines module path segments.
- Package root module is implicit at the entry root (application or library root source file).

Core declarations:

- `mod` declares child modules.
- `use` imports names into scope.
- `pub` exports items across module boundaries.
- Items are private by default.

Name resolution order:

- Lexical scope.
- Current module scope.
- Explicit imports.
- Prelude.

Prelude behavior:

- The standard prelude is in scope by default in v0.1.
- Prelude exposure includes core result/option types and common primitives.

Package boundaries:

- Cross-package access is explicit.
- Public API exposure is controlled through `pub` items and module exports.

Compatibility rule:

- Future namespace features must be additive and must not reinterpret valid v0.1 paths or visibility semantics.

---

*End of Draft 0.1 Spec*
