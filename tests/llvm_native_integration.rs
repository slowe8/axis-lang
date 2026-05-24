#![cfg(feature = "llvm-native")]

use axis_lang::backend::{
    emit_native_executable_file, emit_native_object_file, lower_ssa_to_llvm_ir_with_preference_and_types,
    LlvmAdapterPreference, LlvmModuleArtifact,
};
use axis_lang::passes::{pass_manager_with_config, PassContext, PassProfile, PipelineConfig, PipelineState};
use axis_lang::resolution::ModulePath;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn vendored_clang_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("third_party/llvm-build/bin/clang")
}

fn llvm_toolchain_available() -> bool {
    vendored_clang_path().is_file()
}

fn unique_temp_path(label: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);

    std::env::temp_dir().join(format!("axis-{label}-{nanos}.{extension}"))
}

fn compile_native_artifact(source: &str) -> LlvmModuleArtifact {
    let config = PipelineConfig {
        profile: PassProfile::Test,
        enable_resolve: true,
        enable_type_check: true,
        enable_lower: true,
        enable_ssa: true,
        enable_backend: false,
    };

    let mut state = PipelineState::new(source.to_string());
    let mut context = PassContext::new(ModulePath::root());
    let manager = pass_manager_with_config(config);
    manager.run(&mut state, &mut context).expect("pipeline should succeed");

    let ssa = state.ssa.as_ref().expect("ssa artifact should exist");
    let ssa_types = state.ssa_types.as_ref().expect("ssa types should exist");
    let types = state.backend_types.as_ref().expect("backend types should exist");

    lower_ssa_to_llvm_ir_with_preference_and_types(ssa, ssa_types, types, LlvmAdapterPreference::Native)
        .expect("native artifact generation should succeed")
}

fn run_executable(path: &Path) -> i32 {
    let status = Command::new(path)
        .status()
        .expect("executable should run successfully");
    status.code().expect("process exit code should be available")
}

#[test]
fn emits_object_file_from_native_artifact() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let artifact = compile_native_artifact("fn main() -> int { 7 }");
    let output_path = unique_temp_path("emit-object", "o");

    emit_native_object_file(&artifact, &output_path).expect("object emission should succeed");

    let metadata = fs::metadata(&output_path).expect("object file metadata should be readable");
    assert!(metadata.len() > 0, "object file should be non-empty");

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_executable_and_returns_expected_exit_code() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let artifact = compile_native_artifact("fn main() -> int { 9 }");
    let output_path = unique_temp_path("emit-executable", "bin");

    emit_native_executable_file(&artifact, &output_path).expect("executable emission should succeed");

    let exit = run_executable(&output_path);
    assert_eq!(exit, 9);

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_object_for_if_expression_regression_case() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { if true { 42 } else { 1 } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-if-object", "o");

    emit_native_object_file(&artifact, &output_path).expect("if-expression object emission should succeed");

    let metadata = fs::metadata(&output_path).expect("object file metadata should be readable");
    assert!(metadata.len() > 0, "if-expression object file should be non-empty");

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_object_for_match_expression_regression_case() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { match false { true => 3, false => 19, } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-match-object", "o");

    emit_native_object_file(&artifact, &output_path).expect("match-expression object emission should succeed");

    let metadata = fs::metadata(&output_path).expect("object file metadata should be readable");
    assert!(metadata.len() > 0, "match-expression object file should be non-empty");

    let _ = fs::remove_file(&output_path);
}
