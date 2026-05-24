use crate::backend_contract::validate_text_backend_subset;
#[cfg(feature = "llvm-native")]
use crate::backend_native::{emit_executable_from_ir, emit_object_from_ir, render_native_llvm_ir};
use crate::mir::{LoweredSsaProgram, MirPlace, SsaTerminator, SsaTypeMap, SsaValue};
use crate::types::TypeStore;
use std::path::Path;
use std::fmt;

pub fn initialize() {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlvmModuleArtifact {
	pub module_name: String,
	pub ir_text: String,
	pub function_count: usize,
	pub block_count: usize,
	pub value_count: usize,
}

pub trait LlvmBackendAdapter {
	fn name(&self) -> &'static str;
	fn emit_module(&self, input: BackendInput<'_>) -> Result<LlvmModuleArtifact, BackendLoweringError>;
}

#[derive(Debug, Clone, Copy)]
pub struct BackendInput<'a> {
	pub ssa: &'a LoweredSsaProgram,
	pub ssa_types: &'a SsaTypeMap,
	pub types: &'a TypeStore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendLoweringError {
	NativeAdapterUnavailable,
	UnsupportedVoidReturn { block: usize },
	UnsupportedPhiIncomingValue { block: usize },
	UnsupportedAssignValue { block: usize, statement: usize },
	UnsupportedEvalValue { block: usize, statement: usize },
	UnsupportedReturnValue { block: usize },
	UnsupportedBranchCondition { block: usize },
	PhiTypeMismatch { block: usize },
	AssignTypeMismatch { block: usize, statement: usize },
	ReturnTypeMismatch { block: usize },
	BranchConditionTypeMismatch { block: usize },
	ObjectEmissionUnavailable,
	ExecutableEmissionUnavailable,
	ToolchainInvocationFailed { tool: String, message: String },
}

impl fmt::Display for BackendLoweringError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			BackendLoweringError::NativeAdapterUnavailable => {
				write!(f, "native llvm adapter requested but feature llvm-native is not enabled")
			}
			BackendLoweringError::UnsupportedVoidReturn { block } => {
				write!(f, "void return is not supported in i64 entry function in block {block}")
			}
			BackendLoweringError::UnsupportedPhiIncomingValue { block } => {
				write!(f, "unsupported phi incoming SSA value in block {block}")
			}
			BackendLoweringError::UnsupportedAssignValue { block, statement } => {
				write!(f, "unsupported assign SSA value in block {block}, statement {statement}")
			}
			BackendLoweringError::UnsupportedEvalValue { block, statement } => {
				write!(f, "unsupported eval SSA value in block {block}, statement {statement}")
			}
			BackendLoweringError::UnsupportedReturnValue { block } => {
				write!(f, "unsupported return SSA value in block {block}")
			}
			BackendLoweringError::UnsupportedBranchCondition { block } => {
				write!(f, "unsupported branch condition SSA value in block {block}")
			}
			BackendLoweringError::PhiTypeMismatch { block } => {
				write!(f, "phi incoming type mismatch in block {block}")
			}
			BackendLoweringError::AssignTypeMismatch { block, statement } => {
				write!(f, "assign type mismatch in block {block}, statement {statement}")
			}
			BackendLoweringError::ReturnTypeMismatch { block } => {
				write!(f, "return type mismatch in block {block}")
			}
			BackendLoweringError::BranchConditionTypeMismatch { block } => {
				write!(f, "branch condition type mismatch: expected bool in block {block}")
			}
			BackendLoweringError::ObjectEmissionUnavailable => {
				write!(f, "object emission requires llvm-native feature")
			}
			BackendLoweringError::ExecutableEmissionUnavailable => {
				write!(f, "executable emission requires llvm-native feature")
			}
			BackendLoweringError::ToolchainInvocationFailed { tool, message } => {
				write!(f, "{tool} invocation failed: {message}")
			}
		}
	}
}

#[cfg(feature = "llvm-native")]
pub fn emit_native_object_file(artifact: &LlvmModuleArtifact, output_path: &Path) -> Result<(), BackendLoweringError> {
	emit_object_from_ir(&artifact.ir_text, output_path)
		.map_err(|message| BackendLoweringError::ToolchainInvocationFailed {
			tool: "vendored llvm clang".to_string(),
			message,
		})
}

