use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use axis_lang::backend::{
    emit_native_object_file, lower_ssa_to_llvm_ir_with_preference_and_types, LlvmAdapterPreference,
};
use axis_lang::passes::{pass_manager_for_profile, PassContext, PassProfile, PipelineState};
use axis_lang::resolution::ModulePath;

#[derive(Debug)]
struct Cli {
    source: Option<String>,
    profile: PassProfile,
    dump_ast: bool,
    dump_hir: bool,
    dump_typed: bool,
    dump_mir: bool,
    dump_ssa: bool,
    dump_llvm: bool,
    dump_passes: bool,
    emit_obj: Option<String>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            source: None,
            profile: PassProfile::Dev,
            dump_ast: false,
            dump_hir: false,
            dump_typed: false,
            dump_mir: false,
            dump_ssa: false,
            dump_llvm: false,
            dump_passes: false,
            emit_obj: None,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("axis-lang: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = parse_cli();
    let source = load_source(cli.source.as_deref())?;

    let mut state = PipelineState::new(source);
    let mut context = PassContext::new(ModulePath::root());
    let manager = pass_manager_for_profile(cli.profile);
    manager.run(&mut state, &mut context)?;

    if cli.dump_passes {
        println!("== Pass Log ==");
        for entry in &context.logs {
            println!("{}: {} ({} ms)", entry.pass, entry.message, entry.elapsed_ms);
        }
    }

    if cli.dump_ast {
        if let Some(ast) = state.ast.as_ref() {
            println!("== AST ==");
            println!("{ast:#?}");
        }
    }

    if cli.dump_hir {
        if let Some(hir) = state.hir.as_ref() {
            println!("== HIR ==");
            println!("{hir:#?}");
        }
    }

    if cli.dump_typed {
        if let Some(tir) = state.tir.as_ref() {
            println!("== Typed IR ==");
            println!("{tir:#?}");
        }
    }

    if cli.dump_mir {
        if let Some(mir) = state.mir.as_ref() {
            println!("== MIR ==");
            println!("{mir:#?}");
        }
    }

    if cli.dump_ssa {
        if let Some(ssa) = state.ssa.as_ref() {
            println!("== SSA Scaffold ==");
            println!("{ssa:#?}");
        }
    }

    if cli.dump_llvm {
        if let Some(llvm) = state.llvm.as_ref() {
            println!("== LLVM IR (Scaffold) ==");
            println!("{}", llvm.ir_text);
        }
    }

    if let Some(output_path) = cli.emit_obj.as_deref() {
        let ssa = state
            .ssa
            .as_ref()
            .ok_or_else(|| "object emission requires SSA artifact".to_string())?;
        let ssa_types = state
            .ssa_types
            .as_ref()
            .ok_or_else(|| "object emission requires SSA type map".to_string())?;
        let types = state
            .backend_types
            .as_ref()
            .ok_or_else(|| "object emission requires backend types".to_string())?;

        let artifact = lower_ssa_to_llvm_ir_with_preference_and_types(
            ssa,
            ssa_types,
            types,
            LlvmAdapterPreference::Native,
        )
        .map_err(|error| error.to_string())?;

        emit_native_object_file(&artifact, Path::new(output_path)).map_err(|error| error.to_string())?;
        println!("Emitted object file: {output_path}");
    }

    if !cli.dump_ast && !cli.dump_hir && !cli.dump_typed && !cli.dump_mir && !cli.dump_ssa && !cli.dump_llvm {
        let ast_count = state.ast.as_ref().map(|program| program.items.len()).unwrap_or(0);
        let hir_count = state.hir.as_ref().map(|program| program.symbols.len()).unwrap_or(0);
        let tir_count = state.tir.as_ref().map(|program| program.items.len()).unwrap_or(0);
        let mir_count = state.mir.as_ref().map(|program| program.item_count).unwrap_or(0);
        let ssa_count = state.ssa.as_ref().map(|program| program.value_count).unwrap_or(0);
        let llvm_block_count = state.llvm.as_ref().map(|artifact| artifact.block_count).unwrap_or(0);

        println!("Parsed {ast_count} item(s)");
        println!("Resolved {hir_count} symbol(s)");
        println!("Typed {tir_count} item(s)");
        println!("Lowered {mir_count} item(s)");
        println!("SSA values {ssa_count}");
        println!("LLVM blocks {llvm_block_count}");
        if context.diagnostics.has_errors() {
            println!("Diagnostics: {} error(s)", context.diagnostics.entries().len());
        }
    }

    if !context.diagnostics.is_empty()
        && (cli.dump_passes
            || cli.dump_ast
            || cli.dump_hir
            || cli.dump_typed
            || cli.dump_mir
            || cli.dump_ssa
            || cli.dump_llvm)
    {
        println!("== Diagnostics ==");
        for diagnostic in context.diagnostics.entries() {
            println!("{diagnostic}");
        }
    }

    Ok(())
}

fn parse_cli() -> Cli {
    let mut cli = Cli::default();
    let mut args = env::args().skip(1).peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-" => {
                cli.source = Some(arg);
                if args.peek().is_some() {
                    eprintln!("warning: extra arguments are ignored");
                }
                break;
            }
            "--ast" | "--dump-ast" => cli.dump_ast = true,
            "--hir" | "--dump-hir" => cli.dump_hir = true,
            "--typed" | "--dump-typed" | "--ir" => cli.dump_typed = true,
            "--mir" | "--dump-mir" => cli.dump_mir = true,
            "--ssa" | "--dump-ssa" => cli.dump_ssa = true,
            "--llvm" | "--dump-llvm" => cli.dump_llvm = true,
            "--passes" | "--dump-passes" => cli.dump_passes = true,
            "--emit-obj" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --emit-obj (expected: output object file path)");
                    std::process::exit(2);
                };
                cli.emit_obj = Some(value);
            }
            "--profile" => {
                let Some(value) = args.next() else {
                    eprintln!("missing value for --profile (expected: dev|test|release)");
                    std::process::exit(2);
                };

                let Some(profile) = PassProfile::parse(&value) else {
                    eprintln!("unknown profile '{value}' (expected: dev|test|release)");
                    std::process::exit(2);
                };

                cli.profile = profile;
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ if arg.starts_with('-') => {
                eprintln!("unknown flag: {arg}");
                print_usage();
                std::process::exit(2);
            }
            _ => {
                cli.source = Some(arg);
                if args.peek().is_some() {
                    eprintln!("warning: extra arguments are ignored");
                }
                break;
            }
        }
    }

    cli
}

