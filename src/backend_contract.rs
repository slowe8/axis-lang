use crate::backend::BackendLoweringError;
use crate::mir::{LoweredSsaProgram, SsaTerminator, SsaTypeMap, SsaValue};
use crate::types::TypeStore;

pub fn validate_text_backend_subset(
	ssa: &LoweredSsaProgram,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
) -> Result<(), BackendLoweringError> {
	let bool_type = types.named("bool");
	let int_type = types.named("int");

	for block in &ssa.blocks {
		for phi in &block.phis {
			for incoming in &phi.incoming {
				if !is_supported_phi_incoming(&incoming.value) {
					return Err(BackendLoweringError::UnsupportedPhiIncomingValue { block: block.id });
				}

				if let Some(target_type) = ssa_types.type_of_name(&phi.target) {
					if let Some(incoming_type) = value_type(incoming.value.clone(), ssa_types, bool_type, int_type) {
						if incoming_type != target_type {
							return Err(BackendLoweringError::PhiTypeMismatch { block: block.id });
						}
					}
				}
			}
		}

		for statement in &block.statements {
			match &statement.kind {
				crate::mir::SsaStatementKind::Assign { value, .. } => {
					if !is_supported_assign_value(value) {
						return Err(BackendLoweringError::UnsupportedAssignValue {
							block: block.id,
							statement: statement.id,
						});
					}

					if let crate::mir::SsaStatementKind::Assign { target, value } = &statement.kind {
						if let Some(target_type) = ssa_types.type_of_name(target) {
							if let Some(value_type) = value_type(value.clone(), ssa_types, bool_type, int_type) {
								if value_type != target_type {
									return Err(BackendLoweringError::AssignTypeMismatch {
										block: block.id,
										statement: statement.id,
									});
								}
							}
						}
					}
				}
				crate::mir::SsaStatementKind::Eval(value) => {
					if !is_supported_eval_value(value) {
						return Err(BackendLoweringError::UnsupportedEvalValue {
							block: block.id,
							statement: statement.id,
						});
					}
				}
			}
		}

		match &block.terminator {
			SsaTerminator::Return(None) => {
				return Err(BackendLoweringError::UnsupportedVoidReturn { block: block.id });
			}
			SsaTerminator::Return(Some(value)) => {
				if !is_supported_return_value(value) {
					return Err(BackendLoweringError::UnsupportedReturnValue { block: block.id });
				}

				if let (Some(int_type), Some(return_type)) = (
					int_type,
					value_type(value.clone(), ssa_types, bool_type, int_type),
				) {
					if return_type != int_type && bool_type.is_some_and(|bool_type| return_type != bool_type) {
						return Err(BackendLoweringError::ReturnTypeMismatch { block: block.id });
					}
				}
			}
			SsaTerminator::Branch { condition, .. } => {
				if !is_supported_branch_condition(condition) {
					return Err(BackendLoweringError::UnsupportedBranchCondition { block: block.id });
				}

				if let (Some(bool_type), SsaValue::Name(name)) = (bool_type, condition) {
					if let Some(actual_type) = ssa_types.type_of_name(name) {
						if actual_type != bool_type {
							return Err(BackendLoweringError::BranchConditionTypeMismatch { block: block.id });
						}
					}
				}
			}
			_ => {}
		}
	}

	Ok(())
}

fn value_type(
	value: SsaValue,
	ssa_types: &SsaTypeMap,
	bool_type: Option<crate::types::TypeId>,
	int_type: Option<crate::types::TypeId>,
) -> Option<crate::types::TypeId> {
	match value {
		SsaValue::Boolean(_) => bool_type,
		SsaValue::Integer(_) => int_type,
		SsaValue::Name(name) => ssa_types.type_of_name(&name),
		_ => None,
	}
}

pub fn is_supported_phi_incoming(value: &SsaValue) -> bool {
	matches!(value, SsaValue::Integer(_) | SsaValue::Boolean(_) | SsaValue::Name(_))
}

pub fn is_supported_assign_value(value: &SsaValue) -> bool {
	matches!(value, SsaValue::Integer(_) | SsaValue::Boolean(_) | SsaValue::Name(_))
}

pub fn is_supported_eval_value(value: &SsaValue) -> bool {
	!matches!(value, SsaValue::OpaqueExpr | SsaValue::UnresolvedPlace(_))
}

pub fn is_supported_return_value(value: &SsaValue) -> bool {
	matches!(value, SsaValue::Integer(_) | SsaValue::Boolean(_) | SsaValue::Name(_))
}

