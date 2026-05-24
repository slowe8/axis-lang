use std::collections::BTreeMap;

use crate::frontend::ast::{Block, Expr, Item, Program, Stmt};
use crate::resolution::{
    Module, ModuleGraph, ModulePath, ResolutionScope, ResolvedSymbol, Resolver, Symbol, SymbolKind, Visibility,
};

pub fn initialize() {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub module: ModulePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirNameUse {
    pub name: String,
    pub module: ModulePath,
    pub scope: Option<ResolutionScope>,
    pub symbol_id: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirDeclarationKind {
    Parameter,
    Let,
    ForBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirDeclaration {
    pub name: String,
    pub module: ModulePath,
    pub symbol_id: SymbolId,
    pub kind: HirDeclarationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProgram {
    pub program: Program,
    pub symbols: Vec<HirSymbol>,
    pub name_uses: Vec<HirNameUse>,
    pub declarations: Vec<HirDeclaration>,
}

pub fn build_hir(program: &Program, root_module: ModulePath) -> ResolvedProgram {
    let mut symbols = Vec::new();
    let mut next_id = 0;
    collect_symbols(&program.items, &root_module, &mut symbols, &mut next_id);
    let id_by_name = index_symbols_by_name(&symbols);
    let graph = build_module_graph(&symbols);
    let resolver = Resolver::new(graph);

    let mut name_uses = Vec::new();
    let mut declarations = Vec::new();
    let mut lexical_scope = Vec::new();
    collect_name_uses(
        &program.items,
        &resolver,
        &id_by_name,
        &root_module,
        &mut lexical_scope,
        &mut symbols,
        &mut next_id,
        &mut name_uses,
        &mut declarations,
    );

    ResolvedProgram {
        program: program.clone(),
        symbols,
        name_uses,
        declarations,
    }
}

fn collect_symbols(items: &[Item], module: &ModulePath, out: &mut Vec<HirSymbol>, next_id: &mut usize) {
    for item in items {
        match item {
            Item::Function(function) => out.push(HirSymbol {
                id: alloc_symbol_id(next_id),
                name: function.name.clone(),
                kind: SymbolKind::Function,
                module: module.clone(),
            }),
            Item::Struct(item) => out.push(HirSymbol {
                id: alloc_symbol_id(next_id),
                name: item.name.clone(),
                kind: SymbolKind::Type,
                module: module.clone(),
            }),
            Item::Enum(item) => out.push(HirSymbol {
                id: alloc_symbol_id(next_id),
                name: item.name.clone(),
                kind: SymbolKind::Type,
                module: module.clone(),
            }),
            Item::TypeAlias(item) => out.push(HirSymbol {
                id: alloc_symbol_id(next_id),
                name: item.name.clone(),
                kind: SymbolKind::Type,
                module: module.clone(),
            }),
            Item::Module(item) => {
                out.push(HirSymbol {
                    id: alloc_symbol_id(next_id),
                    name: item.name.clone(),
                    kind: SymbolKind::Module,
                    module: module.clone(),
                });
                let child_module = module.child(item.name.clone());
                collect_symbols(&item.items, &child_module, out, next_id);
            }
            Item::Use(item) => {
                let name = item
                    .alias
                    .clone()
                    .or_else(|| item.path.segments.last().cloned())
                    .unwrap_or_else(|| "<use>".to_string());
                out.push(HirSymbol {
                    id: alloc_symbol_id(next_id),
                    name,
                    kind: SymbolKind::Value,
                    module: module.clone(),
                });
            }
            Item::Impl(item) => collect_symbols(&item.items, module, out, next_id),
            Item::Arena(_) | Item::Stmt(_) => {}
        }
    }
}

fn alloc_symbol_id(next_id: &mut usize) -> SymbolId {
    let id = SymbolId(*next_id);
    *next_id += 1;
    id
}

fn index_symbols_by_name(symbols: &[HirSymbol]) -> BTreeMap<(ModulePath, String), SymbolId> {
    let mut index = BTreeMap::new();
    for symbol in symbols {
        index.insert((symbol.module.clone(), symbol.name.clone()), symbol.id);
    }
    index
}

fn build_module_graph(symbols: &[HirSymbol]) -> ModuleGraph {
    let mut graph = ModuleGraph::new();

    graph.insert_module(Module::new(ModulePath::root()));
    for symbol in symbols {
        graph.insert_module(Module::new(symbol.module.clone()));
    }

    for symbol in symbols {
        if symbol.kind == SymbolKind::Module {
            let child_path = symbol.module.child(symbol.name.clone());
            graph.insert_module(Module::new(child_path.clone()));
            if let Some(module) = graph.module_mut(&symbol.module) {
                module.children.insert(symbol.name.clone(), child_path);
            }
            continue;
        }

        graph.insert_symbol(
            &symbol.module,
            Symbol::new(
                symbol.name.clone(),
                symbol.kind.clone(),
                Visibility::Public,
                symbol.module.clone(),
            ),
        );
    }

    graph
}

fn collect_name_uses(
    items: &[Item],
    resolver: &Resolver,
    id_by_name: &BTreeMap<(ModulePath, String), SymbolId>,
    module: &ModulePath,
    lexical_scope: &mut Vec<(Symbol, SymbolId)>,
    symbols: &mut Vec<HirSymbol>,
    next_id: &mut usize,
    out: &mut Vec<HirNameUse>,
    declarations: &mut Vec<HirDeclaration>,
) {
    for item in items {
        match item {
            Item::Function(function) => {
                let mut local_scope = lexical_scope.clone();
                for param in &function.params {
                    let symbol = Symbol::new(
                        param.name.clone(),
                        SymbolKind::Value,
                        Visibility::Private,
                        module.clone(),
                    );
                    let symbol_id = alloc_symbol_id(next_id);
                    symbols.push(HirSymbol {
                        id: symbol_id,
                        name: param.name.clone(),
                        kind: SymbolKind::Value,
                        module: module.clone(),
                    });
                    declarations.push(HirDeclaration {
                        name: param.name.clone(),
                        module: module.clone(),
                        symbol_id,
                        kind: HirDeclarationKind::Parameter,
                    });
                    local_scope.push((symbol, symbol_id));
                }
                collect_name_uses_from_block(
                    &function.body,
                    resolver,
                    id_by_name,
                    module,
                    &mut local_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
            Item::Module(module_item) => {
                let child_module = module.child(module_item.name.clone());
                let mut module_scope = lexical_scope.clone();
                collect_name_uses(
                    &module_item.items,
                    resolver,
                    id_by_name,
                    &child_module,
                    &mut module_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
            Item::Impl(item) => {
                let mut impl_scope = lexical_scope.clone();
                collect_name_uses(
                    &item.items,
                    resolver,
                    id_by_name,
                    module,
                    &mut impl_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
            Item::Arena(item) => {
                let mut local_scope = lexical_scope.clone();
                collect_name_uses_from_block(
                    &item.body,
                    resolver,
                    id_by_name,
                    module,
                    &mut local_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
            Item::Stmt(stmt) => {
                let mut local_scope = lexical_scope.clone();
                collect_name_uses_from_stmt(
                    stmt,
                    resolver,
                    id_by_name,
                    module,
                    &mut local_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
            Item::Struct(_) | Item::Enum(_) | Item::TypeAlias(_) | Item::Use(_) => {}
        }
    }
}

fn collect_name_uses_from_block(
    block: &Block,
    resolver: &Resolver,
    id_by_name: &BTreeMap<(ModulePath, String), SymbolId>,
    module: &ModulePath,
    lexical_scope: &mut Vec<(Symbol, SymbolId)>,
    symbols: &mut Vec<HirSymbol>,
    next_id: &mut usize,
    out: &mut Vec<HirNameUse>,
    declarations: &mut Vec<HirDeclaration>,
) {
    for stmt in &block.statements {
        collect_name_uses_from_stmt(
            stmt,
            resolver,
            id_by_name,
            module,
            lexical_scope,
            symbols,
            next_id,
            out,
            declarations,
        );
    }

    if let Some(tail) = block.tail.as_ref() {
        collect_name_uses_from_expr(
            tail,
            resolver,
            id_by_name,
            module,
            lexical_scope,
            symbols,
            next_id,
            out,
            declarations,
        );
    }
}

fn collect_name_uses_from_stmt(
    stmt: &Stmt,
    resolver: &Resolver,
    id_by_name: &BTreeMap<(ModulePath, String), SymbolId>,
    module: &ModulePath,
    lexical_scope: &mut Vec<(Symbol, SymbolId)>,
    symbols: &mut Vec<HirSymbol>,
    next_id: &mut usize,
    out: &mut Vec<HirNameUse>,
    declarations: &mut Vec<HirDeclaration>,
) {
    match stmt {
        Stmt::Let { name, value, .. } => {
            if let Some(value) = value {
                collect_name_uses_from_expr(
                    value,
                    resolver,
                    id_by_name,
                    module,
                    lexical_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }

            let symbol = Symbol::new(
                name.clone(),
                SymbolKind::Value,
                Visibility::Private,
                module.clone(),
            );
            let symbol_id = alloc_symbol_id(next_id);
            symbols.push(HirSymbol {
                id: symbol_id,
                name: name.clone(),
                kind: SymbolKind::Value,
                module: module.clone(),
            });
            declarations.push(HirDeclaration {
                name: name.clone(),
                module: module.clone(),
                symbol_id,
                kind: HirDeclarationKind::Let,
            });

            lexical_scope.push((symbol, symbol_id));
        }
        Stmt::Return(value) => {
            if let Some(value) = value {
                collect_name_uses_from_expr(
                    value,
                    resolver,
                    id_by_name,
                    module,
                    lexical_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
        }
        Stmt::Expr(expr) => {
            collect_name_uses_from_expr(
                expr,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            )
        }
    }
}

fn collect_name_uses_from_expr(
    expr: &Expr,
    resolver: &Resolver,
    id_by_name: &BTreeMap<(ModulePath, String), SymbolId>,
    module: &ModulePath,
    lexical_scope: &[(Symbol, SymbolId)],
    symbols: &mut Vec<HirSymbol>,
    next_id: &mut usize,
    out: &mut Vec<HirNameUse>,
    declarations: &mut Vec<HirDeclaration>,
) {
    match expr {
        Expr::Identifier(name) => {
            out.push(resolve_name_use(name, resolver, id_by_name, module, lexical_scope));
        }
        Expr::Path(path) => {
            if let Some(name) = path.segments.last() {
                out.push(resolve_name_use(name, resolver, id_by_name, module, lexical_scope));
            }
        }
        Expr::Block(block) => {
            let mut nested_scope = lexical_scope.to_vec();
            collect_name_uses_from_block(
                block,
                resolver,
                id_by_name,
                module,
                &mut nested_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_name_uses_from_expr(
                condition,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            let mut then_scope = lexical_scope.to_vec();
            collect_name_uses_from_block(
                then_branch,
                resolver,
                id_by_name,
                module,
                &mut then_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            if let Some(else_branch) = else_branch {
                let mut else_scope = lexical_scope.to_vec();
                collect_name_uses_from_block(
                    else_branch,
                    resolver,
                    id_by_name,
                    module,
                    &mut else_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
        }
        Expr::While { condition, body } => {
            collect_name_uses_from_expr(
                condition,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            let mut body_scope = lexical_scope.to_vec();
            collect_name_uses_from_block(
                body,
                resolver,
                id_by_name,
                module,
                &mut body_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::For {
            binding,
            iterable,
            body,
        } => {
            collect_name_uses_from_expr(
                iterable,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            let mut body_scope = lexical_scope.to_vec();
            let symbol = Symbol::new(
                binding.clone(),
                SymbolKind::Value,
                Visibility::Private,
                module.clone(),
            );
            let symbol_id = alloc_symbol_id(next_id);
            symbols.push(HirSymbol {
                id: symbol_id,
                name: binding.clone(),
                kind: SymbolKind::Value,
                module: module.clone(),
            });
            declarations.push(HirDeclaration {
                name: binding.clone(),
                module: module.clone(),
                symbol_id,
                kind: HirDeclarationKind::ForBinding,
            });
            body_scope.push((symbol, symbol_id));
            collect_name_uses_from_block(
                body,
                resolver,
                id_by_name,
                module,
                &mut body_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::Match { value, arms } => {
            collect_name_uses_from_expr(
                value,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            for arm in arms {
                collect_name_uses_from_expr(
                    &arm.value,
                    resolver,
                    id_by_name,
                    module,
                    lexical_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
        }
        Expr::Arena { body, .. } => {
            let mut body_scope = lexical_scope.to_vec();
            collect_name_uses_from_block(
                body,
                resolver,
                id_by_name,
                module,
                &mut body_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::Call { callee, args } => {
            collect_name_uses_from_expr(
                callee,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            for arg in args {
                collect_name_uses_from_expr(
                    arg,
                    resolver,
                    id_by_name,
                    module,
                    lexical_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
        }
        Expr::Field { target, .. } => {
            collect_name_uses_from_expr(
                target,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            )
        }
        Expr::Index { target, index } => {
            collect_name_uses_from_expr(
                target,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            collect_name_uses_from_expr(
                index,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::Try(inner) | Expr::Unary { expr: inner, .. } => {
            collect_name_uses_from_expr(
                inner,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::Binary { left, right, .. } | Expr::Range { start: left, end: right } => {
            collect_name_uses_from_expr(
                left,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
            collect_name_uses_from_expr(
                right,
                resolver,
                id_by_name,
                module,
                lexical_scope,
                symbols,
                next_id,
                out,
                declarations,
            );
        }
        Expr::Tuple(values) => {
            for value in values {
                collect_name_uses_from_expr(
                    value,
                    resolver,
                    id_by_name,
                    module,
                    lexical_scope,
                    symbols,
                    next_id,
                    out,
                    declarations,
                );
            }
        }
        Expr::Integer(_) | Expr::Float(_) | Expr::Boolean(_) | Expr::String(_) | Expr::Char(_) => {}
    }
}

fn resolve_name_use(
    name: &str,
    resolver: &Resolver,
    id_by_name: &BTreeMap<(ModulePath, String), SymbolId>,
    module: &ModulePath,
    lexical_scope: &[(Symbol, SymbolId)],
) -> HirNameUse {
    let lexical_symbols: Vec<Symbol> = lexical_scope.iter().map(|(symbol, _)| symbol.clone()).collect();

    let resolved = resolver.resolve_name(&lexical_symbols, module, &[], name);
    let symbol_id = resolved.as_ref().and_then(|resolved_symbol| {
        if resolved_symbol.scope == ResolutionScope::Lexical {
            lexical_scope
                .iter()
                .rev()
                .find(|(symbol, _)| symbol.name == name)
                .map(|(_, symbol_id)| *symbol_id)
        } else {
            symbol_id_for_resolved_symbol(id_by_name, resolved_symbol)
        }
    });

    HirNameUse {
        name: name.to_string(),
        module: module.clone(),
        scope: resolved.as_ref().map(|resolved_symbol| resolved_symbol.scope.clone()),
        symbol_id,
    }
}

fn symbol_id_for_resolved_symbol(
    id_by_name: &BTreeMap<(ModulePath, String), SymbolId>,
    resolved: &ResolvedSymbol,
) -> Option<SymbolId> {
    id_by_name
        .get(&(resolved.symbol.module.clone(), resolved.symbol.name.clone()))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{
        Block, Expr, FunctionItem, FunctionKind, Item, ModuleItem, Program, Stmt, TypeAliasItem, TypeExpr,
        Visibility,
    };

    #[test]
    fn build_hir_collects_top_level_and_nested_symbols() {
        let program = Program::new(vec![
            Item::Function(FunctionItem {
                decorators: Vec::new(),
                visibility: Visibility::Private,
                kind: FunctionKind::Fn,
                name: "main".to_string(),
                generics: Vec::new(),
                params: Vec::new(),
                return_type: None,
                body: Block::new(vec![Stmt::Expr(Expr::Integer(1))], None),
            }),
            Item::Module(ModuleItem {
                decorators: Vec::new(),
                visibility: Visibility::Private,
                name: "math".to_string(),
                items: vec![Item::TypeAlias(TypeAliasItem {
                    decorators: Vec::new(),
                    visibility: Visibility::Public,
                    name: "Scalar".to_string(),
                    generics: Vec::new(),
                    target: TypeExpr::Path(crate::frontend::ast::PathExpr::new(vec!["float".to_string()], Vec::new())),
                })],
            }),
        ]);

        let hir = build_hir(&program, ModulePath::root());
        assert_eq!(hir.symbols.len(), 3);
        assert!(hir.symbols.iter().any(|symbol| symbol.name == "main" && symbol.kind == SymbolKind::Function));
        assert!(hir.symbols.iter().any(|symbol| symbol.name == "math" && symbol.kind == SymbolKind::Module));
        assert!(hir
            .symbols
            .iter()
            .any(|symbol| symbol.name == "Scalar" && symbol.kind == SymbolKind::Type));
    }

    #[test]
    fn build_hir_resolves_identifier_name_uses() {
        let program = Program::new(vec![Item::Function(FunctionItem {
            decorators: Vec::new(),
            visibility: Visibility::Private,
            kind: FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: None,
            body: Block::new(
                vec![Stmt::Let {
                    name: "x".to_string(),
                    mutable: false,
                    value: Some(Expr::Integer(1)),
                }],
                Some(Expr::Identifier("x".to_string())),
            ),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        assert!(hir
            .name_uses
            .iter()
            .any(|name_use| name_use.name == "x" && name_use.scope == Some(ResolutionScope::Lexical)));
    }
}