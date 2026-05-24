use crate::mir::{LoweredSsaProgram, MirPlace, SsaName, SsaTerminator, SsaTypeMap, SsaValue};
use crate::types::{Type, TypeId, TypeStore};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render_native_llvm_ir(ssa: &LoweredSsaProgram, ssa_types: &SsaTypeMap, types: &TypeStore) -> String {
	let block_scoped_names = collect_block_scoped_definition_names(ssa);

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
						render_operand_for_type(
							&incoming.value,
							phi_type,
							ssa_types,
							types,
							block.id,
							Some(incoming.block),
							&block_scoped_names,
						),
						incoming.block
					)
				})
				.collect::<Vec<_>>()
				.join(", ");
			ir_text.push_str(&format!(
				"  {} = phi {} {}\n",
				render_definition_name(&phi.target, block.id, &block_scoped_names),
				phi_type,
				incoming
			));
		}

		for statement in &block.statements {
			match &statement.kind {
				crate::mir::SsaStatementKind::Assign { target, value } => {
					render_assign_line(
						&mut ir_text,
						block.id,
						target,
						value,
						ssa_types,
						types,
						&block_scoped_names,
					);
				}
				crate::mir::SsaStatementKind::Eval(value) => {
					ir_text.push_str(&format!("  ; eval {}\n", render_value_debug(value)));
				}
			}
		}

		render_terminator_lines(
			&mut ir_text,
			block.id,
			&block.terminator,
			ssa_types,
			types,
			&block_scoped_names,
		);
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