fn load_source(source_path: Option<&str>) -> Result<String, String> {
    match source_path {
        Some(path) if path != "-" => fs::read_to_string(path).map_err(|error| format!("failed to read '{path}': {error}")),
        _ => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|error| format!("failed to read stdin: {error}"))?;
            if source.trim().is_empty() {
                Err("no source provided; pass a file path or pipe source on stdin".to_string())
            } else {
                Ok(source)
            }
        }
    }
}

fn print_usage() {
    eprintln!("Usage: axis-lang [--profile dev|test|release] [--passes] [--ast] [--hir] [--typed] [--mir] [--ssa] [--llvm] [--emit-obj <path>] <source-file>|-");
    eprintln!("  --profile            choose a pass profile (dev, test, release)");
    eprintln!("  --passes             print pass execution log");
    eprintln!("  --ast, --dump-ast     print the parsed AST");
    eprintln!("  --hir, --dump-hir     print the resolved HIR");
    eprintln!("  --typed, --ir         print the typed intermediate representation");
    eprintln!("  --mir, --dump-mir     print the lowered MIR summary");
    eprintln!("  --ssa, --dump-ssa     print the SSA scaffolding derived from MIR");
    eprintln!("  --llvm, --dump-llvm   print LLVM backend scaffold output");
	eprintln!("  --emit-obj <path>     emit an object file (requires --features llvm-native)");
    eprintln!("  -                     read source from stdin");
}
