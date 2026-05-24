#![cfg(feature = "llvm-native")]

use axis_lang::backend::{
    emit_native_executable_file, emit_native_object_file, lower_ssa_to_llvm_ir_with_preference_and_types,
    BackendLoweringError, LlvmAdapterPreference, LlvmModuleArtifact,
};
use axis_lang::hir::SymbolId;
use axis_lang::mir::{
    LoweredSsaProgram, MirPlace, SsaBasicBlock, SsaName, SsaStatement, SsaStatementKind, SsaTerminator, SsaTypeMap,
    SsaValue,
};
use axis_lang::passes::{pass_manager_with_config, PassContext, PassProfile, PipelineConfig, PipelineState};
use axis_lang::resolution::ModulePath;
use axis_lang::types::TypeStore;
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

#[test]
fn emits_executable_for_non_literal_bool_match() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { let mood = false; match mood { true => 3, false => 19, } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-match-bool-executable", "bin");

    emit_native_executable_file(&artifact, &output_path)
        .expect("non-literal bool match executable emission should succeed");

    let exit = run_executable(&output_path);
    assert_eq!(exit, 19);

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_executable_for_non_literal_int_match() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { let code = 2; match code { 1 => 10, 2 => 22, _ => 0, } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-match-int-executable", "bin");

    emit_native_executable_file(&artifact, &output_path)
        .expect("non-literal int match executable emission should succeed");

    let exit = run_executable(&output_path);
    assert_eq!(exit, 22);

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_executable_for_non_literal_char_match() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { let grade = 'B'; match grade { 'A' => 100, 'B' => 85, _ => 0, } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-match-char-executable", "bin");

    emit_native_executable_file(&artifact, &output_path)
        .expect("non-literal char match executable emission should succeed");

    let exit = run_executable(&output_path);
    assert_eq!(exit, 85);

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_executable_for_non_literal_string_match() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { let mood = \"party\"; match mood { \"work\" => 2, \"party\" => 42, _ => 0, } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-match-string-executable", "bin");

    emit_native_executable_file(&artifact, &output_path)
        .expect("non-literal string match executable emission should succeed");

    let exit = run_executable(&output_path);
    assert_eq!(exit, 42);

    let _ = fs::remove_file(&output_path);
}

#[test]
fn emits_executable_for_non_literal_string_match_fallback() {
    if !llvm_toolchain_available() {
        eprintln!("skipping: vendored clang is not built at {}", vendored_clang_path().display());
        return;
    }

    let source = "fn main() -> int { let mood = \"rest\"; match mood { \"work\" => 2, \"party\" => 42, _ => 7, } }";
    let artifact = compile_native_artifact(source);
    let output_path = unique_temp_path("emit-match-string-fallback-executable", "bin");

    emit_native_executable_file(&artifact, &output_path)
        .expect("non-literal string match fallback executable emission should succeed");

    let exit = run_executable(&output_path);
    assert_eq!(exit, 7);

    let _ = fs::remove_file(&output_path);
}

#[test]
fn rejects_mistyped_compare_before_native_llvm_emission() {
    let types = TypeStore::new();
    let mut ssa_types = SsaTypeMap::default();
    let bool_type = types.named("bool").expect("bool type should exist");
    let int_type = types.named("int").expect("int type should exist");

    let target = SsaName {
        place: MirPlace::Local(Some(SymbolId(90))),
        version: 1,
    };
    let int_name = SsaName {
        place: MirPlace::Local(Some(SymbolId(91))),
        version: 1,
    };
    let _ = ssa_types.insert_name_type(&target, bool_type);
    let _ = ssa_types.insert_name_type(&int_name, int_type);

    let ssa = LoweredSsaProgram {
        blocks: vec![SsaBasicBlock {
            id: 0,
            phis: Vec::new(),
            statements: vec![SsaStatement {
                id: 0,
                kind: SsaStatementKind::Assign {
                    target,
                    value: SsaValue::CompareEqString {
                        lhs: Box::new(SsaValue::Name(int_name)),
                        rhs: "topic".to_string(),
                    },
                },
            }],
            terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
        }],
        value_count: 1,
    };

    let error = lower_ssa_to_llvm_ir_with_preference_and_types(&ssa, &ssa_types, &types, LlvmAdapterPreference::Native)
        .expect_err("mistyped compare should fail in backend contract before llvm emission");
    assert_eq!(
        error,
        BackendLoweringError::AssignTypeMismatch {
            block: 0,
            statement: 0,
        }
    );
}
