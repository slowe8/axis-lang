use crate::mir::{LoweredSsaProgram, MirPlace, SsaName, SsaTerminator, SsaTypeMap, SsaValue};
use crate::types::{Type, TypeId, TypeStore};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render_native_llvm_ir(ssa: &LoweredSsaProgram, ssa_types: &SsaTypeMap, types: &TypeStore) -> String {
	let mut ir_text = String::new();
	ir_text.push_str("; axis-lang native llvm adapter\n");
	ir_text.push_str("; feature llvm-native enabled\n");
	ir_text.push_str(&format!("; functions: {}\n", if ssa.blocks.is_empty() { 0 } else { 1 }));
	ir_text.push_str(&format!("; blocks: {}\n", ssa.blocks.len()));
	ir_text.push_str(&format!("; values: {}\n\n", ssa.value_count));

	if ssa.blocks.is_empty() {
		ir_text.push_str("; no functions emitted\n");
		return ir_text;
	}

	ir_text.push_str("define i64 @axis_main() {\n");
	for block in &ssa.blocks {
		ir_text.push_str(&format!("bb{}:\n", block.id));

		for phi in &block.phis {
			let phi_type = name_llvm_type(&phi.target, ssa_types, types);
			let incoming = phi
				.incoming
				.iter()
				.map(|incoming| {
					format!(
						"[ {}, %bb{} ]",
						render_operand_for_type(&incoming.value, phi_type, ssa_types, types),
						incoming.block
					)
				})
				.collect::<Vec<_>>()
				.join(", ");
			ir_text.push_str(&format!("  {} = phi {} {}\n", render_name(&phi.target), phi_type, incoming));
		}

		for statement in &block.statements {
			match &statement.kind {
				crate::mir::SsaStatementKind::Assign { target, value } => {
					render_assign_line(&mut ir_text, target, value, ssa_types, types);
				}
				crate::mir::SsaStatementKind::Eval(value) => {
					ir_text.push_str(&format!("  ; eval {}\n", render_value_debug(value)));
				}
			}
		}

		render_terminator_lines(&mut ir_text, block.id, &block.terminator, ssa_types, types);
	}
	ir_text.push_str("}\n");

	ir_text
}

pub fn emit_object_from_ir(ir_text: &str, output_path: &Path) -> Result<(), String> {
	let temp_ir = temporary_ir_path(output_path);
	let clang_path = resolve_clang_path()?;

	fs::write(&temp_ir, ir_text).map_err(|error| format!("failed to write temporary IR file: {error}"))?;

	let status = Command::new(&clang_path)
		.arg("-Wno-override-module")
		.arg("-c")
		.arg(&temp_ir)
		.arg("-o")
		.arg(output_path)
		.status();

	let _ = fs::remove_file(&temp_ir);

	match status {
		Ok(status) if status.success() => Ok(()),
		Ok(status) => Err(format!("{} exited with status {status}", clang_path.display())),
		Err(error) => Err(format!("failed to execute {}: {error}", clang_path.display())),
	}
}

fn resolve_clang_path() -> Result<PathBuf, String> {
	if let Ok(explicit_path) = env::var("AXIS_LLVM_CLANG") {
		let path = PathBuf::from(explicit_path);
		if path.is_file() {
			return Ok(path);
		}
		return Err(format!(
			"AXIS_LLVM_CLANG is set but does not point to an executable file: {}",
			path.display()
		));
	}

	let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let submodule_clang = repo_root.join("third_party/llvm-build/bin/clang");
	if submodule_clang.is_file() {
		return Ok(submodule_clang);
	}

	Err(format!(
		"clang from vendored LLVM not found at {} (run scripts/bootstrap_llvm.sh)",
		submodule_clang.display()
	))
}

fn temporary_ir_path(output_path: &Path) -> std::path::PathBuf {
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_nanos())
		.unwrap_or(0);

	let mut path = output_path.to_path_buf();
	path.set_extension(format!("axis.tmp.{nanos}.ll"));
	path
}

