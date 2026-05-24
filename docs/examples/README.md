# Axis Examples

These examples are intended to run with the native LLVM path and executable emission.

## Compile and run one example

```bash
AXIS_LLVM_BACKEND=native cargo run --features llvm-native -- --emit-exe target/examples/literal_return docs/examples/literal_return.axis
./target/examples/literal_return
echo $?
```

## Compile and run all examples

```bash
mkdir -p target/examples
for file in docs/examples/*.axis; do
  name="$(basename "$file" .axis)"
  exe="target/examples/$name"
  AXIS_LLVM_BACKEND=native cargo run --features llvm-native -- --emit-exe "$exe" "$file"
  "$exe"
  echo "$name => exit=$?"
done
```