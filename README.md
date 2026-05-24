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

Current implemented pipeline:
- Parse -> Resolve -> TypeCheck -> MIR Lowering -> SSA Scaffold -> LLVM adapter lowering
- Native object/executable emission via vendored LLVM clang (`--features llvm-native`)
- Runnable examples in `docs/examples` through `scripts/run_examples.sh`

## Prerequisites

- Rust toolchain (`cargo`)
- Build tools for vendored LLVM bootstrap: `cmake`, `ninja`, and a C/C++ compiler
- `git` (for submodule checkout)

## Build and Test

1. Build the compiler:
    - `cargo build`

2. Run default tests:
    - `cargo test`

3. Run llvm-native tests:
    - `cargo test --features llvm-native`

4. Run focused native integration suite:
    - `cargo test --features llvm-native --test llvm_native_integration -- --nocapture`

## Usage

CLI shape:
- `axis-lang [--profile dev|test|release] [--passes] [--ast] [--hir] [--typed] [--mir] [--ssa] [--llvm] [--emit-obj <path>] [--emit-exe <path>] <source-file>|-`

Common commands:

1. Compile a source file through the default pipeline summary output:
    - `cargo run -- path/to/program.axis`

2. Read source from stdin:
    - `echo 'fn main() -> int { 7 }' | cargo run -- -`

3. Dump pass log plus intermediate artifacts:
    - `cargo run -- --passes --ast --hir --typed --mir --ssa --llvm path/to/program.axis`

4. Show built-in CLI help:
    - `cargo run -- --help`

## Vendored LLVM Build

Object emission through the native backend uses a vendored LLVM checkout (git submodule) and a locally built clang binary.

1. Initialize submodule and build vendored clang:
    - `./scripts/bootstrap_llvm.sh`

2. Emit an object file from stdin Axis source:
    - `echo 'fn main() -> int { 7 }' | AXIS_LLVM_BACKEND=native cargo run --features llvm-native -- --emit-obj target/tmp/main.o -`

3. Emit a native executable from stdin Axis source:
    - `echo 'fn main() -> int { 9 }' | AXIS_LLVM_BACKEND=native cargo run --features llvm-native -- --emit-exe target/tmp/main -`

4. Run emitted executable:
    - `./target/tmp/main; echo $?`

By default, object emission expects clang at `third_party/llvm-build/bin/clang`. Override with `AXIS_LLVM_CLANG=/custom/path/to/clang` if needed.

## Run Examples

Run all examples end-to-end (compile + execute):
- `./scripts/run_examples.sh`

Run a single example manually:
- `AXIS_LLVM_BACKEND=native cargo run --features llvm-native -- --emit-exe target/examples/match_topic docs/examples/match_topic.axis`
- `./target/examples/match_topic; echo $?`

See [docs/axis_core_spec.md](docs/axis_core_spec.md) for the full language specification.

## License

See [LICENSE](LICENSE) for details.