fn render_assign_line(
	ir_text: &mut String,
	target: &SsaName,
	value: &SsaValue,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
) {
	let target_name = render_name(target);
	let target_type = name_llvm_type(target, ssa_types, types);

	match (target_type, value) {
		("i64", SsaValue::Integer(integer)) => {
			ir_text.push_str(&format!("  {target_name} = add i64 0, {integer}\n"));
		}
		("i64", SsaValue::Boolean(boolean)) => {
			ir_text.push_str(&format!(
				"  {target_name} = zext i1 {} to i64\n",
				if *boolean { "1" } else { "0" }
			));
		}
		("i64", SsaValue::Name(name)) => {
			let source_type = name_llvm_type(name, ssa_types, types);
			if source_type == "i1" {
				ir_text.push_str(&format!("  {target_name} = zext i1 {} to i64\n", render_name(name)));
			} else {
				ir_text.push_str(&format!("  {target_name} = add i64 0, {}\n", render_name(name)));
			}
		}
		("i1", SsaValue::Boolean(boolean)) => {
			ir_text.push_str(&format!("  {target_name} = or i1 0, {}\n", if *boolean { 1 } else { 0 }));
		}
		("i1", SsaValue::Integer(integer)) => {
			ir_text.push_str(&format!("  {target_name} = or i1 0, {}\n", if *integer == 0 { 0 } else { 1 }));
		}
		("i1", SsaValue::Name(name)) => {
			ir_text.push_str(&format!("  {target_name} = or i1 0, {}\n", render_name(name)));
		}
		_ => {
			ir_text.push_str(&format!(
				"  ; {} = {}\n",
				target_name,
				render_operand_for_type(value, target_type, ssa_types, types)
			));
		}
	}
}

fn render_terminator_lines(
	ir_text: &mut String,
	block_id: usize,
	terminator: &SsaTerminator,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
) {
	match terminator {
		SsaTerminator::Return(Some(value)) => match value {
			SsaValue::Name(name) if name_llvm_type(name, ssa_types, types) == "i1" => {
				let cast_name = format!("%ret_cast_bb{block_id}");
				ir_text.push_str(&format!("  {cast_name} = zext i1 {} to i64\n", render_name(name)));
				ir_text.push_str(&format!("  ret i64 {cast_name}\n"));
			}
			_ => {
				ir_text.push_str(&format!("  ret i64 {}\n", render_operand_for_type(value, "i64", ssa_types, types)));
			}
		},
		SsaTerminator::Return(None) => ir_text.push_str("  ret void\n"),
		SsaTerminator::Goto(target) => ir_text.push_str(&format!("  br label %bb{target}\n")),
		SsaTerminator::Branch {
			condition,
			then_block,
			else_block,
		} => {
			ir_text.push_str(&format!(
				"  br i1 {}, label %bb{}, label %bb{}\n",
				render_operand_for_type(condition, "i1", ssa_types, types),
				then_block,
				else_block
			));
		}
	}
}

fn name_llvm_type(name: &SsaName, ssa_types: &SsaTypeMap, types: &TypeStore) -> &'static str {
	ssa_types
		.type_of_name(name)
		.map(|ty| llvm_type_from_id(ty, types))
		.unwrap_or("i64")
}

fn llvm_type_from_id(ty: TypeId, types: &TypeStore) -> &'static str {
	match types.get(ty) {
		Some(Type::Primitive(crate::types::PrimitiveType::Bool)) => "i1",
		Some(Type::Primitive(crate::types::PrimitiveType::Int)) => "i64",
		Some(Type::Primitive(crate::types::PrimitiveType::Float)) => "double",
		Some(Type::Primitive(crate::types::PrimitiveType::Char)) => "i8",
		_ => "i64",
	}
}

fn render_operand_for_type(
	value: &SsaValue,
	target_type: &str,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
) -> String {
	match (target_type, value) {
		("i1", SsaValue::Boolean(boolean)) => {
			if *boolean {
				"1".to_string()
			} else {
				"0".to_string()
			}
		}
		("i1", SsaValue::Integer(integer)) => {
			if *integer == 0 {
				"0".to_string()
			} else {
				"1".to_string()
			}
		}
		("i1", SsaValue::Name(name)) => {
			if name_llvm_type(name, ssa_types, types) == "i1" {
				render_name(name)
			} else {
				"1".to_string()
			}
		}
		(_, SsaValue::Integer(integer)) => integer.to_string(),
		(_, SsaValue::Boolean(boolean)) => {
			if *boolean {
				"1".to_string()
			} else {
				"0".to_string()
			}
		}
		(_, SsaValue::Name(name)) => render_name(name),
		_ => "0".to_string(),
	}
}

fn render_name(name: &SsaName) -> String {
	match &name.place {
		MirPlace::Local(symbol) => match symbol {
			Some(symbol) => format!("%local_{}_v{}", symbol.0, name.version),
			None => format!("%local_none_v{}", name.version),
		},
		MirPlace::Temp(temp) => format!("%tmp_{}_v{}", temp, name.version),
	}
}

fn render_value_debug(value: &SsaValue) -> String {
	match value {
		SsaValue::Unit => "unit".to_string(),
		SsaValue::Integer(value) => value.to_string(),
		SsaValue::Float(value) => value.clone(),
		SsaValue::Boolean(value) => value.to_string(),
		SsaValue::String(value) => format!("\"{}\"", value),
		SsaValue::Char(value) => format!("'{}'", value),
		SsaValue::Name(name) => render_name(name),
		SsaValue::OpaqueExpr => "<opaque-expr>".to_string(),
		SsaValue::UnresolvedPlace(place) => format!("<unresolved:{place:?}>"),
	}
}