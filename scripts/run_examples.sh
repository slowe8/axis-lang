#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mkdir -p target/examples
failed=0

for file in docs/examples/*.axis; do
  name="$(basename "$file" .axis)"
  exe="target/examples/$name"

  echo "-- $name --"
  if ! AXIS_LLVM_BACKEND=native cargo run --features llvm-native -- --emit-exe "$exe" "$file"; then
    echo "$name => build failed"
    failed=1
    echo
    continue
  fi

  set +e
  "$exe"
  code=$?
  set -e

  echo "$name => exit=$code"
  echo
done

exit "$failed"
