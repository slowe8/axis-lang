#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
submodule_path="$repo_root/third_party/llvm-project"
build_path="$repo_root/third_party/llvm-build"

if [[ ! -d "$repo_root/.git" ]]; then
  echo "error: expected git repository at $repo_root" >&2
  exit 1
fi

cd "$repo_root"
git submodule update --init --recursive third_party/llvm-project

cmake -S "$submodule_path/llvm" -B "$build_path" -G Ninja \
  -DLLVM_ENABLE_PROJECTS=clang \
  -DCMAKE_BUILD_TYPE=Release \
  -DLLVM_TARGETS_TO_BUILD=Native \
  -DLLVM_INCLUDE_TESTS=OFF \
  -DLLVM_INCLUDE_BENCHMARKS=OFF \
  -DLLVM_INCLUDE_EXAMPLES=OFF

cmake --build "$build_path" --target clang -- -j"$(nproc)"

echo "Built vendored clang at: $build_path/bin/clang"
