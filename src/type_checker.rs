use crate::diagnostics::Diagnostics;
use crate::frontend::ast::{
    BinaryOp, Block, Expr, FunctionItem, FunctionKind, GenericParam, Item, MatchArm, Pattern, Program, Stmt,
    TypeExpr, UnaryOp,
};
use crate::hir::{HirDeclaration, HirNameUse, ResolvedProgram, SymbolId};
use crate::resolution::ModulePath;
use crate::types::{
    Binding, BindingKind, FunctionContext, FunctionType, GenericType, Mutability, Ownership,
    ParameterType, Place, PrimitiveType, Type, TypeEnvironment, TypeId, TypeStore,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedProgram {
    pub items: Vec<CheckedItem>,
    pub diagnostics: Diagnostics,
    pub types: TypeStore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckedItem {
    Function(CheckedFunctionItem),
    Struct(CheckedStructItem),
    Enum(CheckedEnumItem),
    TypeAlias(CheckedTypeAliasItem),
    Impl(CheckedImplItem),
    Module(CheckedModuleItem),
    Use(CheckedUseItem),
    Arena(CheckedArenaItem),
    Stmt(CheckedStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedFunctionItem {
    pub name: String,
    pub signature: FunctionType,
    pub param_symbol_ids: Vec<Option<SymbolId>>,
    pub body: CheckedBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStructItem {
    pub name: String,
    pub fields: Vec<(String, TypeId)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedEnumItem {
    pub name: String,
    pub variants: Vec<(String, Vec<TypeId>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedTypeAliasItem {
    pub name: String,
    pub target: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedImplItem {
    pub target: TypeId,
    pub items: Vec<CheckedItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedModuleItem {
    pub name: String,
    pub items: Vec<CheckedItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedUseItem {
    pub path: Vec<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedArenaItem {
    pub name: String,
    pub body: CheckedBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStmt {
    pub kind: CheckedStmtKind,
    pub ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckedStmtKind {
    Let {
        name: String,
        mutable: bool,
        symbol_id: Option<SymbolId>,
        value: Option<CheckedExpr>,
    },
    Return(Option<CheckedExpr>),
    Expr(CheckedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedBlock {
    pub statements: Vec<CheckedStmt>,
    pub tail: Option<Box<CheckedExpr>>,
    pub ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedExpr {
    pub kind: CheckedExprKind,
    pub ty: TypeId,
    pub symbol_id: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckedExprKind {
    Identifier(String),
    Integer(i64),
    Float(String),
    Boolean(bool),
    String(String),
    Char(char),
    Path(Vec<String>),
    Block(CheckedBlock),
    If {
        condition: Box<CheckedExpr>,
        then_branch: CheckedBlock,
        else_branch: Option<CheckedBlock>,
    },
    While {
        condition: Box<CheckedExpr>,
        body: CheckedBlock,
    },
    For {
        binding: String,
        binding_symbol_id: Option<SymbolId>,
        iterable: Box<CheckedExpr>,
        body: CheckedBlock,
    },
    Match {
        value: Box<CheckedExpr>,
        arms: Vec<CheckedMatchArm>,
    },
    Arena {
        name: String,
        body: CheckedBlock,
    },
    Call {
        callee: Box<CheckedExpr>,
        args: Vec<CheckedExpr>,
    },
    Field {
        target: Box<CheckedExpr>,
        name: String,
    },
    Index {
        target: Box<CheckedExpr>,
        index: Box<CheckedExpr>,
    },
    Try(Box<CheckedExpr>),
    Unary {
        op: UnaryOp,
        expr: Box<CheckedExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<CheckedExpr>,
        right: Box<CheckedExpr>,
    },
    Tuple(Vec<CheckedExpr>),
    Range {
        start: Box<CheckedExpr>,
        end: Box<CheckedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedMatchArm {
    pub pattern: Pattern,
    pub value: CheckedExpr,
}

#[derive(Debug, Clone)]
pub struct TypeChecker {
    pub env: TypeEnvironment,
    pub types: TypeStore,
    pub diagnostics: Diagnostics,
    hir_name_uses: Vec<HirNameUse>,
    hir_name_use_index: usize,
    hir_declarations: Vec<HirDeclaration>,
    hir_declaration_index: usize,
}

impl TypeChecker {
    pub fn new(module: ModulePath) -> Self {
        let mut types = TypeStore::new();
        let mut env = TypeEnvironment::new(module);
        let diagnostics = Diagnostics::new();

        let _ = types.intern(Type::Error);
        for builtin in ["unit", "never", "bool", "int", "float", "string", "char"] {
            if let Some(ty) = types.named(builtin) {
                env.declare_type(builtin.to_string(), ty);
            }
        }

        Self {
            env,
            types,
            diagnostics,
            hir_name_uses: Vec::new(),
            hir_name_use_index: 0,
            hir_declarations: Vec::new(),
            hir_declaration_index: 0,
        }
    }

    pub fn check_program(mut self, program: &Program) -> CheckedProgram {
        self.predeclare_program(program);

        let items = program.items.iter().map(|item| self.check_item(item)).collect();

        CheckedProgram {
            items,
            diagnostics: self.diagnostics,
            types: self.types,
        }
    }

    pub fn check_hir(mut self, hir: &ResolvedProgram) -> CheckedProgram {
        self.hir_name_uses = hir.name_uses.clone();
        self.hir_name_use_index = 0;
        self.hir_declarations = hir.declarations.clone();
        self.hir_declaration_index = 0;
        self.check_program(&hir.program)
    }

    fn predeclare_program(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Function(function) => {
                    let signature = self.function_signature(function);
                    let ty = if function.kind == FunctionKind::Task {
                        self.types.intern(Type::Task(signature.clone()))
                    } else {
                        self.types.intern(Type::Function(signature.clone()))
                    };

                    self.env.declare_binding(Binding {
                        name: function.name.clone(),
                        ty,
                        mutability: Mutability::Immutable,
                        ownership: Ownership::Owned,
                        kind: BindingKind::Global,
                        module: self.env.module.clone(),
                        place: Place::Binding {
                            name: function.name.clone(),
                        },
                    });
                }
                Item::Struct(item) => {
                    let ty = self.types.intern(Type::Named {
                        path: vec![item.name.clone()],
                        arguments: Vec::new(),
                    });
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::Enum(item) => {
                    let ty = self.types.intern(Type::Named {
                        path: vec![item.name.clone()],
                        arguments: Vec::new(),
                    });
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::TypeAlias(item) => {
                    let ty = self.resolve_type_expr(&item.target);
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::Module(item) => {
                    let ty = self.types.intern(Type::Named {
                        path: vec![item.name.clone()],
                        arguments: Vec::new(),
                    });
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::Impl(_) | Item::Use(_) | Item::Arena(_) | Item::Stmt(_) => {}
            }
        }
    }

    fn check_item(&mut self, item: &Item) -> CheckedItem {
        match item {
            Item::Function(function) => CheckedItem::Function(self.check_function_item(function)),
            Item::Struct(item) => CheckedItem::Struct(self.check_struct_item(item)),
            Item::Enum(item) => CheckedItem::Enum(self.check_enum_item(item)),
            Item::TypeAlias(item) => CheckedItem::TypeAlias(self.check_type_alias_item(item)),
            Item::Impl(item) => CheckedItem::Impl(self.check_impl_item(item)),
            Item::Module(item) => CheckedItem::Module(self.check_module_item(item)),
            Item::Use(item) => CheckedItem::Use(self.check_use_item(item)),
            Item::Arena(item) => CheckedItem::Arena(self.check_arena_item(item)),
            Item::Stmt(stmt) => CheckedItem::Stmt(self.check_stmt(stmt)),
        }
    }

    fn check_function_item(&mut self, item: &FunctionItem) -> CheckedFunctionItem {
        let signature = self.function_signature(item);
        let return_type = signature.return_type;
        let previous_expected_return = self.env.expected_return;
        let previous_task_context = self.env.in_task_context;

        self.env.push_scope();
        let previous_function = self.env.current_function.clone();
        self.env.current_function = Some(FunctionContext {
            name: item.name.clone(),
            module: self.env.module.clone(),
            signature: signature.clone(),
            generics: self.check_generics(&item.generics),
            is_public: matches!(item.visibility, crate::frontend::ast::Visibility::Public),
            is_task: item.kind == FunctionKind::Task,
        });
        self.env.expected_return = Some(return_type);
        self.env.in_task_context = item.kind == FunctionKind::Task;

        let mut param_symbol_ids = Vec::with_capacity(item.params.len());
        for param in &item.params {
            let symbol_id = self.consume_hir_declaration(&param.name);
            param_symbol_ids.push(symbol_id);
            let param_ty = param
                .ty
                .as_ref()
                .map(|ty| self.resolve_type_expr(ty))
                .unwrap_or_else(|| self.error_type());

            self.env.declare_binding(Binding {
                name: param.name.clone(),
                ty: param_ty,
                mutability: Mutability::Immutable,
                ownership: Ownership::Owned,
                kind: BindingKind::Parameter,
                module: self.env.module.clone(),
                place: Place::Binding {
                    name: param.name.clone(),
                },
            });
        }

        let body = self.check_block(&item.body);
        let expected_return = self.env.expected_return;
        self.env.pop_scope();
        self.env.current_function = previous_function;
        self.env.expected_return = previous_expected_return;
        self.env.in_task_context = previous_task_context;

        if let Some(expected) = expected_return {
            if body.ty != expected {
                self.diagnostics.error(
                    "AXIS-TC-001",
                    format!("function '{}' body type does not match the expected return type", item.name),
                );
            }
        }

        CheckedFunctionItem {
            name: item.name.clone(),
            signature,
            param_symbol_ids,
            body,
        }
    }

    fn check_struct_item(&mut self, item: &crate::frontend::ast::StructItem) -> CheckedStructItem {
        let fields = item
            .fields
            .iter()
            .map(|field| (field.name.clone(), self.resolve_type_expr(&field.ty)))
            .collect();

        CheckedStructItem {
            name: item.name.clone(),
            fields,
        }
    }

    fn check_enum_item(&mut self, item: &crate::frontend::ast::EnumItem) -> CheckedEnumItem {
        let variants = item
            .variants
            .iter()
            .map(|variant| {
                (
                    variant.name.clone(),
                    variant.fields.iter().map(|field| self.resolve_type_expr(field)).collect(),
                )
            })
            .collect();

        CheckedEnumItem {
            name: item.name.clone(),
            variants,
        }
    }

    fn check_type_alias_item(&mut self, item: &crate::frontend::ast::TypeAliasItem) -> CheckedTypeAliasItem {
        CheckedTypeAliasItem {
            name: item.name.clone(),
            target: self.resolve_type_expr(&item.target),
        }
    }

    fn check_impl_item(&mut self, item: &crate::frontend::ast::ImplItem) -> CheckedImplItem {
        CheckedImplItem {
            target: self.resolve_type_expr(&item.target),
            items: item.items.iter().map(|item| self.check_item(item)).collect(),
        }
    }

    fn check_module_item(&mut self, item: &crate::frontend::ast::ModuleItem) -> CheckedModuleItem {
        let previous_module = self.env.module.clone();
        self.env.module = previous_module.child(item.name.clone());
        self.env.push_scope();
        self.predeclare_items(&item.items);
        let items = item.items.iter().map(|item| self.check_item(item)).collect();
        self.env.pop_scope();
        self.env.module = previous_module;

        CheckedModuleItem {
            name: item.name.clone(),
            items,
        }
    }

    fn check_use_item(&mut self, item: &crate::frontend::ast::UseItem) -> CheckedUseItem {
        CheckedUseItem {
            path: item.path.segments.clone(),
            alias: item.alias.clone(),
        }
    }

    fn check_arena_item(&mut self, item: &crate::frontend::ast::ArenaItem) -> CheckedArenaItem {
        CheckedArenaItem {
            name: item.name.clone(),
            body: self.check_block(&item.body),
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> CheckedStmt {
        match stmt {
            Stmt::Let { name, mutable, value } => {
                let symbol_id = self.consume_hir_declaration(name);
                let value = value.as_ref().map(|expr| self.check_expr(expr));
                let ty = value.as_ref().map(|expr| expr.ty).unwrap_or_else(|| self.unit_type());

                self.env.declare_binding(Binding {
                    name: name.clone(),
                    ty,
                    mutability: if *mutable { Mutability::Mutable } else { Mutability::Immutable },
                    ownership: Ownership::Owned,
                    kind: BindingKind::Local,
                    module: self.env.module.clone(),
                    place: Place::Binding {
                        name: name.clone(),
                    },
                });

                CheckedStmt {
                    kind: CheckedStmtKind::Let {
                        name: name.clone(),
                        mutable: *mutable,
                        symbol_id,
                        value,
                    },
                    ty,
                }
            }
            Stmt::Return(value) => {
                let value = value.as_ref().map(|expr| self.check_expr(expr));
                let ty = value.as_ref().map(|expr| expr.ty).unwrap_or_else(|| self.unit_type());

                if let Some(expected) = self.env.expected_return {
                    if ty != expected && !matches!(self.types.get(ty), Some(Type::Error)) {
                        self.diagnostics.error(
                            "AXIS-TC-002",
                            "return type does not match the current function signature",
                        );
                    }
                }

                CheckedStmt {
                    kind: CheckedStmtKind::Return(value),
                    ty,
                }
            }
            Stmt::Expr(expr) => {
                let expr = self.check_expr(expr);
                CheckedStmt {
                    kind: CheckedStmtKind::Expr(expr.clone()),
                    ty: expr.ty,
                }
            }
        }
    }

    fn check_block(&mut self, block: &Block) -> CheckedBlock {
        self.env.push_scope();

        let statements = block.statements.iter().map(|stmt| self.check_stmt(stmt)).collect::<Vec<_>>();
        let tail = block.tail.as_ref().map(|expr| Box::new(self.check_expr(expr)));
        let ty = tail.as_ref().map(|expr| expr.ty).unwrap_or_else(|| self.unit_type());

        self.env.pop_scope();

        CheckedBlock {
            statements,
            tail,
            ty,
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> CheckedExpr {
        match expr {
            Expr::Identifier(name) => self.check_identifier_expr(name),
            Expr::Integer(value) => CheckedExpr {
                kind: CheckedExprKind::Integer(*value),
                ty: self.int_type(),
                symbol_id: None,
            },
            Expr::Float(text) => CheckedExpr {
                kind: CheckedExprKind::Float(text.clone()),
                ty: self.float_type(),
                symbol_id: None,
            },
            Expr::Boolean(value) => CheckedExpr {
                kind: CheckedExprKind::Boolean(*value),
                ty: self.bool_type(),
                symbol_id: None,
            },
            Expr::String(value) => CheckedExpr {
                kind: CheckedExprKind::String(value.clone()),
                ty: self.string_type(),
                symbol_id: None,
            },
            Expr::Char(value) => CheckedExpr {
                kind: CheckedExprKind::Char(*value),
                ty: self.char_type(),
                symbol_id: None,
            },
            Expr::Path(path) => self.check_path_expr(path),
            Expr::Block(block) => {
                let block = self.check_block(block);
                let ty = block.ty;
                CheckedExpr {
                    kind: CheckedExprKind::Block(block),
                    ty,
                    symbol_id: None,
                }
            }
            Expr::If { condition, then_branch, else_branch } => {
                let condition = Box::new(self.check_expr(condition));
                let then_branch = self.check_block(then_branch);
                let else_branch = else_branch.as_ref().map(|branch| self.check_block(branch));
                let ty = else_branch.as_ref().map(|branch| branch.ty).unwrap_or(then_branch.ty);
                CheckedExpr {
                    kind: CheckedExprKind::If {
                        condition,
                        then_branch,
                        else_branch,
                    },
                    ty,
                    symbol_id: None,
                }
            }
            Expr::While { condition, body } => CheckedExpr {
                kind: CheckedExprKind::While {
                    condition: Box::new(self.check_expr(condition)),
                    body: self.check_block(body),
                },
                ty: self.unit_type(),
                symbol_id: None,
            },
            Expr::For { binding, iterable, body } => {
                let iterable = Box::new(self.check_expr(iterable));
                let binding_ty = self.error_type();
                let binding_symbol_id = self.consume_hir_declaration(binding);
                self.env.push_scope();
                self.env.declare_binding(Binding {
                    name: binding.clone(),
                    ty: binding_ty,
                    mutability: Mutability::Immutable,
                    ownership: Ownership::Owned,
                    kind: BindingKind::Local,
                    module: self.env.module.clone(),
                    place: Place::Binding {
                        name: binding.clone(),
                    },
                });
                let body = self.check_block(body);
                self.env.pop_scope();
                CheckedExpr {
                    kind: CheckedExprKind::For {
                        binding: binding.clone(),
                        binding_symbol_id,
                        iterable,
                        body,
                    },
                    ty: self.unit_type(),
                    symbol_id: None,
                }
            }
            Expr::Match { value, arms } => {
                let value = Box::new(self.check_expr(value));
                let arms = arms.iter().map(|arm| self.check_match_arm(arm)).collect::<Vec<_>>();
                let ty = arms.first().map(|arm| arm.value.ty).unwrap_or_else(|| self.unit_type());
                CheckedExpr {
                    kind: CheckedExprKind::Match { value, arms },
                    ty,
                    symbol_id: None,
                }
            }
            Expr::Arena { name, body } => CheckedExpr {
                kind: CheckedExprKind::Arena {
                    name: name.clone(),
                    body: self.check_block(body),
                },
                ty: self.unit_type(),
                symbol_id: None,
            },
            Expr::Call { callee, args } => {
                let callee = Box::new(self.check_expr(callee));
                let args = args.iter().map(|arg| self.check_expr(arg)).collect::<Vec<_>>();
                let ty = self.call_result_type(callee.ty);
                CheckedExpr {
                    kind: CheckedExprKind::Call { callee, args },
                    ty,
                    symbol_id: None,
                }
            }
            Expr::Field { target, name } => CheckedExpr {
                kind: CheckedExprKind::Field {
                    target: Box::new(self.check_expr(target)),
                    name: name.clone(),
                },
                ty: self.error_type(),
                symbol_id: None,
            },
            Expr::Index { target, index } => CheckedExpr {
                kind: CheckedExprKind::Index {
                    target: Box::new(self.check_expr(target)),
                    index: Box::new(self.check_expr(index)),
                },
                ty: self.error_type(),
                symbol_id: None,
            },
            Expr::Try(expr) => {
                let inner = Box::new(self.check_expr(expr));
                CheckedExpr {
                    ty: inner.ty,
                    kind: CheckedExprKind::Try(inner),
                    symbol_id: None,
                }
            }
            Expr::Unary { op, expr } => CheckedExpr {
                ty: self.unary_result_type(op, expr),
                kind: CheckedExprKind::Unary {
                    op: op.clone(),
                    expr: Box::new(self.check_expr(expr)),
                },
                symbol_id: None,
            },
            Expr::Binary { op, left, right } => {
                let left = Box::new(self.check_expr(left));
                let right = Box::new(self.check_expr(right));
                let ty = self.binary_result_type(op, left.ty, right.ty);
                CheckedExpr {
                    kind: CheckedExprKind::Binary {
                        op: op.clone(),
                        left,
                        right,
                    },
                    ty,
                    symbol_id: None,
                }
            }
            Expr::Tuple(values) => {
                let values = values.iter().map(|value| self.check_expr(value)).collect::<Vec<_>>();
                let member_types = values.iter().map(|value| value.ty).collect::<Vec<_>>();
                let ty = self.types.intern(Type::Tuple(member_types));
                CheckedExpr {
                    kind: CheckedExprKind::Tuple(values),
                    ty,
                    symbol_id: None,
                }
            }
            Expr::Range { start, end } => {
                let start = Box::new(self.check_expr(start));
                let end = Box::new(self.check_expr(end));
                let ty = self.types.intern(Type::Named {
                    path: vec!["Range".to_string()],
                    arguments: vec![start.ty, end.ty],
                });
                CheckedExpr {
                    kind: CheckedExprKind::Range { start, end },
                    ty,
                    symbol_id: None,
                }
            }
        }
    }

    fn consume_hir_name_use(&mut self, expected_name: &str) -> Option<SymbolId> {
        if self.hir_name_uses.is_empty() {
            return None;
        }

        if let Some(current) = self.hir_name_uses.get(self.hir_name_use_index) {
            if current.name == expected_name {
                self.hir_name_use_index += 1;
                return current.symbol_id;
            }
        }

        if let Some(offset) = self.hir_name_uses[self.hir_name_use_index..]
            .iter()
            .position(|name_use| name_use.name == expected_name)
        {
            let index = self.hir_name_use_index + offset;
            self.hir_name_use_index = index + 1;
            return self.hir_name_uses[index].symbol_id;
        }

        None
    }

    fn consume_hir_declaration(&mut self, expected_name: &str) -> Option<SymbolId> {
        if self.hir_declarations.is_empty() {
            return None;
        }

        if let Some(current) = self.hir_declarations.get(self.hir_declaration_index) {
            if current.name == expected_name {
                self.hir_declaration_index += 1;
                return Some(current.symbol_id);
            }
        }

        if let Some(offset) = self.hir_declarations[self.hir_declaration_index..]
            .iter()
            .position(|declaration| declaration.name == expected_name)
        {
            let index = self.hir_declaration_index + offset;
            self.hir_declaration_index = index + 1;
            return Some(self.hir_declarations[index].symbol_id);
        }

        None
    }

    fn check_identifier_expr(&mut self, name: &str) -> CheckedExpr {
        let symbol_id = self.consume_hir_name_use(name);
        if let Some(binding) = self.env.lookup_binding(name) {
            return CheckedExpr {
                kind: CheckedExprKind::Identifier(name.to_string()),
                ty: binding.ty,
                symbol_id,
            };
        }

        if let Some(ty) = self.env.lookup_type(name) {
            return CheckedExpr {
                kind: CheckedExprKind::Identifier(name.to_string()),
                ty,
                symbol_id,
            };
        }

        self.diagnostics.error("AXIS-TC-003", format!("unknown identifier '{name}'"));
        CheckedExpr {
            kind: CheckedExprKind::Identifier(name.to_string()),
            ty: self.error_type(),
            symbol_id,
        }
    }

    fn check_path_expr(&mut self, path: &crate::frontend::ast::PathExpr) -> CheckedExpr {
        if let Some(last) = path.segments.last() {
            let symbol_id = self.consume_hir_name_use(last);
            if let Some(ty) = self.env.lookup_type(last) {
                return CheckedExpr {
                    kind: CheckedExprKind::Path(path.segments.clone()),
                    ty,
                    symbol_id,
                };
            }

            if let Some(binding) = self.env.lookup_binding(last) {
                return CheckedExpr {
                    kind: CheckedExprKind::Path(path.segments.clone()),
                    ty: binding.ty,
                    symbol_id,
                };
            }

            return CheckedExpr {
                kind: CheckedExprKind::Path(path.segments.clone()),
                ty: self.error_type(),
                symbol_id,
            };
        }

        CheckedExpr {
            kind: CheckedExprKind::Path(path.segments.clone()),
            ty: self.error_type(),
            symbol_id: None,
        }
    }

    fn check_match_arm(&mut self, arm: &MatchArm) -> CheckedMatchArm {
        CheckedMatchArm {
            pattern: arm.pattern.clone(),
            value: self.check_expr(&arm.value),
        }
    }

    fn check_generics(&mut self, generics: &[GenericParam]) -> Vec<GenericType> {
        generics
            .iter()
            .map(|generic| {
                let bounds: Vec<TypeId> = generic
                    .bounds
                    .iter()
                    .map(|bound| self.resolve_type_expr(bound))
                    .collect();
                let ty = self.types.intern(Type::Generic(GenericType {
                    name: generic.name.clone(),
                    bounds: bounds.clone(),
                }));
                self.env.declare_local_type(generic.name.clone(), ty);
                GenericType {
                    name: generic.name.clone(),
                    bounds,
                }
            })
            .collect()
    }

    fn predeclare_items(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Function(function) => {
                    let signature = self.function_signature(function);
                    let ty = if function.kind == FunctionKind::Task {
                        self.types.intern(Type::Task(signature.clone()))
                    } else {
                        self.types.intern(Type::Function(signature.clone()))
                    };

                    self.env.declare_binding(Binding {
                        name: function.name.clone(),
                        ty,
                        mutability: Mutability::Immutable,
                        ownership: Ownership::Owned,
                        kind: BindingKind::Global,
                        module: self.env.module.clone(),
                        place: Place::Binding {
                            name: function.name.clone(),
                        },
                    });
                }
                Item::Struct(item) => {
                    let ty = self.types.intern(Type::Named {
                        path: vec![item.name.clone()],
                        arguments: Vec::new(),
                    });
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::Enum(item) => {
                    let ty = self.types.intern(Type::Named {
                        path: vec![item.name.clone()],
                        arguments: Vec::new(),
                    });
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::TypeAlias(item) => {
                    let ty = self.resolve_type_expr(&item.target);
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::Module(item) => {
                    let ty = self.types.intern(Type::Named {
                        path: vec![item.name.clone()],
                        arguments: Vec::new(),
                    });
                    self.env.declare_type(item.name.clone(), ty);
                }
                Item::Impl(_) | Item::Use(_) | Item::Arena(_) | Item::Stmt(_) => {}
            }
        }
    }

    fn function_signature(&mut self, item: &FunctionItem) -> FunctionType {
        let parameters = item
            .params
            .iter()
            .map(|param| ParameterType {
                name: Some(param.name.clone()),
                ty: param.ty.as_ref().map(|ty| self.resolve_type_expr(ty)).unwrap_or_else(|| self.error_type()),
                mutability: Mutability::Immutable,
            })
            .collect();

        let return_type = item
            .return_type
            .as_ref()
            .map(|ty| self.resolve_type_expr(ty))
            .unwrap_or_else(|| self.unit_type());

        FunctionType {
            parameters,
            return_type,
            captures: Vec::new(),
            is_task: item.kind == FunctionKind::Task,
        }
    }

    fn resolve_type_expr(&mut self, ty: &TypeExpr) -> TypeId {
        match ty {
            TypeExpr::Unit => self.unit_type(),
            TypeExpr::Path(path) => {
                if path.segments.len() == 1 {
                    if let Some(ty) = self.env.lookup_type(&path.segments[0]) {
                        return ty;
                    }
                }

                let arguments = path.arguments.iter().map(|arg| self.resolve_type_expr(arg)).collect();
                self.types.intern(Type::Named {
                    path: path.segments.clone(),
                    arguments,
                })
            }
            TypeExpr::Reference { mutable, target } => {
                let target = self.resolve_type_expr(target);
                self.types.intern(Type::Reference {
                    mutable: *mutable,
                    target,
                })
            }
            TypeExpr::Tuple(values) => {
                let values = values.iter().map(|value| self.resolve_type_expr(value)).collect();
                self.types.intern(Type::Tuple(values))
            }
            TypeExpr::Array { element, length } => {
                let element = self.resolve_type_expr(element);
                self.types.intern(Type::Array {
                    element,
                    length: *length,
                })
            }
        }
    }

    fn call_result_type(&mut self, callee: TypeId) -> TypeId {
        match self.types.get(callee) {
            Some(Type::Function(function)) | Some(Type::Task(function)) => function.return_type,
            Some(Type::Named { path, .. }) if path.last().is_some_and(|segment| segment == "Result") => {
                self.error_type()
            }
            _ => self.error_type(),
        }
    }

    fn binary_result_type(&mut self, op: &BinaryOp, left: TypeId, right: TypeId) -> TypeId {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if left == right && self.is_numeric_type(left) {
                    left
                } else {
                    self.error_type()
                }
            }
            BinaryOp::Range => self.types.intern(Type::Named {
                path: vec!["Range".to_string()],
                arguments: vec![left, right],
            }),
            BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::EqualEqual
            | BinaryOp::NotEqual => self.bool_type(),
        }
    }

    fn unary_result_type(&mut self, op: &UnaryOp, expr: &Expr) -> TypeId {
        match op {
            UnaryOp::Negate => {
                let ty = self.check_expr(expr).ty;
                if self.is_numeric_type(ty) { ty } else { self.error_type() }
            }
        }
    }

    fn is_numeric_type(&self, ty: TypeId) -> bool {
        matches!(self.types.get(ty), Some(Type::Primitive(PrimitiveType::Int | PrimitiveType::Float)) | Some(Type::Numeric(_)))
    }

    fn unit_type(&mut self) -> TypeId {
        self.types.named("unit").unwrap_or_else(|| self.types.intern(Type::Unit))
    }

    fn bool_type(&mut self) -> TypeId {
        self.types.named("bool").unwrap_or_else(|| self.types.intern(Type::Primitive(PrimitiveType::Bool)))
    }

    fn int_type(&mut self) -> TypeId {
        self.types.named("int").unwrap_or_else(|| self.types.intern(Type::Primitive(PrimitiveType::Int)))
    }

    fn float_type(&mut self) -> TypeId {
        self.types.named("float").unwrap_or_else(|| self.types.intern(Type::Primitive(PrimitiveType::Float)))
    }

    fn string_type(&mut self) -> TypeId {
        self.types.named("string").unwrap_or_else(|| self.types.intern(Type::Primitive(PrimitiveType::String)))
    }

    fn char_type(&mut self) -> TypeId {
        self.types.named("char").unwrap_or_else(|| self.types.intern(Type::Primitive(PrimitiveType::Char)))
    }

    fn error_type(&mut self) -> TypeId {
        self.types.intern(Type::Error)
    }
}

impl CheckedProgram {
    pub fn types_count(&self) -> usize {
        self.types.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Block, Expr, FunctionItem, FunctionKind, Param, Program, Stmt, TypeExpr};

    #[test]
    fn checker_types_literals_and_function_bodies() {
        let program = Program::new(vec![Item::Function(FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: FunctionKind::Fn,
            name: "answer".to_string(),
            generics: Vec::new(),
            params: vec![Param {
                name: "value".to_string(),
                ty: Some(TypeExpr::Path(crate::frontend::ast::PathExpr::new(vec!["int".to_string()], Vec::new()))),
            }],
            return_type: Some(TypeExpr::Path(crate::frontend::ast::PathExpr::new(vec!["int".to_string()], Vec::new()))),
            body: Block::new(vec![Stmt::Let {
                name: "x".to_string(),
                mutable: false,
                value: Some(Expr::Integer(1)),
            }], Some(Expr::Identifier("x".to_string()))),
        })]);

        let checked = TypeChecker::new(ModulePath::root()).check_program(&program);
        assert!(checked.diagnostics.is_empty());
        assert_eq!(checked.items.len(), 1);

        match &checked.items[0] {
            CheckedItem::Function(function) => {
                assert_eq!(function.name, "answer");
                assert_eq!(function.body.ty, function.signature.return_type);
                assert_eq!(function.signature.parameters.len(), 1);
            }
            other => panic!("unexpected checked item: {other:?}"),
        }
    }

    #[test]
    fn checker_reports_unknown_names() {
        let program = Program::new(vec![Item::Stmt(Stmt::Expr(Expr::Identifier("missing".to_string())))]);
        let checked = TypeChecker::new(ModulePath::root()).check_program(&program);

        assert!(checked.diagnostics.has_errors());
        assert!(matches!(checked.items[0], CheckedItem::Stmt(_)));
    }
}