#[cfg(feature = "llvm-native")]
pub fn emit_native_executable_file(artifact: &LlvmModuleArtifact, output_path: &Path) -> Result<(), BackendLoweringError> {
	emit_executable_from_ir(&artifact.ir_text, output_path)
		.map_err(|message| BackendLoweringError::ToolchainInvocationFailed {
			tool: "vendored llvm clang".to_string(),
			message,
		})
}

#[cfg(not(feature = "llvm-native"))]
pub fn emit_native_object_file(_artifact: &LlvmModuleArtifact, _output_path: &Path) -> Result<(), BackendLoweringError> {
	Err(BackendLoweringError::ObjectEmissionUnavailable)
}

#[cfg(not(feature = "llvm-native"))]
pub fn emit_native_executable_file(_artifact: &LlvmModuleArtifact, _output_path: &Path) -> Result<(), BackendLoweringError> {
	Err(BackendLoweringError::ExecutableEmissionUnavailable)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlvmAdapterPreference {
	Auto,
	Text,
	Native,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TextLlvmAdapter;

impl LlvmBackendAdapter for TextLlvmAdapter {
	fn name(&self) -> &'static str {
		"text-llvm-adapter"
	}

	fn emit_module(&self, input: BackendInput<'_>) -> Result<LlvmModuleArtifact, BackendLoweringError> {
		emit_textual_llvm(input)
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NativeLlvmAdapter;

impl LlvmBackendAdapter for NativeLlvmAdapter {
	fn name(&self) -> &'static str {
		"native-llvm-adapter"
	}

	fn emit_module(&self, input: BackendInput<'_>) -> Result<LlvmModuleArtifact, BackendLoweringError> {
		emit_native_llvm_placeholder(input)
	}
}

pub fn lower_ssa_to_llvm_ir(ssa: &LoweredSsaProgram) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	let default_types = TypeStore::new();
	let default_ssa_types = SsaTypeMap::default();
	lower_ssa_to_llvm_ir_with_types(ssa, &default_ssa_types, &default_types)
}

pub fn lower_ssa_to_llvm_ir_with_types(
	ssa: &LoweredSsaProgram,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	lower_ssa_to_llvm_ir_with_preference_and_types(ssa, ssa_types, types, LlvmAdapterPreference::Auto)
}

pub fn lower_ssa_to_llvm_ir_with_preference(
	ssa: &LoweredSsaProgram,
	preference: LlvmAdapterPreference,
	) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	let default_types = TypeStore::new();
	let default_ssa_types = SsaTypeMap::default();
	lower_ssa_to_llvm_ir_with_preference_and_types(ssa, &default_ssa_types, &default_types, preference)
}

