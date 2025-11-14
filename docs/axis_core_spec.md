
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
Result Option
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

Operators: `+`, `-`, `*` for elementwise/matrix ops.

#### Generic matrices (planned)
```
Matrix<T, M, N>
Vector<T, N>
```

### 3.4. Result/Option
Primary error/optional system.

---

## 4. Bindings & Mutability

- `let` → immutable
- `var` → mutable

```
let x = 10;
var y = 0;
y += 1;
```

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

---

## 9. Ownership & Borrowing

- Move semantics by default.
- References: `&T`, mut refs planned: `&mut T`.

Borrow checker (planned) enforces:
- 1 mutable borrow OR
- many immutable borrows.

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

---

## 13. Standard Library (minimal v0)

- `println`
- `fs::read_to_string`
- Basic math: `sqrt`, etc.
- Core types: `vecN`, `mat4x4`, `Result`, `Option`

---

*End of Draft 0.1 Spec*
