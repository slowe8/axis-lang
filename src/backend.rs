use crate::mir::LoweredSsaProgram;

pub fn initialize() {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlvmModuleArtifact {
	pub module_name: String,
	pub ir_text: String,
	pub function_count: usize,
	pub block_count: usize,
	pub value_count: usize,
}

pub fn lower_ssa_to_llvm_ir(ssa: &LoweredSsaProgram) -> LlvmModuleArtifact {
	let function_count = if ssa.blocks.is_empty() { 0 } else { 1 };
	let block_count = ssa.blocks.len();

	// Placeholder LLVM text artifact until real instruction lowering lands.
	let ir_text = format!(
		"; axis-lang llvm scaffold\n; functions: {function_count}\n; blocks: {block_count}\n; values: {}\n",
		ssa.value_count
	);

	LlvmModuleArtifact {
		module_name: "axis.main".to_string(),
		ir_text,
		function_count,
		block_count,
		value_count: ssa.value_count,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mir::LoweredSsaProgram;

	#[test]
	fn lower_ssa_to_llvm_ir_emits_stub_text_with_counts() {
		let ssa = LoweredSsaProgram {
			blocks: Vec::new(),
			value_count: 3,
		};

		let artifact = lower_ssa_to_llvm_ir(&ssa);
		assert_eq!(artifact.function_count, 0);
		assert_eq!(artifact.block_count, 0);
		assert_eq!(artifact.value_count, 3);
		assert!(artifact.ir_text.contains("axis-lang llvm scaffold"));
	}
}