pub fn lower_ssa_to_llvm_ir_with_preference_and_types(
	ssa: &LoweredSsaProgram,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
	preference: LlvmAdapterPreference,
) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	let input = BackendInput {
		ssa,
		ssa_types,
		types,
	};

	match resolve_adapter(preference)? {
		AdapterSelection::Text => TextLlvmAdapter.emit_module(input),
		AdapterSelection::Native => NativeLlvmAdapter.emit_module(input),
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdapterSelection {
	Text,
	Native,
}

fn resolve_adapter(preference: LlvmAdapterPreference) -> Result<AdapterSelection, BackendLoweringError> {
	match preference {
		LlvmAdapterPreference::Text => Ok(AdapterSelection::Text),
		LlvmAdapterPreference::Native => {
			if cfg!(feature = "llvm-native") {
				Ok(AdapterSelection::Native)
			} else {
				Err(BackendLoweringError::NativeAdapterUnavailable)
			}
		}
		LlvmAdapterPreference::Auto => match std::env::var("AXIS_LLVM_BACKEND") {
			Ok(value) if value.eq_ignore_ascii_case("native") => {
				if cfg!(feature = "llvm-native") {
					Ok(AdapterSelection::Native)
				} else {
					Err(BackendLoweringError::NativeAdapterUnavailable)
				}
			}
			_ => Ok(AdapterSelection::Text),
		},
	}
}

fn emit_textual_llvm(input: BackendInput<'_>) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	validate_text_backend_subset(input.ssa, input.ssa_types, input.types)?;

	let ssa = input.ssa;

	let function_count = if ssa.blocks.is_empty() { 0 } else { 1 };
	let block_count = ssa.blocks.len();

	let mut ir_text = String::new();
	ir_text.push_str("; axis-lang llvm scaffold\n");
	ir_text.push_str(&format!("; functions: {function_count}\n"));
	ir_text.push_str(&format!("; blocks: {block_count}\n"));
	ir_text.push_str(&format!("; values: {}\n\n", ssa.value_count));

	if function_count == 0 {
		ir_text.push_str("; no functions emitted\n");
	} else {
		ir_text.push_str("define i64 @axis_main() {\n");
		for block in &ssa.blocks {
			ir_text.push_str(&format!("bb{}:\n", block.id));

			for phi in &block.phis {
				ir_text.push_str(&format!(
					"  ; phi {} <- {}\n",
					render_name(&phi.target),
					phi.incoming
						.iter()
						.map(|incoming| format!("bb{}:{}", incoming.block, render_ssa_value(&incoming.value)))
						.collect::<Vec<_>>()
						.join(", ")
				));
			}

			for statement in &block.statements {
				match &statement.kind {
					crate::mir::SsaStatementKind::Assign { target, value } => {
						if let Some(line) = emit_assign_line(target, value) {
							ir_text.push_str(&format!("  {line}\n"));
						} else {
							ir_text.push_str(&format!(
								"  ; {} = {}\n",
								render_name(target),
								render_ssa_value(value)
							));
						}
					}
					crate::mir::SsaStatementKind::Eval(value) => {
						ir_text.push_str(&format!("  ; eval {}\n", render_ssa_value(value)));
					}
				}
			}

			ir_text.push_str(&format!("  {}\n", render_terminator(&block.terminator)));
		}
		ir_text.push_str("}\n");
	}

	Ok(LlvmModuleArtifact {
		module_name: "axis.main".to_string(),
		ir_text,
		function_count,
		block_count,
		value_count: ssa.value_count,
	})
}

#[cfg(feature = "llvm-native")]
fn emit_native_llvm_placeholder(input: BackendInput<'_>) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	validate_text_backend_subset(input.ssa, input.ssa_types, input.types)?;

	Ok(LlvmModuleArtifact {
		module_name: "axis.main.native".to_string(),
		ir_text: render_native_llvm_ir(input.ssa, input.ssa_types, input.types),
		function_count: if input.ssa.blocks.is_empty() { 0 } else { 1 },
		block_count: input.ssa.blocks.len(),
		value_count: input.ssa.value_count,
	})
}

#[cfg(not(feature = "llvm-native"))]
fn emit_native_llvm_placeholder(_input: BackendInput<'_>) -> Result<LlvmModuleArtifact, BackendLoweringError> {
	Err(BackendLoweringError::NativeAdapterUnavailable)
}

fn emit_assign_line(target: &crate::mir::SsaName, value: &SsaValue) -> Option<String> {
	let target = render_name(target);
	match value {
		SsaValue::Integer(value) => Some(format!("{target} = add i64 0, {value}")),
		SsaValue::Name(name) => Some(format!("{target} = add i64 0, {}", render_name(name))),
		SsaValue::Boolean(value) => Some(format!("{target} = zext i1 {} to i64", if *value { "true" } else { "false" })),
		_ => None,
	}
}

fn render_terminator(terminator: &SsaTerminator) -> String {
	match terminator {
		SsaTerminator::Return(Some(value)) => format!("ret i64 {}", render_ssa_i64_operand(value)),
		SsaTerminator::Return(None) => "ret void".to_string(),
		SsaTerminator::Goto(target) => format!("br label %bb{target}"),
		SsaTerminator::Branch {
			condition,
			then_block,
			else_block,
		} => format!(
			"br i1 {}, label %bb{}, label %bb{}",
			render_ssa_i1_operand(condition),
			then_block,
			else_block
		),
	}
}

fn render_name(name: &crate::mir::SsaName) -> String {
	match &name.place {
		MirPlace::Local(symbol) => match symbol {
			Some(symbol) => format!("%local_{}_v{}", symbol.0, name.version),
			None => format!("%local_none_v{}", name.version),
		},
		MirPlace::Temp(temp) => format!("%tmp_{}_v{}", temp, name.version),
	}
}

fn render_ssa_i64_operand(value: &SsaValue) -> String {
	match value {
		SsaValue::Integer(value) => value.to_string(),
		SsaValue::Boolean(value) => {
			if *value {
				"1".to_string()
			} else {
				"0".to_string()
			}
		}
		SsaValue::Name(name) => render_name(name),
		_ => "0".to_string(),
	}
}

fn render_ssa_i1_operand(value: &SsaValue) -> String {
	match value {
		SsaValue::Boolean(value) => {
			if *value {
				"true".to_string()
			} else {
				"false".to_string()
			}
		}
		SsaValue::Name(name) => render_name(name),
		SsaValue::Integer(value) => {
			if *value == 0 {
				"false".to_string()
			} else {
				"true".to_string()
			}
		}
		_ => "false".to_string(),
	}
}

fn render_ssa_value(value: &SsaValue) -> String {
	match value {
		SsaValue::Unit => "unit".to_string(),
		SsaValue::Integer(value) => value.to_string(),
		SsaValue::Float(value) => value.clone(),
		SsaValue::Boolean(value) => value.to_string(),
		SsaValue::String(value) => format!("\"{}\"", value),
		SsaValue::Char(value) => format!("'{}'", value),
		SsaValue::CompareEqInt { lhs, rhs } => {
			format!("cmp_eq_int({}, {})", render_ssa_value(lhs), rhs)
		}
		SsaValue::CompareEqChar { lhs, rhs } => {
			format!("cmp_eq_char({}, '{}')", render_ssa_value(lhs), rhs)
		}
		SsaValue::CompareEqString { lhs, rhs } => {
			format!("cmp_eq_string({}, \"{}\")", render_ssa_value(lhs), rhs)
		}
		SsaValue::Name(name) => render_name(name),
		SsaValue::OpaqueExpr => "<opaque-expr>".to_string(),
		SsaValue::UnresolvedPlace(place) => format!("<unresolved:{place:?}>"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mir::{
		LoweredSsaProgram, SsaBasicBlock, SsaName, SsaPhi, SsaPhiIncoming, SsaStatement, SsaStatementKind,
		SsaTerminator, SsaValue,
	};

	#[test]
	fn lower_ssa_to_llvm_ir_emits_stub_text_with_counts() {
		let ssa = LoweredSsaProgram {
			blocks: Vec::new(),
			value_count: 3,
		};

		let artifact = lower_ssa_to_llvm_ir(&ssa).expect("lowering should succeed");
		assert_eq!(artifact.function_count, 0);
		assert_eq!(artifact.block_count, 0);
		assert_eq!(artifact.value_count, 3);
		assert!(artifact.ir_text.contains("axis-lang llvm scaffold"));
		assert!(!artifact.ir_text.contains("define i64 @axis_main()"));
	}

	#[test]
	fn lower_ssa_to_llvm_ir_emits_blocks_and_terminators() {
		let ssa = LoweredSsaProgram {
			blocks: vec![
				SsaBasicBlock {
					id: 0,
					phis: vec![SsaPhi {
						target: SsaName {
							place: MirPlace::Temp(1),
							version: 2,
						},
						incoming: vec![SsaPhiIncoming {
							block: 1,
							value: SsaValue::Integer(3),
						}],
					}],
					statements: vec![SsaStatement {
						id: 0,
						kind: SsaStatementKind::Assign {
							target: SsaName {
								place: MirPlace::Local(None),
								version: 1,
							},
							value: SsaValue::Integer(7),
						},
					}],
					terminator: SsaTerminator::Goto(1),
				},
				SsaBasicBlock {
					id: 1,
					phis: Vec::new(),
					statements: Vec::new(),
					terminator: SsaTerminator::Return(Some(SsaValue::Name(SsaName {
						place: MirPlace::Local(None),
						version: 1,
					}))),
				},
			],
			value_count: 2,
		};

		let artifact = lower_ssa_to_llvm_ir(&ssa).expect("lowering should succeed");
		assert!(artifact.ir_text.contains("define i64 @axis_main()"));
		assert!(artifact.ir_text.contains("bb0:"));
		assert!(artifact.ir_text.contains("br label %bb1"));
		assert!(artifact.ir_text.contains("ret i64 %local_none_v1"));
		assert!(artifact.ir_text.contains("%local_none_v1 = add i64 0, 7"));
	}

	#[cfg(feature = "llvm-native")]
	#[test]
	fn native_preference_emits_native_adapter_banner() {
		let ssa = LoweredSsaProgram {
			blocks: Vec::new(),
			value_count: 0,
		};

		let artifact = lower_ssa_to_llvm_ir_with_preference(&ssa, LlvmAdapterPreference::Native)
			.expect("native preference should emit scaffold when available");
		assert!(artifact.module_name.contains("native"));
		assert!(artifact.ir_text.contains("native llvm adapter"));
	}

	#[cfg(feature = "llvm-native")]
	#[test]
	fn native_renderer_emits_phi_instruction() {
		let ssa = LoweredSsaProgram {
			blocks: vec![
				SsaBasicBlock {
					id: 0,
					phis: vec![SsaPhi {
						target: SsaName {
							place: MirPlace::Temp(1),
							version: 1,
						},
						incoming: vec![
							SsaPhiIncoming {
								block: 1,
								value: SsaValue::Integer(1),
							},
							SsaPhiIncoming {
								block: 2,
								value: SsaValue::Integer(2),
							},
						],
					}],
					statements: Vec::new(),
					terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
				},
				SsaBasicBlock {
					id: 1,
					phis: Vec::new(),
					statements: Vec::new(),
					terminator: SsaTerminator::Goto(0),
				},
				SsaBasicBlock {
					id: 2,
					phis: Vec::new(),
					statements: Vec::new(),
					terminator: SsaTerminator::Goto(0),
				},
			],
			value_count: 1,
		};

		let artifact = lower_ssa_to_llvm_ir_with_preference(&ssa, LlvmAdapterPreference::Native)
			.expect("native lowering should succeed");

		assert!(artifact.ir_text.contains("phi i64 [ 1, %bb1 ], [ 2, %bb2 ]"));
	}

	#[cfg(not(feature = "llvm-native"))]
	#[test]
	fn native_preference_requires_feature_gate() {
		let ssa = LoweredSsaProgram {
			blocks: Vec::new(),
			value_count: 0,
		};

		let error = lower_ssa_to_llvm_ir_with_preference(&ssa, LlvmAdapterPreference::Native)
			.expect_err("native lowering should fail when feature is disabled");
		assert_eq!(error, BackendLoweringError::NativeAdapterUnavailable);
	}

	#[test]
	fn lowering_rejects_unsupported_branch_condition() {
		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: Vec::new(),
				terminator: SsaTerminator::Branch {
					condition: SsaValue::Integer(1),
					then_block: 1,
					else_block: 2,
				},
			}],
			value_count: 0,
		};

		let error = lower_ssa_to_llvm_ir(&ssa).expect_err("integer branch condition should be rejected");
		assert_eq!(error, BackendLoweringError::UnsupportedBranchCondition { block: 0 });
	}

	#[test]
	fn lowering_rejects_void_return_for_i64_entry() {
		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: Vec::new(),
				terminator: SsaTerminator::Return(None),
			}],
			value_count: 0,
		};

		let error = lower_ssa_to_llvm_ir(&ssa).expect_err("void return should be rejected for i64 entry");
		assert_eq!(error, BackendLoweringError::UnsupportedVoidReturn { block: 0 });
	}

	#[test]
	fn lowering_rejects_unsupported_assign_value() {
		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: vec![SsaStatement {
					id: 7,
					kind: SsaStatementKind::Assign {
						target: SsaName {
							place: MirPlace::Temp(0),
							version: 1,
						},
						value: SsaValue::Float("1.0".to_string()),
					},
				}],
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 1,
		};

		let error = lower_ssa_to_llvm_ir(&ssa).expect_err("float assign should be rejected");
		assert_eq!(
			error,
			BackendLoweringError::UnsupportedAssignValue {
				block: 0,
				statement: 7,
			}
		);
	}

	#[test]
	fn lowering_rejects_unsupported_eval_value() {
		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: vec![SsaStatement {
					id: 9,
					kind: SsaStatementKind::Eval(SsaValue::OpaqueExpr),
				}],
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 0,
		};

		let error = lower_ssa_to_llvm_ir(&ssa).expect_err("opaque eval should be rejected");
		assert_eq!(
			error,
			BackendLoweringError::UnsupportedEvalValue {
				block: 0,
				statement: 9,
			}
		);
	}

	#[test]
	fn lowering_rejects_unsupported_phi_incoming_value() {
		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: vec![SsaPhi {
					target: SsaName {
						place: MirPlace::Temp(1),
						version: 1,
					},
					incoming: vec![SsaPhiIncoming {
						block: 2,
						value: SsaValue::String("bad".to_string()),
					}],
				}],
				statements: Vec::new(),
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 0,
		};

		let error = lower_ssa_to_llvm_ir(&ssa).expect_err("string phi incoming should be rejected");
		assert_eq!(error, BackendLoweringError::UnsupportedPhiIncomingValue { block: 0 });
	}
}
