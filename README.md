# Axis

> A Rust-flavored, no-GC systems language with first-class linear algebra types, arenas, and structured async.

## Overview

Axis is a modern systems programming language designed for high-performance numerical computing and safe low-level programming. It combines the safety guarantees of Rust with built-in support for linear algebra and arena-based memory management.

## Key Features

- **Safe by default** – Ownership and borrowing with compile-time checks, no data races or use-after-free
- **No garbage collector** – Predictable performance with direct control over memory
- **Built-in linear algebra** – Native `vec4`, `mat4x4`, and other matrix/vector types with SIMD support
- **Arena memory management** – First-class arena/region support for efficient scratch allocation
- **Modern async** – Clear `fn` vs `task` distinction with structured concurrency
- **Result-based errors** – No exceptions, explicit error handling with `Result<T, E>`

## Quick Examples

### Basic function
```axis
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Linear algebra
```axis
let v: vec4<f32> = [1.0, 2.0, 3.0, 4.0];
let m: mat4x4<f32> = mat4x4::identity();
let result = m * v;
```

### Error handling
```axis
fn load_config(p: &str) -> Result<Config, IoError> {
    let txt = fs::read_to_string(p)?;
    let cfg = parse_config(&txt)?;
    Ok(cfg)
}
```

### Arena allocation
```axis
fn process() -> Result<(), Error> {
    arena frame {
        let buf = frame.alloc_array<f32>(1024);
        let mat = frame.alloc(Matrix::zero());
    }
    Ok(())
}
```

### Async tasks
```axis
task fetch(url: &str) -> Result<Response, NetError> {
    let res = await http::get(url)?;
    Ok(res)
}
```

## Type System

- **Primitives**: `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `bool`
- **Collections**: Arrays `[T; N]`, tuples `(T, U)`
- **User types**: `struct`, `enum`
- **Linear algebra**: `vec4<T>`, `mat4x4<T>`, etc.
- **Error types**: `Result<T, E>`, `Option<T>`

## Performance Decorators

Axis provides hints for optimization:
- `@simd` – SIMD vectorization
- `@parallel_for` – Parallel loop execution
- `@gpu_kernel` – GPU computation

## Status

Axis is in early development (Draft 0.1). The specification is evolving and many features are planned but not yet implemented.

See [docs/axis_core_spec.md](docs/axis_core_spec.md) for the full language specification.

## License

See [LICENSE](LICENSE) for details.