pub fn emit_executable_from_ir(ir_text: &str, output_path: &Path) -> Result<(), String> {
	let clang_path = resolve_clang_path()?;
	let temp_object = temporary_artifact_path(output_path, "obj");
	let temp_entry_source = temporary_artifact_path(output_path, "entry.c");
	let temp_entry_object = temporary_artifact_path(output_path, "entry.o");

	emit_object_from_ir(ir_text, &temp_object)?;

	let entry_source = "extern long axis_main(void);\nint main(void) { return (int)axis_main(); }\n";
	fs::write(&temp_entry_source, entry_source)
		.map_err(|error| format!("failed to write temporary entry source: {error}"))?;

	let compile_status = Command::new(&clang_path)
		.arg("-c")
		.arg(&temp_entry_source)
		.arg("-o")
		.arg(&temp_entry_object)
		.status();

	match compile_status {
		Ok(status) if status.success() => {}
		Ok(status) => {
			cleanup_temporary_file(&temp_object);
			cleanup_temporary_file(&temp_entry_source);
			cleanup_temporary_file(&temp_entry_object);
			return Err(format!(
				"{} failed to compile entry shim with status {status}",
				clang_path.display()
			));
		}
		Err(error) => {
			cleanup_temporary_file(&temp_object);
			cleanup_temporary_file(&temp_entry_source);
			cleanup_temporary_file(&temp_entry_object);
			return Err(format!(
				"failed to execute {} while compiling entry shim: {error}",
				clang_path.display()
			));
		}
	}

	let link_status = Command::new(&clang_path)
		.arg(&temp_object)
		.arg(&temp_entry_object)
		.arg("-o")
		.arg(output_path)
		.status();

	cleanup_temporary_file(&temp_object);
	cleanup_temporary_file(&temp_entry_source);
	cleanup_temporary_file(&temp_entry_object);

	match link_status {
		Ok(status) if status.success() => Ok(()),
		Ok(status) => Err(format!(
			"{} failed to link executable with status {status}",
			clang_path.display()
		)),
		Err(error) => Err(format!(
			"failed to execute {} while linking executable: {error}",
			clang_path.display()
		)),
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

fn temporary_artifact_path(output_path: &Path, suffix: &str) -> std::path::PathBuf {
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_nanos())
		.unwrap_or(0);

	let mut path = output_path.to_path_buf();
	path.set_extension(format!("axis.tmp.{nanos}.{suffix}"));
	path
}

fn cleanup_temporary_file(path: &Path) {
	let _ = fs::remove_file(path);
}

fn render_assign_line(
	ir_text: &mut String,
	block_id: usize,
	target: &SsaName,
	value: &SsaValue,
	ssa_types: &SsaTypeMap,
	types: &TypeStore,
	block_scoped_names: &HashSet<String>,
) {
	let target_name = render_definition_name(target, block_id, block_scoped_names);
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
			let source_name = render_use_name(name, block_id, None, block_scoped_names);
			if source_type == "i1" {
				ir_text.push_str(&format!("  {target_name} = zext i1 {source_name} to i64\n"));
			} else {
				ir_text.push_str(&format!("  {target_name} = add i64 0, {source_name}\n"));
			}
		}
		("i1", SsaValue::Boolean(boolean)) => {
			ir_text.push_str(&format!("  {target_name} = or i1 0, {}\n", if *boolean { 1 } else { 0 }));
		}
		("i1", SsaValue::Integer(integer)) => {
			ir_text.push_str(&format!("  {target_name} = or i1 0, {}\n", if *integer == 0 { 0 } else { 1 }));
		}
		("i1", SsaValue::Name(name)) => {
			ir_text.push_str(&format!(
				"  {target_name} = or i1 0, {}\n",
				render_use_name(name, block_id, None, block_scoped_names)
			));
		}
		_ => {
			ir_text.push_str(&format!(
				"  ; {} = {}\n",
				target_name,
				render_operand_for_type(
					value,
					target_type,
					ssa_types,
					types,
					block_id,
					None,
					block_scoped_names,
				)
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
	block_scoped_names: &HashSet<String>,
) {
	match terminator {
		SsaTerminator::Return(Some(value)) => match value {
			SsaValue::Name(name) if name_llvm_type(name, ssa_types, types) == "i1" => {
				let cast_name = format!("%ret_cast_bb{block_id}");
				ir_text.push_str(&format!(
					"  {cast_name} = zext i1 {} to i64\n",
					render_use_name(name, block_id, None, block_scoped_names)
				));
				ir_text.push_str(&format!("  ret i64 {cast_name}\n"));
			}
			_ => {
				ir_text.push_str(&format!(
					"  ret i64 {}\n",
					render_operand_for_type(value, "i64", ssa_types, types, block_id, None, block_scoped_names)
				));
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
				render_operand_for_type(condition, "i1", ssa_types, types, block_id, None, block_scoped_names),
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
	current_block: usize,
	source_block: Option<usize>,
	block_scoped_names: &HashSet<String>,
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
				render_use_name(name, current_block, source_block, block_scoped_names)
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
		(_, SsaValue::Name(name)) => render_use_name(name, current_block, source_block, block_scoped_names),
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

fn collect_block_scoped_definition_names(ssa: &LoweredSsaProgram) -> HashSet<String> {
	let mut definitions: HashMap<String, HashSet<usize>> = HashMap::new();

	for block in &ssa.blocks {
		for phi in &block.phis {
			definitions
				.entry(render_name(&phi.target))
				.or_default()
				.insert(block.id);
		}

		for statement in &block.statements {
			if let crate::mir::SsaStatementKind::Assign { target, .. } = &statement.kind {
				definitions.entry(render_name(target)).or_default().insert(block.id);
			}
		}
	}

	definitions
		.into_iter()
		.filter_map(|(name, blocks)| if blocks.len() > 1 { Some(name) } else { None })
		.collect()
}

fn render_definition_name(name: &SsaName, block_id: usize, block_scoped_names: &HashSet<String>) -> String {
	let base = render_name(name);
	if block_scoped_names.contains(&base) {
		format!("{base}_bb{block_id}")
	} else {
		base
	}
}

fn render_use_name(
	name: &SsaName,
	current_block: usize,
	source_block: Option<usize>,
	block_scoped_names: &HashSet<String>,
) -> String {
	let base = render_name(name);
	if !block_scoped_names.contains(&base) {
		return base;
	}

	let block_id = source_block.unwrap_or(current_block);
	format!("{base}_bb{block_id}")
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