pub fn is_supported_branch_condition(value: &SsaValue) -> bool {
	matches!(value, SsaValue::Boolean(_) | SsaValue::Name(_))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::hir::SymbolId;
	use crate::mir::{
		LoweredSsaProgram, MirPlace, SsaBasicBlock, SsaName, SsaPhi, SsaPhiIncoming, SsaStatement,
		SsaStatementKind, SsaTerminator, SsaValue,
	};
	use crate::types::TypeStore;

	fn sample_values() -> Vec<(&'static str, SsaValue)> {
		vec![
			("unit", SsaValue::Unit),
			("integer", SsaValue::Integer(42)),
			("float", SsaValue::Float("3.14".to_string())),
			("boolean", SsaValue::Boolean(true)),
			("string", SsaValue::String("hello".to_string())),
			("char", SsaValue::Char('x')),
			(
				"name",
				SsaValue::Name(SsaName {
					place: MirPlace::Local(Some(SymbolId(1))),
					version: 1,
				}),
			),
			("opaque", SsaValue::OpaqueExpr),
			("unresolved", SsaValue::UnresolvedPlace(MirPlace::Temp(9))),
		]
	}

	fn expected_phi_support(value: &SsaValue) -> bool {
		matches!(value, SsaValue::Integer(_) | SsaValue::Boolean(_) | SsaValue::Name(_))
	}

	fn expected_assign_support(value: &SsaValue) -> bool {
		matches!(value, SsaValue::Integer(_) | SsaValue::Boolean(_) | SsaValue::Name(_))
	}

	fn expected_eval_support(value: &SsaValue) -> bool {
		!matches!(value, SsaValue::OpaqueExpr | SsaValue::UnresolvedPlace(_))
	}

	fn expected_return_support(value: &SsaValue) -> bool {
		matches!(value, SsaValue::Integer(_) | SsaValue::Boolean(_) | SsaValue::Name(_))
	}

	fn expected_branch_support(value: &SsaValue) -> bool {
		matches!(value, SsaValue::Boolean(_) | SsaValue::Name(_))
	}

	fn ssa_with_phi(value: SsaValue) -> LoweredSsaProgram {
		LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: vec![SsaPhi {
					target: SsaName {
						place: MirPlace::Temp(1),
						version: 1,
					},
					incoming: vec![SsaPhiIncoming { block: 1, value }],
				}],
				statements: Vec::new(),
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 0,
		}
	}

	fn ssa_with_assign(value: SsaValue) -> LoweredSsaProgram {
		LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: vec![SsaStatement {
					id: 0,
					kind: SsaStatementKind::Assign {
						target: SsaName {
							place: MirPlace::Temp(2),
							version: 1,
						},
						value,
					},
				}],
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 1,
		}
	}

	fn ssa_with_eval(value: SsaValue) -> LoweredSsaProgram {
		LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: vec![SsaStatement {
					id: 0,
					kind: SsaStatementKind::Eval(value),
				}],
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 0,
		}
	}

	fn ssa_with_return(value: SsaValue) -> LoweredSsaProgram {
		LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: Vec::new(),
				terminator: SsaTerminator::Return(Some(value)),
			}],
			value_count: 0,
		}
	}

	fn ssa_with_branch_condition(value: SsaValue) -> LoweredSsaProgram {
		LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: Vec::new(),
				terminator: SsaTerminator::Branch {
					condition: value,
					then_block: 1,
					else_block: 2,
				},
			}],
			value_count: 0,
		}
	}

	#[test]
	fn table_driven_phi_contract() {
		let types = TypeStore::new();
		let ssa_types = SsaTypeMap::default();
		for (label, value) in sample_values() {
			let expected = expected_phi_support(&value);
			let result = validate_text_backend_subset(&ssa_with_phi(value), &ssa_types, &types);
			assert_eq!(result.is_ok(), expected, "phi contract mismatch for {label}");
		}
	}

	#[test]
	fn table_driven_assign_contract() {
		let types = TypeStore::new();
		let ssa_types = SsaTypeMap::default();
		for (label, value) in sample_values() {
			let expected = expected_assign_support(&value);
			let result = validate_text_backend_subset(&ssa_with_assign(value), &ssa_types, &types);
			assert_eq!(result.is_ok(), expected, "assign contract mismatch for {label}");
		}
	}

	#[test]
	fn table_driven_eval_contract() {
		let types = TypeStore::new();
		let ssa_types = SsaTypeMap::default();
		for (label, value) in sample_values() {
			let expected = expected_eval_support(&value);
			let result = validate_text_backend_subset(&ssa_with_eval(value), &ssa_types, &types);
			assert_eq!(result.is_ok(), expected, "eval contract mismatch for {label}");
		}
	}

	#[test]
	fn table_driven_return_contract() {
		let types = TypeStore::new();
		let ssa_types = SsaTypeMap::default();
		for (label, value) in sample_values() {
			let expected = expected_return_support(&value);
			let result = validate_text_backend_subset(&ssa_with_return(value), &ssa_types, &types);
			assert_eq!(result.is_ok(), expected, "return contract mismatch for {label}");
		}
	}

	#[test]
	fn table_driven_branch_contract() {
		let types = TypeStore::new();
		let ssa_types = SsaTypeMap::default();
		for (label, value) in sample_values() {
			let expected = expected_branch_support(&value);
			let result = validate_text_backend_subset(&ssa_with_branch_condition(value), &ssa_types, &types);
			assert_eq!(result.is_ok(), expected, "branch contract mismatch for {label}");
		}
	}

	#[test]
	fn branch_name_type_must_be_bool_when_type_map_provides_entry() {
		let types = TypeStore::new();
		let mut ssa_types = SsaTypeMap::default();
		let int_type = types.named("int").expect("int type should exist");

		let condition_name = SsaName {
			place: MirPlace::Local(Some(SymbolId(5))),
			version: 1,
		};
		let _ = ssa_types.insert_name_type(&condition_name, int_type);

		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: Vec::new(),
				terminator: SsaTerminator::Branch {
					condition: SsaValue::Name(condition_name),
					then_block: 1,
					else_block: 2,
				},
			}],
			value_count: 0,
		};

		let error = validate_text_backend_subset(&ssa, &ssa_types, &types)
			.expect_err("non-bool condition name should fail when typed map provides condition type");
		assert_eq!(error, BackendLoweringError::BranchConditionTypeMismatch { block: 0 });
	}

	#[test]
	fn assign_name_type_must_match_target_type_when_known() {
		let types = TypeStore::new();
		let mut ssa_types = SsaTypeMap::default();
		let int_type = types.named("int").expect("int type should exist");
		let bool_type = types.named("bool").expect("bool type should exist");

		let target = SsaName {
			place: MirPlace::Local(Some(SymbolId(10))),
			version: 1,
		};
		let source = SsaName {
			place: MirPlace::Local(Some(SymbolId(11))),
			version: 1,
		};

		let _ = ssa_types.insert_name_type(&target, bool_type);
		let _ = ssa_types.insert_name_type(&source, int_type);

		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: vec![SsaStatement {
					id: 4,
					kind: SsaStatementKind::Assign {
						target,
						value: SsaValue::Name(source),
					},
				}],
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 0,
		};

		let error = validate_text_backend_subset(&ssa, &ssa_types, &types)
			.expect_err("assign with mismatched known types should fail");
		assert_eq!(
			error,
			BackendLoweringError::AssignTypeMismatch {
				block: 0,
				statement: 4,
			}
		);
	}

	#[test]
	fn phi_incoming_type_must_match_target_when_known() {
		let types = TypeStore::new();
		let mut ssa_types = SsaTypeMap::default();
		let int_type = types.named("int").expect("int type should exist");
		let bool_type = types.named("bool").expect("bool type should exist");

		let target = SsaName {
			place: MirPlace::Temp(1),
			version: 1,
		};
		let incoming_name = SsaName {
			place: MirPlace::Temp(2),
			version: 1,
		};
		let _ = ssa_types.insert_name_type(&target, bool_type);
		let _ = ssa_types.insert_name_type(&incoming_name, int_type);

		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: vec![SsaPhi {
					target,
					incoming: vec![SsaPhiIncoming {
						block: 1,
						value: SsaValue::Name(incoming_name),
					}],
				}],
				statements: Vec::new(),
				terminator: SsaTerminator::Return(Some(SsaValue::Integer(0))),
			}],
			value_count: 0,
		};

		let error = validate_text_backend_subset(&ssa, &ssa_types, &types)
			.expect_err("phi incoming with mismatched known type should fail");
		assert_eq!(error, BackendLoweringError::PhiTypeMismatch { block: 0 });
	}

	#[test]
	fn return_name_type_must_match_int_entry_contract_when_known() {
		let types = TypeStore::new();
		let mut ssa_types = SsaTypeMap::default();
		let string_type = types.named("string").expect("string type should exist");

		let returned_name = SsaName {
			place: MirPlace::Local(Some(SymbolId(12))),
			version: 1,
		};
		let _ = ssa_types.insert_name_type(&returned_name, string_type);

		let ssa = LoweredSsaProgram {
			blocks: vec![SsaBasicBlock {
				id: 0,
				phis: Vec::new(),
				statements: Vec::new(),
				terminator: SsaTerminator::Return(Some(SsaValue::Name(returned_name))),
			}],
			value_count: 0,
		};

		let error = validate_text_backend_subset(&ssa, &ssa_types, &types)
			.expect_err("known non-int/bool return type should fail entry return contract");
		assert_eq!(error, BackendLoweringError::ReturnTypeMismatch { block: 0 });
	}
}