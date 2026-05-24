use std::time::Instant;

use crate::backend::{lower_ssa_to_llvm_ir_with_types, LlvmModuleArtifact};
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::frontend::ast::Program;
use crate::frontend::parser::Parser;
use crate::hir::{build_hir, ResolvedProgram};
use crate::mir::{
    build_ssa_scaffold_with_types, lower_from_tir, verify_lowered_program, verify_ssa_scaffold, LoweredProgram,
    LoweredSsaProgram, SsaTypeMap,
};
use crate::resolution::ModulePath;
use crate::type_checker::CheckedProgram;
use crate::type_checker::TypeChecker;
use crate::types::TypeStore;

pub fn initialize() {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassLogEntry {
    pub pass: String,
    pub message: String,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassContext {
    pub module: ModulePath,
    pub diagnostics: Diagnostics,
    pub logs: Vec<PassLogEntry>,
}

impl PassContext {
    pub fn new(module: ModulePath) -> Self {
        Self {
            module,
            diagnostics: Diagnostics::new(),
            logs: Vec::new(),
        }
    }

    pub fn log(&mut self, pass: &str, message: impl Into<String>, elapsed_ms: u128) {
        self.logs.push(PassLogEntry {
            pass: pass.to_string(),
            message: message.into(),
            elapsed_ms,
        });
    }

    pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

impl Default for PassContext {
    fn default() -> Self {
        Self::new(ModulePath::root())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineState {
    pub source: String,
    pub ast: Option<Program>,
    pub hir: Option<ResolvedProgram>,
    pub tir: Option<CheckedProgram>,
    pub backend_types: Option<TypeStore>,
    pub mir: Option<LoweredProgram>,
    pub ssa: Option<LoweredSsaProgram>,
    pub ssa_types: Option<SsaTypeMap>,
    pub llvm: Option<LlvmModuleArtifact>,
}

impl PipelineState {
    pub fn new(source: String) -> Self {
        Self {
            source,
            ast: None,
            hir: None,
            tir: None,
            backend_types: None,
            mir: None,
            ssa: None,
            ssa_types: None,
            llvm: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassProfile {
    Dev,
    Test,
    Release,
}

impl PassProfile {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "dev" => Some(Self::Dev),
            "test" => Some(Self::Test),
            "release" => Some(Self::Release),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineConfig {
    pub profile: PassProfile,
    pub enable_resolve: bool,
    pub enable_type_check: bool,
    pub enable_lower: bool,
    pub enable_ssa: bool,
    pub enable_backend: bool,
}

impl PipelineConfig {
    pub fn for_profile(profile: PassProfile) -> Self {
        match profile {
            PassProfile::Dev => Self {
                profile,
                enable_resolve: true,
                enable_type_check: true,
                enable_lower: true,
                enable_ssa: true,
                enable_backend: true,
            },
            PassProfile::Test => Self {
                profile,
                enable_resolve: true,
                enable_type_check: true,
                enable_lower: true,
                enable_ssa: true,
                enable_backend: true,
            },
            PassProfile::Release => Self {
                profile,
                enable_resolve: true,
                enable_type_check: true,
                enable_lower: true,
                enable_ssa: true,
                enable_backend: true,
            },
        }
    }
}

pub trait CompilerPass {
    fn name(&self) -> &'static str;
    fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String>;
}

#[derive(Default)]
pub struct PassManager {
    passes: Vec<Box<dyn CompilerPass>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P: CompilerPass + 'static>(&mut self, pass: P) {
        self.passes.push(Box::new(pass));
    }

    pub fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String> {
        for pass in &self.passes {
            let started = Instant::now();
            pass.run(state, context)?;
            let elapsed_ms = started.elapsed().as_millis();
            context.log(pass.name(), "completed", elapsed_ms);
        }

        Ok(())
    }
}

pub fn default_pass_manager() -> PassManager {
    pass_manager_for_profile(PassProfile::Dev)
}

pub fn pass_manager_for_profile(profile: PassProfile) -> PassManager {
    pass_manager_with_config(PipelineConfig::for_profile(profile))
}

pub fn pass_manager_with_config(config: PipelineConfig) -> PassManager {
    let mut manager = PassManager::new();
    manager.register(ParsePass);
    if config.enable_resolve {
        manager.register(ResolvePass);
    }
    if config.enable_type_check {
        manager.register(TypeCheckPass);
    }
    if config.enable_lower {
        manager.register(LowerPass);
    }
    if config.enable_ssa {
        manager.register(SsaPass);
    }
    if config.enable_backend {
        manager.register(BackendLlvmPass);
    }
    manager
}

pub struct ParsePass;

impl CompilerPass for ParsePass {
    fn name(&self) -> &'static str {
        "parse"
    }

    fn run(&self, state: &mut PipelineState, _context: &mut PassContext) -> Result<(), String> {
        let mut parser = Parser::new(&state.source);
        state.ast = Some(parser.parse_program());
        Ok(())
    }
}

pub struct ResolvePass;

impl CompilerPass for ResolvePass {
    fn name(&self) -> &'static str {
        "resolve"
    }

    fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String> {
        let Some(ast) = state.ast.as_ref() else {
            return Err("resolve pass requires AST".to_string());
        };

        state.hir = Some(build_hir(ast, context.module.clone()));
        Ok(())
    }
}

pub struct TypeCheckPass;

impl CompilerPass for TypeCheckPass {
    fn name(&self) -> &'static str {
        "type-check"
    }

    fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String> {
        let hir = state.hir.as_ref().ok_or_else(|| "type-check pass requires HIR".to_string())?;

        let checker = TypeChecker::new(context.module.clone());
        let checked = checker.check_hir(hir);
        context.diagnostics.extend(checked.diagnostics.entries().iter().cloned());
        state.backend_types = Some(checked.types.clone());
        state.tir = Some(checked);
        Ok(())
    }
}

pub struct LowerPass;

impl CompilerPass for LowerPass {
    fn name(&self) -> &'static str {
        "lower"
    }

    fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String> {
        let hir = state.hir.as_ref().ok_or_else(|| "lower pass requires HIR".to_string())?;
        let tir = state.tir.as_ref().ok_or_else(|| "lower pass requires TIR".to_string())?;
        let mir = lower_from_tir(tir, hir);
        let diagnostics = verify_lowered_program(&mir);
        context.diagnostics.extend(diagnostics);
        state.mir = Some(mir);
        Ok(())
    }
}

pub struct SsaPass;

impl CompilerPass for SsaPass {
    fn name(&self) -> &'static str {
        "ssa"
    }

    fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String> {
        let mir = state.mir.as_ref().ok_or_else(|| "ssa pass requires MIR".to_string())?;
        let tir = state.tir.as_ref().ok_or_else(|| "ssa pass requires TIR".to_string())?;
        let (ssa, ssa_types) = build_ssa_scaffold_with_types(mir, tir);
        let diagnostics = verify_ssa_scaffold(&ssa);
        context.diagnostics.extend(diagnostics);
        state.ssa_types = Some(ssa_types);
        state.ssa = Some(ssa);
        Ok(())
    }
}

pub struct BackendLlvmPass;

impl CompilerPass for BackendLlvmPass {
    fn name(&self) -> &'static str {
        "backend-llvm"
    }

    fn run(&self, state: &mut PipelineState, context: &mut PassContext) -> Result<(), String> {
        if context.diagnostics.has_errors() {
            return Err("backend-llvm pass aborted due to existing diagnostics".to_string());
        }

        let ssa = state.ssa.as_ref().ok_or_else(|| "backend-llvm pass requires SSA".to_string())?;
        let ssa_types = state
            .ssa_types
            .as_ref()
            .ok_or_else(|| "backend-llvm pass requires SSA type map".to_string())?;
        let types = state
            .backend_types
            .as_ref()
            .ok_or_else(|| "backend-llvm pass requires backend types".to_string())?;
        state.llvm = Some(
            lower_ssa_to_llvm_ir_with_types(ssa, ssa_types, types).map_err(|error| error.to_string())?,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pipeline_populates_ast_hir_and_tir() {
        let mut state = PipelineState::new("fn main() -> int { let x = 1; x }".to_string());
        let mut context = PassContext::default();
        let manager = default_pass_manager();

        manager.run(&mut state, &mut context).expect("pipeline run should succeed");

        assert!(state.ast.is_some());
        assert!(state.hir.is_some());
        assert!(state.tir.is_some());
        assert!(state.backend_types.is_some());
        assert!(state.mir.is_some());
        assert!(state.ssa.is_some());
        assert!(state.ssa_types.is_some());
        assert!(state.llvm.is_some());
        assert_eq!(context.logs.len(), 6);
    }

    #[test]
    fn pass_manager_for_profile_registers_lowering() {
        let mut state = PipelineState::new("fn main() -> int { let x = 1; x }".to_string());
        let mut context = PassContext::default();
        let manager = pass_manager_for_profile(PassProfile::Release);

        manager.run(&mut state, &mut context).expect("release pipeline run should succeed");
        assert!(state.mir.is_some());
        assert!(state.ssa.is_some());
        assert!(state.ssa_types.is_some());
        assert!(state.llvm.is_some());
    }

    #[test]
    fn backend_pass_requires_backend_types_artifact() {
        let mut state = PipelineState::new("".to_string());
        state.ssa = Some(LoweredSsaProgram {
            blocks: Vec::new(),
            value_count: 0,
        });
        state.ssa_types = Some(SsaTypeMap::default());

        let mut context = PassContext::default();
        let pass = BackendLlvmPass;

        let error = pass.run(&mut state, &mut context).expect_err("backend pass should require backend types");
        assert!(error.contains("backend types"));
    }
}