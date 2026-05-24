use std::collections::BTreeSet;

use crate::diagnostics::Diagnostic;
use crate::hir::{ResolvedProgram, SymbolId};
use crate::type_checker::{
    CheckedBlock, CheckedExpr, CheckedExprKind, CheckedItem, CheckedProgram, CheckedStmt, CheckedStmtKind,
};
use crate::types::TypeId;

pub fn initialize() {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub blocks: Vec<MirBasicBlock>,
    pub temp_count: usize,
    pub item_count: usize,
    pub symbol_count: usize,
    pub type_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredSsaProgram {
    pub blocks: Vec<SsaBasicBlock>,
    pub value_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SsaTypeMap {
    pub symbol_types: std::collections::BTreeMap<SymbolId, TypeId>,
    pub name_types: std::collections::BTreeMap<(MirPlace, usize), TypeId>,
}

impl SsaTypeMap {
    pub fn insert_name_type(&mut self, name: &SsaName, ty: TypeId) -> bool {
        self.name_types
            .insert((name.place.clone(), name.version), ty)
            .is_none_or(|previous| previous != ty)
    }

    pub fn type_of_name(&self, name: &SsaName) -> Option<TypeId> {
        self.name_types.get(&(name.place.clone(), name.version)).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaBasicBlock {
    pub id: usize,
    pub phis: Vec<SsaPhi>,
    pub statements: Vec<SsaStatement>,
    pub terminator: SsaTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaPhi {
    pub target: SsaName,
    pub incoming: Vec<SsaPhiIncoming>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaPhiIncoming {
    pub block: usize,
    pub value: SsaValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaStatement {
    pub id: usize,
    pub kind: SsaStatementKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaStatementKind {
    Assign { target: SsaName, value: SsaValue },
    Eval(SsaValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaName {
    pub place: MirPlace,
    pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaTerminator {
    Return(Option<SsaValue>),
    Goto(usize),
    Branch {
        condition: SsaValue,
        then_block: usize,
        else_block: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaValue {
    Unit,
    Integer(i64),
    Float(String),
    Boolean(bool),
    String(String),
    Char(char),
    Name(SsaName),
    OpaqueExpr,
    UnresolvedPlace(MirPlace),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirBasicBlock {
    pub id: usize,
    pub statements: Vec<MirStatement>,
    pub terminator: MirTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirStatement {
    pub id: usize,
    pub kind: MirStatementKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirStatementKind {
    AssignPlace { place: MirPlace, value: MirValue },
    JoinPlace { place: MirPlace, incoming_blocks: Vec<usize> },
    Eval(MirValue),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MirPlace {
    Local(Option<SymbolId>),
    Temp(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirTerminator {
    Return(Option<MirValue>),
    Goto(usize),
    Branch {
        condition: MirValue,
        then_block: usize,
        else_block: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirValue {
    Unit,
    Integer(i64),
    Float(String),
    Boolean(bool),
    String(String),
    Char(char),
    SymbolRef(Option<SymbolId>),
    Temp(usize),
    OpaqueExpr,
}

#[derive(Debug, Default)]
struct MirBuilder {
    blocks: Vec<MirBasicBlock>,
    next_block: usize,
    next_stmt: usize,
    next_temp: usize,
}

impl MirBuilder {
    fn alloc_block(&mut self, terminator: MirTerminator) -> usize {
        let id = self.next_block;
        self.next_block += 1;
        self.blocks.push(MirBasicBlock {
            id,
            statements: Vec::new(),
            terminator,
        });
        id
    }

    fn set_terminator(&mut self, block_id: usize, terminator: MirTerminator) {
        if let Some(block) = self.blocks.get_mut(block_id) {
            block.terminator = terminator;
        }
    }

    fn push_statement(&mut self, block_id: usize, kind: MirStatementKind) {
        let id = self.next_stmt;
        self.next_stmt += 1;
        if let Some(block) = self.blocks.get_mut(block_id) {
            block.statements.push(MirStatement { id, kind });
        }
    }

    fn alloc_temp(&mut self) -> usize {
        let temp = self.next_temp;
        self.next_temp += 1;
        temp
    }

    fn lower_checked_item(&mut self, item: &CheckedItem) {
        if let CheckedItem::Function(function) = item {
            self.lower_block_as_function(&function.body);
        }
    }

    fn lower_block_as_function(&mut self, block: &CheckedBlock) {
        let entry = self.alloc_block(MirTerminator::Return(None));
        let (active_block, tail_value) = self.lower_block_into(entry, block);
        if let Some(active_block) = active_block {
            self.set_terminator(active_block, MirTerminator::Return(tail_value));
        }
    }

    fn lower_block_into(&mut self, block_id: usize, block: &CheckedBlock) -> (Option<usize>, Option<MirValue>) {
        let mut active_block = Some(block_id);

        for stmt in &block.statements {
            let Some(current_block) = active_block else {
                return (None, None);
            };

            match &stmt.kind {
                CheckedStmtKind::Let {
                    value,
                    symbol_id,
                    ..
                } => {
                    if let Some(value) = value {
                        let (lowered, next_block) = self.lower_expr(current_block, value);
                        let Some(next_block) = next_block else {
                            return (None, None);
                        };
                        self.push_statement(next_block, MirStatementKind::AssignPlace {
                            place: MirPlace::Local(*symbol_id),
                            value: lowered,
                        });
                        active_block = Some(next_block);
                    }
                }
                CheckedStmtKind::Return(value) => {
                    if let Some(value) = value {
                        let (lowered, next_block) = self.lower_expr(current_block, value);
                        let Some(next_block) = next_block else {
                            return (None, None);
                        };
                        self.set_terminator(next_block, MirTerminator::Return(Some(lowered)));
                    } else {
                        self.set_terminator(current_block, MirTerminator::Return(None));
                    }
                    return (None, None);
                }
                CheckedStmtKind::Expr(expr) => {
                    let (value, next_block) = self.lower_expr(current_block, expr);
                    let Some(next_block) = next_block else {
                        return (None, None);
                    };
                    self.push_statement(next_block, MirStatementKind::Eval(value));
                    active_block = Some(next_block);
                }
            }
        }

        if let Some(tail) = block.tail.as_ref() {
            let Some(current_block) = active_block else {
                return (None, None);
            };
            let (tail_value, next_block) = self.lower_expr(current_block, tail);
            return (next_block, Some(tail_value));
        }

        (active_block, None)
    }

    fn lower_expr(&mut self, block_id: usize, expr: &CheckedExpr) -> (MirValue, Option<usize>) {
        match &expr.kind {
            CheckedExprKind::Integer(value) => (MirValue::Integer(*value), Some(block_id)),
            CheckedExprKind::Float(value) => (MirValue::Float(value.clone()), Some(block_id)),
            CheckedExprKind::Boolean(value) => (MirValue::Boolean(*value), Some(block_id)),
            CheckedExprKind::String(value) => (MirValue::String(value.clone()), Some(block_id)),
            CheckedExprKind::Char(value) => (MirValue::Char(*value), Some(block_id)),
            CheckedExprKind::Identifier(_) | CheckedExprKind::Path(_) => (MirValue::SymbolRef(expr.symbol_id), Some(block_id)),
            CheckedExprKind::Tuple(values) => {
                let mut current_block = block_id;
                for value in values {
                    let (lowered, next_block) = self.lower_expr(current_block, value);
                    let Some(next_block) = next_block else {
                        return (MirValue::OpaqueExpr, None);
                    };
                    self.push_statement(next_block, MirStatementKind::Eval(lowered));
                    current_block = next_block;
                }
                (MirValue::OpaqueExpr, Some(current_block))
            }
            CheckedExprKind::Block(block) => {
                let (active_block, tail_value) = self.lower_block_into(block_id, block);
                (tail_value.unwrap_or(MirValue::Unit), active_block)
            }
            CheckedExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let (condition, condition_block) = self.lower_expr(block_id, condition);
                let Some(condition_block) = condition_block else {
                    return (MirValue::Unit, None);
                };

                let then_block = self.alloc_block(MirTerminator::Return(None));
                let else_block = self.alloc_block(MirTerminator::Return(None));
                let join_block = self.alloc_block(MirTerminator::Return(None));
                let join_temp = self.alloc_temp();
                let mut incoming_blocks = Vec::new();

                self.set_terminator(
                    condition_block,
                    MirTerminator::Branch {
                        condition,
                        then_block,
                        else_block,
                    },
                );

                let (then_active, then_value) = self.lower_block_into(then_block, then_branch);
                let then_value = then_value.unwrap_or(MirValue::Unit);
                if let Some(then_active) = then_active {
                    self.push_statement(
                        then_active,
                        MirStatementKind::AssignPlace {
                            place: MirPlace::Temp(join_temp),
                            value: then_value,
                        },
                    );
                    self.set_terminator(then_active, MirTerminator::Goto(join_block));
                    incoming_blocks.push(then_active);
                }

                let (else_active, else_value) = else_branch
                    .as_ref()
                    .map(|else_block_expr| self.lower_block_into(else_block, else_block_expr))
                    .unwrap_or((Some(else_block), None));
                let else_value = else_value.unwrap_or(MirValue::Unit);
                if let Some(else_active) = else_active {
                    self.push_statement(
                        else_active,
                        MirStatementKind::AssignPlace {
                            place: MirPlace::Temp(join_temp),
                            value: else_value,
                        },
                    );
                    self.set_terminator(else_active, MirTerminator::Goto(join_block));
                    incoming_blocks.push(else_active);
                }

                self.push_statement(
                    join_block,
                    MirStatementKind::JoinPlace {
                        place: MirPlace::Temp(join_temp),
                        incoming_blocks,
                    },
                );

                (MirValue::Temp(join_temp), Some(join_block))
            }
            CheckedExprKind::Call { callee, args } => {
                let mut current_block = block_id;
                let (callee_value, next_block) = self.lower_expr(current_block, callee);
                let Some(next_block) = next_block else {
                    return (MirValue::OpaqueExpr, None);
                };
                self.push_statement(next_block, MirStatementKind::Eval(callee_value));
                current_block = next_block;

                for arg in args {
                    let (arg_value, next_block) = self.lower_expr(current_block, arg);
                    let Some(next_block) = next_block else {
                        return (MirValue::OpaqueExpr, None);
                    };
                    self.push_statement(next_block, MirStatementKind::Eval(arg_value));
                    current_block = next_block;
                }

                let temp = self.alloc_temp();
                self.push_statement(
                    current_block,
                    MirStatementKind::AssignPlace {
                        place: MirPlace::Temp(temp),
                        value: MirValue::OpaqueExpr,
                    },
                );
                (MirValue::Temp(temp), Some(current_block))
            }
            CheckedExprKind::Unary { expr, .. } => self.lower_expr(block_id, expr),
            CheckedExprKind::Binary { left, right, .. }
            | CheckedExprKind::Range {
                start: left,
                end: right,
            } => {
                let (left, next_block) = self.lower_expr(block_id, left);
                let Some(next_block) = next_block else {
                    return (MirValue::OpaqueExpr, None);
                };
                self.push_statement(next_block, MirStatementKind::Eval(left));

                let (right, next_block) = self.lower_expr(next_block, right);
                let Some(next_block) = next_block else {
                    return (MirValue::OpaqueExpr, None);
                };
                self.push_statement(next_block, MirStatementKind::Eval(right));

                (MirValue::OpaqueExpr, Some(next_block))
            }
            CheckedExprKind::While { condition, body } => {
                let loop_header_block = self.alloc_block(MirTerminator::Return(None));
                let loop_body_block = self.alloc_block(MirTerminator::Return(None));
                let loop_latch_block = self.alloc_block(MirTerminator::Return(None));
                let loop_exit_block = self.alloc_block(MirTerminator::Return(None));

                self.set_terminator(block_id, MirTerminator::Goto(loop_header_block));

                let (condition_value, condition_eval_block) = self.lower_expr(loop_header_block, condition);
                let Some(condition_eval_block) = condition_eval_block else {
                    return (MirValue::Unit, None);
                };

                self.set_terminator(
                    condition_eval_block,
                    MirTerminator::Branch {
                        condition: condition_value,
                        then_block: loop_body_block,
                        else_block: loop_exit_block,
                    },
                );

                let (body_active, _) = self.lower_block_into(loop_body_block, body);
                if let Some(body_active) = body_active {
                    self.set_terminator(body_active, MirTerminator::Goto(loop_latch_block));
                }

                self.set_terminator(loop_latch_block, MirTerminator::Goto(loop_header_block));

                let loop_result_temp = self.alloc_temp();
                self.push_statement(
                    loop_exit_block,
                    MirStatementKind::JoinPlace {
                        place: MirPlace::Temp(loop_result_temp),
                        incoming_blocks: vec![condition_eval_block],
                    },
                );

                (MirValue::Temp(loop_result_temp), Some(loop_exit_block))
            }
            CheckedExprKind::For { iterable, body, .. } => {
                let (iterable_value, iterable_block) = self.lower_expr(block_id, iterable);
                let Some(iterable_block) = iterable_block else {
                    return (MirValue::Unit, None);
                };
                self.push_statement(iterable_block, MirStatementKind::Eval(iterable_value));

                let loop_header_block = self.alloc_block(MirTerminator::Return(None));
                let loop_body_block = self.alloc_block(MirTerminator::Return(None));
                let loop_latch_block = self.alloc_block(MirTerminator::Return(None));
                let loop_exit_block = self.alloc_block(MirTerminator::Return(None));

                self.set_terminator(iterable_block, MirTerminator::Goto(loop_header_block));

                self.set_terminator(
                    loop_header_block,
                    MirTerminator::Branch {
                        condition: MirValue::OpaqueExpr,
                        then_block: loop_body_block,
                        else_block: loop_exit_block,
                    },
                );

                let (body_active, _) = self.lower_block_into(loop_body_block, body);
                if let Some(body_active) = body_active {
                    self.set_terminator(body_active, MirTerminator::Goto(loop_latch_block));
                }

                self.set_terminator(loop_latch_block, MirTerminator::Goto(loop_header_block));

                let loop_result_temp = self.alloc_temp();
                self.push_statement(
                    loop_exit_block,
                    MirStatementKind::JoinPlace {
                        place: MirPlace::Temp(loop_result_temp),
                        incoming_blocks: vec![loop_header_block],
                    },
                );

                (MirValue::Temp(loop_result_temp), Some(loop_exit_block))
            }
            CheckedExprKind::Match { value, arms } => {
                let (match_value, match_block) = self.lower_expr(block_id, value);
                let Some(match_block) = match_block else {
                    return (MirValue::Unit, None);
                };
                self.push_statement(match_block, MirStatementKind::Eval(match_value));

                if arms.is_empty() {
                    return (MirValue::Unit, Some(match_block));
                }

                let join_block = self.alloc_block(MirTerminator::Return(Some(MirValue::Unit)));
                let join_temp = self.alloc_temp();
                let mut incoming_blocks = Vec::new();
                self.push_statement(
                    match_block,
                    MirStatementKind::AssignPlace {
                        place: MirPlace::Temp(join_temp),
                        value: MirValue::Unit,
                    },
                );
                let mut dispatch_block = match_block;
                let mut default_incoming_block = match_block;

                for (index, arm) in arms.iter().enumerate() {
                    let arm_block = self.alloc_block(MirTerminator::Return(None));
                    let fallback_block = if index == arms.len() - 1 {
                        default_incoming_block = dispatch_block;
                        join_block
                    } else {
                        self.alloc_block(MirTerminator::Return(None))
                    };

                    self.set_terminator(
                        dispatch_block,
                        MirTerminator::Branch {
                            condition: MirValue::OpaqueExpr,
                            then_block: arm_block,
                            else_block: fallback_block,
                        },
                    );

                    let (arm_value, arm_active) = self.lower_expr(arm_block, &arm.value);
                    if let Some(arm_active) = arm_active {
                        self.push_statement(
                            arm_active,
                            MirStatementKind::AssignPlace {
                                place: MirPlace::Temp(join_temp),
                                value: arm_value,
                            },
                        );
                        self.set_terminator(arm_active, MirTerminator::Goto(join_block));
                        incoming_blocks.push(arm_active);
                    }

                    dispatch_block = fallback_block;
                }

                incoming_blocks.push(default_incoming_block);
                self.push_statement(
                    join_block,
                    MirStatementKind::JoinPlace {
                        place: MirPlace::Temp(join_temp),
                        incoming_blocks,
                    },
                );

                (MirValue::Temp(join_temp), Some(join_block))
            }
            CheckedExprKind::Arena { body, .. } => {
                let (active_block, tail_value) = self.lower_block_into(block_id, body);
                (tail_value.unwrap_or(MirValue::Unit), active_block)
            }
            CheckedExprKind::Field { target, .. } => {
                let (target, next_block) = self.lower_expr(block_id, target);
                let Some(next_block) = next_block else {
                    return (MirValue::OpaqueExpr, None);
                };
                self.push_statement(next_block, MirStatementKind::Eval(target));
                (MirValue::OpaqueExpr, Some(next_block))
            }
            CheckedExprKind::Index { target, index } => {
                let (target, next_block) = self.lower_expr(block_id, target);
                let Some(next_block) = next_block else {
                    return (MirValue::OpaqueExpr, None);
                };
                self.push_statement(next_block, MirStatementKind::Eval(target));

                let (index, next_block) = self.lower_expr(next_block, index);
                let Some(next_block) = next_block else {
                    return (MirValue::OpaqueExpr, None);
                };
                self.push_statement(next_block, MirStatementKind::Eval(index));

                (MirValue::OpaqueExpr, Some(next_block))
            }
            CheckedExprKind::Try(expr) => self.lower_expr(block_id, expr),
        }
    }
}

pub fn lower_from_tir(tir: &CheckedProgram, hir: &ResolvedProgram) -> LoweredProgram {
    let mut builder = MirBuilder::default();
    for item in &tir.items {
        builder.lower_checked_item(item);
    }

    LoweredProgram {
        blocks: builder.blocks,
        temp_count: builder.next_temp,
        item_count: tir.items.len(),
        symbol_count: hir.symbols.len(),
        type_count: tir.types_count(),
    }
}

pub fn build_ssa_scaffold(mir: &LoweredProgram) -> LoweredSsaProgram {
    let analysis = analyze_cfg(mir);
    let out_versions = compute_block_out_versions(mir, &analysis);
    let mut blocks = Vec::with_capacity(mir.blocks.len());
    let mut value_count = 0;

    for block_id in &analysis.reverse_post_order {
        let block = &mir.blocks[*block_id];
        let mut versions = incoming_versions_for_block(*block_id, &analysis, &out_versions);
        let mut phis = Vec::new();
        let mut statements = Vec::new();

        for statement in &block.statements {
            match &statement.kind {
                MirStatementKind::AssignPlace { place, value } => {
                    let version = next_ssa_version(&mut versions, place.clone());
                    value_count += 1;
                    statements.push(SsaStatement {
                        id: statement.id,
                        kind: SsaStatementKind::Assign {
                            target: SsaName {
                                place: place.clone(),
                                version,
                            },
                            value: map_mir_value_to_ssa(value, &versions),
                        },
                    });
                }
                MirStatementKind::JoinPlace {
                    place,
                    incoming_blocks,
                } => {
                    let version = next_ssa_version(&mut versions, place.clone());
                    value_count += 1;
                    let incoming = incoming_blocks
                        .iter()
                        .map(|incoming_block| SsaPhiIncoming {
                            block: *incoming_block,
                            value: out_versions
                                .get(*incoming_block)
                                .and_then(|versions| versions.get(place).copied())
                                .map(|incoming_version| {
                                    SsaValue::Name(SsaName {
                                        place: place.clone(),
                                        version: incoming_version,
                                    })
                                })
                                .unwrap_or_else(|| SsaValue::UnresolvedPlace(place.clone())),
                        })
                        .collect();

                    phis.push(SsaPhi {
                        target: SsaName {
                            place: place.clone(),
                            version,
                        },
                        incoming,
                    });
                }
                MirStatementKind::Eval(value) => statements.push(SsaStatement {
                    id: statement.id,
                    kind: SsaStatementKind::Eval(map_mir_value_to_ssa(value, &versions)),
                }),
            }
        }

        blocks.push(SsaBasicBlock {
            id: block.id,
            phis,
            statements,
            terminator: map_mir_terminator_to_ssa(&block.terminator, &versions),
        });
    }

    blocks.sort_by_key(|block| block.id);

    LoweredSsaProgram { blocks, value_count }
}

pub fn build_ssa_scaffold_with_types(mir: &LoweredProgram, tir: &CheckedProgram) -> (LoweredSsaProgram, SsaTypeMap) {
    let ssa = build_ssa_scaffold(mir);
    let mut type_map = SsaTypeMap {
        symbol_types: collect_symbol_types(tir),
        name_types: std::collections::BTreeMap::new(),
    };

    seed_symbol_backed_name_types(&ssa, &mut type_map);
    propagate_ssa_name_types(&ssa, &mut type_map, tir);

    (ssa, type_map)
}

fn collect_symbol_types(tir: &CheckedProgram) -> std::collections::BTreeMap<SymbolId, TypeId> {
    let mut symbol_types = std::collections::BTreeMap::new();
    for item in &tir.items {
        collect_symbol_types_from_item(item, &mut symbol_types);
    }
    symbol_types
}

fn collect_symbol_types_from_item(
    item: &CheckedItem,
    symbol_types: &mut std::collections::BTreeMap<SymbolId, TypeId>,
) {
    match item {
        CheckedItem::Function(function) => {
            for (symbol_id, parameter) in function.param_symbol_ids.iter().zip(function.signature.parameters.iter()) {
                if let Some(symbol_id) = symbol_id {
                    symbol_types.entry(*symbol_id).or_insert(parameter.ty);
                }
            }
            collect_symbol_types_from_block(&function.body, symbol_types);
        }
        CheckedItem::Arena(arena) => collect_symbol_types_from_block(&arena.body, symbol_types),
        CheckedItem::Stmt(statement) => collect_symbol_types_from_stmt(statement, symbol_types),
        CheckedItem::Module(module_item) => {
            for nested in &module_item.items {
                collect_symbol_types_from_item(nested, symbol_types);
            }
        }
        CheckedItem::Impl(impl_item) => {
            for nested in &impl_item.items {
                collect_symbol_types_from_item(nested, symbol_types);
            }
        }
        CheckedItem::Struct(_)
        | CheckedItem::Enum(_)
        | CheckedItem::TypeAlias(_)
        | CheckedItem::Use(_) => {}
    }
}

fn collect_symbol_types_from_block(
    block: &CheckedBlock,
    symbol_types: &mut std::collections::BTreeMap<SymbolId, TypeId>,
) {
    for statement in &block.statements {
        collect_symbol_types_from_stmt(statement, symbol_types);
    }

    if let Some(tail) = block.tail.as_ref() {
        collect_symbol_types_from_expr(tail, symbol_types);
    }
}

fn collect_symbol_types_from_stmt(
    statement: &CheckedStmt,
    symbol_types: &mut std::collections::BTreeMap<SymbolId, TypeId>,
) {
    match &statement.kind {
        CheckedStmtKind::Let {
            symbol_id,
            value,
            ..
        } => {
            if let (Some(symbol_id), Some(value)) = (symbol_id, value) {
                symbol_types.entry(*symbol_id).or_insert(value.ty);
                collect_symbol_types_from_expr(value, symbol_types);
            }
        }
        CheckedStmtKind::Return(value) => {
            if let Some(value) = value {
                collect_symbol_types_from_expr(value, symbol_types);
            }
        }
        CheckedStmtKind::Expr(expr) => collect_symbol_types_from_expr(expr, symbol_types),
    }
}

fn collect_symbol_types_from_expr(
    expr: &CheckedExpr,
    symbol_types: &mut std::collections::BTreeMap<SymbolId, TypeId>,
) {
    if let Some(symbol_id) = expr.symbol_id {
        symbol_types.entry(symbol_id).or_insert(expr.ty);
    }

    match &expr.kind {
        CheckedExprKind::Block(block) => collect_symbol_types_from_block(block, symbol_types),
        CheckedExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_symbol_types_from_expr(condition, symbol_types);
            collect_symbol_types_from_block(then_branch, symbol_types);
            if let Some(else_branch) = else_branch {
                collect_symbol_types_from_block(else_branch, symbol_types);
            }
        }
        CheckedExprKind::While { condition, body } => {
            collect_symbol_types_from_expr(condition, symbol_types);
            collect_symbol_types_from_block(body, symbol_types);
        }
        CheckedExprKind::For {
            binding_symbol_id,
            iterable,
            body,
            ..
        } => {
            if let Some(symbol_id) = binding_symbol_id {
                symbol_types.entry(*symbol_id).or_insert(expr.ty);
            }
            collect_symbol_types_from_expr(iterable, symbol_types);
            collect_symbol_types_from_block(body, symbol_types);
        }
        CheckedExprKind::Match { value, arms } => {
            collect_symbol_types_from_expr(value, symbol_types);
            for arm in arms {
                collect_symbol_types_from_expr(&arm.value, symbol_types);
            }
        }
        CheckedExprKind::Arena { body, .. } => collect_symbol_types_from_block(body, symbol_types),
        CheckedExprKind::Call { callee, args } => {
            collect_symbol_types_from_expr(callee, symbol_types);
            for arg in args {
                collect_symbol_types_from_expr(arg, symbol_types);
            }
        }
        CheckedExprKind::Field { target, .. }
        | CheckedExprKind::Try(target)
        | CheckedExprKind::Unary { expr: target, .. } => collect_symbol_types_from_expr(target, symbol_types),
        CheckedExprKind::Index { target, index } => {
            collect_symbol_types_from_expr(target, symbol_types);
            collect_symbol_types_from_expr(index, symbol_types);
        }
        CheckedExprKind::Binary { left, right, .. }
        | CheckedExprKind::Range {
            start: left,
            end: right,
        } => {
            collect_symbol_types_from_expr(left, symbol_types);
            collect_symbol_types_from_expr(right, symbol_types);
        }
        CheckedExprKind::Tuple(values) => {
            for value in values {
                collect_symbol_types_from_expr(value, symbol_types);
            }
        }
        CheckedExprKind::Identifier(_)
        | CheckedExprKind::Integer(_)
        | CheckedExprKind::Float(_)
        | CheckedExprKind::Boolean(_)
        | CheckedExprKind::String(_)
        | CheckedExprKind::Char(_)
        | CheckedExprKind::Path(_) => {}
    }
}

fn seed_symbol_backed_name_types(ssa: &LoweredSsaProgram, type_map: &mut SsaTypeMap) {
    for block in &ssa.blocks {
        for phi in &block.phis {
            seed_symbol_backed_name(&phi.target, type_map);
            for incoming in &phi.incoming {
                if let SsaValue::Name(name) = &incoming.value {
                    seed_symbol_backed_name(name, type_map);
                }
            }
        }

        for statement in &block.statements {
            match &statement.kind {
                SsaStatementKind::Assign { target, value } => {
                    seed_symbol_backed_name(target, type_map);
                    if let SsaValue::Name(name) = value {
                        seed_symbol_backed_name(name, type_map);
                    }
                }
                SsaStatementKind::Eval(value) => {
                    if let SsaValue::Name(name) = value {
                        seed_symbol_backed_name(name, type_map);
                    }
                }
            }
        }

        match &block.terminator {
            SsaTerminator::Return(Some(SsaValue::Name(name))) => seed_symbol_backed_name(name, type_map),
            SsaTerminator::Branch { condition: SsaValue::Name(name), .. } => seed_symbol_backed_name(name, type_map),
            _ => {}
        }
    }
}

fn seed_symbol_backed_name(name: &SsaName, type_map: &mut SsaTypeMap) {
    if let MirPlace::Local(Some(symbol_id)) = &name.place {
        if let Some(ty) = type_map.symbol_types.get(symbol_id).copied() {
            let _ = type_map.insert_name_type(name, ty);
        }
    }
}

fn propagate_ssa_name_types(ssa: &LoweredSsaProgram, type_map: &mut SsaTypeMap, tir: &CheckedProgram) {
    let bool_ty = tir.types.named("bool");
    let int_ty = tir.types.named("int");
    let float_ty = tir.types.named("float");
    let string_ty = tir.types.named("string");
    let char_ty = tir.types.named("char");
    let unit_ty = tir.types.named("unit");

    let mut changed = true;
    while changed {
        changed = false;

        for block in &ssa.blocks {
            for phi in &block.phis {
                let inferred = phi
                    .incoming
                    .iter()
                    .find_map(|incoming| infer_ssa_value_type(&incoming.value, type_map, bool_ty, int_ty, float_ty, string_ty, char_ty, unit_ty));

                if let Some(ty) = inferred {
                    changed |= type_map.insert_name_type(&phi.target, ty);
                }
            }

            for statement in &block.statements {
                if let SsaStatementKind::Assign { target, value } = &statement.kind {
                    if let Some(ty) = infer_ssa_value_type(value, type_map, bool_ty, int_ty, float_ty, string_ty, char_ty, unit_ty)
                    {
                        changed |= type_map.insert_name_type(target, ty);
                    }
                }
            }
        }
    }
}

fn infer_ssa_value_type(
    value: &SsaValue,
    type_map: &SsaTypeMap,
    bool_ty: Option<TypeId>,
    int_ty: Option<TypeId>,
    float_ty: Option<TypeId>,
    string_ty: Option<TypeId>,
    char_ty: Option<TypeId>,
    unit_ty: Option<TypeId>,
) -> Option<TypeId> {
    match value {
        SsaValue::Boolean(_) => bool_ty,
        SsaValue::Integer(_) => int_ty,
        SsaValue::Float(_) => float_ty,
        SsaValue::String(_) => string_ty,
        SsaValue::Char(_) => char_ty,
        SsaValue::Unit => unit_ty,
        SsaValue::Name(name) => type_map.type_of_name(name),
        SsaValue::OpaqueExpr | SsaValue::UnresolvedPlace(_) => None,
    }
}

fn compute_block_out_versions(
    mir: &LoweredProgram,
    analysis: &MirCfgAnalysis,
) -> Vec<std::collections::BTreeMap<MirPlace, usize>> {
    let mut out_versions: Vec<std::collections::BTreeMap<MirPlace, usize>> = vec![std::collections::BTreeMap::new(); mir.blocks.len()];

    if analysis.reverse_post_order.is_empty() {
        return out_versions;
    }

    let mut changed = true;
    while changed {
        changed = false;

        for block_id in &analysis.reverse_post_order {
            let block = &mir.blocks[*block_id];
            let mut versions = incoming_versions_for_block(*block_id, analysis, &out_versions);

            for statement in &block.statements {
                match &statement.kind {
                    MirStatementKind::AssignPlace { place, .. }
                    | MirStatementKind::JoinPlace { place, .. } => {
                        let _ = next_ssa_version(&mut versions, place.clone());
                    }
                    MirStatementKind::Eval(_) => {}
                }
            }

            if versions != out_versions[*block_id] {
                out_versions[*block_id] = versions;
                changed = true;
            }
        }
    }

    out_versions
}

fn map_mir_terminator_to_ssa(
    terminator: &MirTerminator,
    versions: &std::collections::BTreeMap<MirPlace, usize>,
) -> SsaTerminator {
    match terminator {
        MirTerminator::Return(value) => SsaTerminator::Return(value.as_ref().map(|value| map_mir_value_to_ssa(value, versions))),
        MirTerminator::Goto(target) => SsaTerminator::Goto(*target),
        MirTerminator::Branch {
            condition,
            then_block,
            else_block,
        } => SsaTerminator::Branch {
            condition: map_mir_value_to_ssa(condition, versions),
            then_block: *then_block,
            else_block: *else_block,
        },
    }
}

fn map_mir_value_to_ssa(
    value: &MirValue,
    versions: &std::collections::BTreeMap<MirPlace, usize>,
) -> SsaValue {
    match value {
        MirValue::Unit => SsaValue::Unit,
        MirValue::Integer(value) => SsaValue::Integer(*value),
        MirValue::Float(value) => SsaValue::Float(value.clone()),
        MirValue::Boolean(value) => SsaValue::Boolean(*value),
        MirValue::String(value) => SsaValue::String(value.clone()),
        MirValue::Char(value) => SsaValue::Char(*value),
        MirValue::OpaqueExpr => SsaValue::OpaqueExpr,
        MirValue::Temp(temp) => ssa_name_or_unresolved(MirPlace::Temp(*temp), versions),
        MirValue::SymbolRef(symbol_id) => ssa_name_or_unresolved(MirPlace::Local(*symbol_id), versions),
    }
}

fn ssa_name_or_unresolved(
    place: MirPlace,
    versions: &std::collections::BTreeMap<MirPlace, usize>,
) -> SsaValue {
    versions
        .get(&place)
        .copied()
        .map(|version| {
            SsaValue::Name(SsaName {
                place: place.clone(),
                version,
            })
        })
        .unwrap_or(SsaValue::UnresolvedPlace(place))
}

fn incoming_versions_for_block(
    block_id: usize,
    analysis: &MirCfgAnalysis,
    out_versions: &[std::collections::BTreeMap<MirPlace, usize>],
) -> std::collections::BTreeMap<MirPlace, usize> {
    let Some(first_predecessor) = analysis.predecessors[block_id].first().copied() else {
        return std::collections::BTreeMap::new();
    };

    let mut merged = out_versions[first_predecessor].clone();
    for predecessor in analysis.predecessors[block_id].iter().skip(1) {
        merged.retain(|place, version| out_versions[*predecessor].get(place).is_some_and(|other| other == version));
    }

    merged
}

fn next_ssa_version(versions: &mut std::collections::BTreeMap<MirPlace, usize>, place: MirPlace) -> usize {
    let entry = versions.entry(place).or_insert(0);
    *entry += 1;
    *entry
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirCfgAnalysis {
    pub successors: Vec<Vec<usize>>,
    pub predecessors: Vec<Vec<usize>>,
    pub reverse_post_order: Vec<usize>,
    pub dominators: Vec<BTreeSet<usize>>,
}

pub fn mir_successors(terminator: &MirTerminator) -> Vec<usize> {
    match terminator {
        MirTerminator::Return(_) => Vec::new(),
        MirTerminator::Goto(target) => vec![*target],
        MirTerminator::Branch {
            then_block,
            else_block,
            ..
        } => vec![*then_block, *else_block],
    }
}

pub fn analyze_cfg(mir: &LoweredProgram) -> MirCfgAnalysis {
    let block_count = mir.blocks.len();
    let mut successors = vec![Vec::new(); block_count];
    let mut predecessors = vec![Vec::new(); block_count];

    for (index, block) in mir.blocks.iter().enumerate() {
        for successor in mir_successors(&block.terminator)
            .into_iter()
            .filter(|target| *target < block_count)
        {
            successors[index].push(successor);
            predecessors[successor].push(index);
        }
    }

    let reverse_post_order = compute_reverse_post_order(&successors, 0);
    let dominators = compute_dominators(&predecessors, 0);

    MirCfgAnalysis {
        successors,
        predecessors,
        reverse_post_order,
        dominators,
    }
}

fn compute_reverse_post_order(successors: &[Vec<usize>], entry: usize) -> Vec<usize> {
    if successors.is_empty() || entry >= successors.len() {
        return Vec::new();
    }

    fn dfs(block: usize, successors: &[Vec<usize>], visited: &mut [bool], post_order: &mut Vec<usize>) {
        if visited[block] {
            return;
        }
        visited[block] = true;
        for successor in &successors[block] {
            dfs(*successor, successors, visited, post_order);
        }
        post_order.push(block);
    }

    let mut visited = vec![false; successors.len()];
    let mut post_order = Vec::new();
    dfs(entry, successors, &mut visited, &mut post_order);
    post_order.reverse();
    post_order
}

fn compute_dominators(predecessors: &[Vec<usize>], entry: usize) -> Vec<BTreeSet<usize>> {
    let block_count = predecessors.len();
    if block_count == 0 {
        return Vec::new();
    }

    let all_blocks: BTreeSet<usize> = (0..block_count).collect();
    let mut dominators = vec![BTreeSet::new(); block_count];

    for block in 0..block_count {
        if block == entry {
            dominators[block].insert(block);
        } else {
            dominators[block] = all_blocks.clone();
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        for block in 0..block_count {
            if block == entry {
                continue;
            }

            let new_set = if predecessors[block].is_empty() {
                let mut singleton = BTreeSet::new();
                singleton.insert(block);
                singleton
            } else {
                let mut intersection = all_blocks.clone();
                for predecessor in &predecessors[block] {
                    intersection = intersection
                        .intersection(&dominators[*predecessor])
                        .copied()
                        .collect();
                }
                intersection.insert(block);
                intersection
            };

            if new_set != dominators[block] {
                dominators[block] = new_set;
                changed = true;
            }
        }
    }

    dominators
}

pub fn verify_lowered_program(mir: &LoweredProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let block_count = mir.blocks.len();
    let analysis = analyze_cfg(mir);

    for (index, block) in mir.blocks.iter().enumerate() {
        if block.id != index {
            diagnostics.push(Diagnostic::error(
                "AXIS-MIR-001",
                format!(
                    "block ID mismatch: expected block at index {index} to have id {index}, found {}",
                    block.id
                ),
            ));
        }

        match block.terminator {
            MirTerminator::Return(_) => {}
            MirTerminator::Goto(target) => {
                if target >= block_count {
                    diagnostics.push(Diagnostic::error(
                        "AXIS-MIR-002",
                        format!("block {} has goto to out-of-range block {target}", block.id),
                    ));
                }
            }
            MirTerminator::Branch {
                then_block,
                else_block,
                ..
            } => {
                for target in [then_block, else_block] {
                    if target >= block_count {
                        diagnostics.push(Diagnostic::error(
                            "AXIS-MIR-003",
                            format!("block {} has branch to out-of-range block {target}", block.id),
                        ));
                    }
                }
            }
        }
    }

    for (index, block) in mir.blocks.iter().enumerate() {
        for statement in &block.statements {
            if let MirStatementKind::JoinPlace {
                incoming_blocks, ..
            } = &statement.kind
            {
                if incoming_blocks.is_empty() {
                    diagnostics.push(Diagnostic::error(
                        "AXIS-MIR-004",
                        format!("block {} has JoinPlace with no incoming blocks", block.id),
                    ));
                    continue;
                }

                let mut seen = BTreeSet::new();
                for incoming in incoming_blocks {
                    if *incoming >= block_count {
                        diagnostics.push(Diagnostic::error(
                            "AXIS-MIR-005",
                            format!(
                                "block {} JoinPlace references out-of-range incoming block {}",
                                block.id, incoming
                            ),
                        ));
                        continue;
                    }

                    if !seen.insert(*incoming) {
                        diagnostics.push(Diagnostic::error(
                            "AXIS-MIR-006",
                            format!(
                                "block {} JoinPlace lists duplicate incoming block {}",
                                block.id, incoming
                            ),
                        ));
                    }

                    if !analysis.predecessors[index].contains(incoming) {
                        diagnostics.push(Diagnostic::error(
                            "AXIS-MIR-007",
                            format!(
                                "block {} JoinPlace incoming block {} is not a predecessor",
                                block.id, incoming
                            ),
                        ));
                    }
                }
            }
        }
    }

    diagnostics
}

pub fn verify_ssa_scaffold(ssa: &LoweredSsaProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for block in &ssa.blocks {
        for phi in &block.phis {
            for incoming in &phi.incoming {
                if matches!(incoming.value, SsaValue::UnresolvedPlace(_)) {
                    diagnostics.push(Diagnostic::warning(
                        "AXIS-SSA-001",
                        format!(
                            "block {} phi for {:?} has unresolved incoming from block {}",
                            block.id, phi.target.place, incoming.block
                        ),
                    ));
                }
            }
        }

        for statement in &block.statements {
            let value = match &statement.kind {
                SsaStatementKind::Assign { value, .. } | SsaStatementKind::Eval(value) => value,
            };

            if matches!(value, SsaValue::UnresolvedPlace(_)) {
                diagnostics.push(Diagnostic::error(
                    "AXIS-SSA-002",
                    format!("block {} statement {} contains unresolved SSA value", block.id, statement.id),
                ));
            }
        }

        match &block.terminator {
            SsaTerminator::Return(value) => {
                if value.as_ref().is_some_and(|value| matches!(value, SsaValue::UnresolvedPlace(_))) {
                    diagnostics.push(Diagnostic::error(
                        "AXIS-SSA-003",
                        format!("block {} return terminator contains unresolved SSA value", block.id),
                    ));
                }
            }
            SsaTerminator::Branch { condition, .. } => {
                if matches!(condition, SsaValue::UnresolvedPlace(_)) {
                    diagnostics.push(Diagnostic::error(
                        "AXIS-SSA-004",
                        format!("block {} branch condition contains unresolved SSA value", block.id),
                    ));
                }
            }
            SsaTerminator::Goto(_) => {}
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, Item, Program, Stmt};
    use crate::hir::build_hir;
    use crate::resolution::ModulePath;
    use crate::type_checker::TypeChecker;

    #[test]
    fn lower_from_tir_tracks_counts() {
        let program = Program::new(vec![Item::Stmt(Stmt::Expr(Expr::Integer(1)))]);
        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        assert_eq!(mir.item_count, 1);
        assert_eq!(mir.symbol_count, hir.symbols.len());
        assert!(mir.type_count >= 1);
        assert!(mir.blocks.is_empty());
    }

    #[test]
    fn lower_from_tir_emits_blocks_for_function_items() {
        let program = Program::new(vec![Item::Function(crate::frontend::ast::FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: crate::frontend::ast::FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Some(crate::frontend::ast::TypeExpr::Path(crate::frontend::ast::PathExpr::new(
                vec!["int".to_string()],
                Vec::new(),
            ))),
            body: crate::frontend::ast::Block::new(
                vec![Stmt::Let {
                    name: "x".to_string(),
                    mutable: false,
                    value: Some(Expr::Integer(1)),
                }],
                Some(Expr::Identifier("x".to_string())),
            ),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        assert!(!mir.blocks.is_empty());
        assert!(mir.blocks.iter().any(|block| !block.statements.is_empty()));
        assert!(mir.blocks.iter().any(|block| {
            block
                .statements
                .iter()
                .any(|stmt| matches!(stmt.kind, MirStatementKind::AssignPlace { place: MirPlace::Local(Some(_)), .. }))
        }));
    }

    #[test]
    fn lower_from_tir_emits_control_flow_blocks_for_loops_and_match() {
        let program = Program::new(vec![Item::Function(crate::frontend::ast::FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: crate::frontend::ast::FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Some(crate::frontend::ast::TypeExpr::Path(crate::frontend::ast::PathExpr::new(
                vec!["int".to_string()],
                Vec::new(),
            ))),
            body: crate::frontend::ast::Block::new(
                vec![Stmt::Expr(Expr::While {
                    condition: Box::new(Expr::Boolean(true)),
                    body: crate::frontend::ast::Block::new(vec![Stmt::Expr(Expr::Integer(1))], None),
                })],
                Some(Expr::Match {
                    value: Box::new(Expr::Integer(1)),
                    arms: vec![
                        crate::frontend::ast::MatchArm::new(crate::frontend::ast::Pattern::Integer(1), Expr::Integer(2)),
                        crate::frontend::ast::MatchArm::new(crate::frontend::ast::Pattern::Wildcard, Expr::Integer(3)),
                    ],
                }),
            ),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        assert!(mir.blocks.len() >= 4);
        assert!(mir.blocks.iter().any(|block| matches!(block.terminator, MirTerminator::Branch { .. })));
    }

    #[test]
    fn lower_from_tir_routes_post_if_statements_to_continuation_block() {
        let program = Program::new(vec![Item::Function(crate::frontend::ast::FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: crate::frontend::ast::FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Some(crate::frontend::ast::TypeExpr::Path(crate::frontend::ast::PathExpr::new(
                vec!["int".to_string()],
                Vec::new(),
            ))),
            body: crate::frontend::ast::Block::new(
                vec![
                    Stmt::Expr(Expr::If {
                        condition: Box::new(Expr::Boolean(true)),
                        then_branch: crate::frontend::ast::Block::new(vec![], Some(Expr::Integer(1))),
                        else_branch: Some(crate::frontend::ast::Block::new(vec![], Some(Expr::Integer(2)))),
                    }),
                    Stmt::Let {
                        name: "x".to_string(),
                        mutable: false,
                        value: Some(Expr::Integer(3)),
                    },
                ],
                Some(Expr::Identifier("x".to_string())),
            ),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        assert!(mir.blocks.iter().any(|block| matches!(block.terminator, MirTerminator::Goto(_))));
        assert!(mir.blocks.iter().any(|block| {
            block
                .statements
                .iter()
                .any(|stmt| matches!(stmt.kind, MirStatementKind::AssignPlace { place: MirPlace::Local(Some(_)), .. }))
        }));
        assert!(mir.blocks.iter().all(|block| {
            if !matches!(block.terminator, MirTerminator::Branch { .. }) {
                return true;
            }
            !block
                .statements
                .iter()
                .any(|stmt| matches!(stmt.kind, MirStatementKind::AssignPlace { place: MirPlace::Local(_), .. }))
        }));
    }

    #[test]
    fn lower_from_tir_materializes_if_and_match_values_into_temps() {
        let program = Program::new(vec![Item::Function(crate::frontend::ast::FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: crate::frontend::ast::FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Some(crate::frontend::ast::TypeExpr::Path(crate::frontend::ast::PathExpr::new(
                vec!["int".to_string()],
                Vec::new(),
            ))),
            body: crate::frontend::ast::Block::new(
                vec![
                    Stmt::Let {
                        name: "a".to_string(),
                        mutable: false,
                        value: Some(Expr::If {
                            condition: Box::new(Expr::Boolean(true)),
                            then_branch: crate::frontend::ast::Block::new(vec![], Some(Expr::Integer(1))),
                            else_branch: Some(crate::frontend::ast::Block::new(vec![], Some(Expr::Integer(2)))),
                        }),
                    },
                    Stmt::Let {
                        name: "b".to_string(),
                        mutable: false,
                        value: Some(Expr::Match {
                            value: Box::new(Expr::Integer(1)),
                            arms: vec![
                                crate::frontend::ast::MatchArm::new(
                                    crate::frontend::ast::Pattern::Integer(1),
                                    Expr::Integer(3),
                                ),
                                crate::frontend::ast::MatchArm::new(
                                    crate::frontend::ast::Pattern::Wildcard,
                                    Expr::Integer(4),
                                ),
                            ],
                        }),
                    },
                ],
                Some(Expr::Identifier("a".to_string())),
            ),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        let assign_temp_count = mir
            .blocks
            .iter()
            .flat_map(|block| block.statements.iter())
            .filter(|stmt| matches!(stmt.kind, MirStatementKind::AssignPlace { place: MirPlace::Temp(_), .. }))
            .count();

        let join_temp_count = mir
            .blocks
            .iter()
            .flat_map(|block| block.statements.iter())
            .filter(|stmt| matches!(stmt.kind, MirStatementKind::JoinPlace { place: MirPlace::Temp(_), .. }))
            .count();

        assert!(assign_temp_count >= 2);
        assert!(join_temp_count >= 2);
        assert!(mir
            .blocks
            .iter()
            .any(|block| matches!(block.terminator, MirTerminator::Branch { .. })));
    }

    #[test]
    fn lower_from_tir_canonicalizes_loops_with_latch_and_exit_join() {
        let program = Program::new(vec![Item::Function(crate::frontend::ast::FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: crate::frontend::ast::FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Some(crate::frontend::ast::TypeExpr::Path(crate::frontend::ast::PathExpr::new(
                vec!["int".to_string()],
                Vec::new(),
            ))),
            body: crate::frontend::ast::Block::new(
                vec![
                    Stmt::Expr(Expr::While {
                        condition: Box::new(Expr::Boolean(true)),
                        body: crate::frontend::ast::Block::new(vec![Stmt::Expr(Expr::Integer(1))], None),
                    }),
                    Stmt::Expr(Expr::For {
                        binding: "v".to_string(),
                        iterable: Box::new(Expr::Identifier("values".to_string())),
                        body: crate::frontend::ast::Block::new(vec![Stmt::Expr(Expr::Integer(2))], None),
                    }),
                ],
                Some(Expr::Integer(0)),
            ),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        let join_temp_count = mir
            .blocks
            .iter()
            .flat_map(|block| block.statements.iter())
            .filter(|stmt| matches!(stmt.kind, MirStatementKind::JoinPlace { place: MirPlace::Temp(_), .. }))
            .count();

        let goto_count = mir
            .blocks
            .iter()
            .filter(|block| matches!(block.terminator, MirTerminator::Goto(_)))
            .count();

        assert!(join_temp_count >= 2);
        assert!(goto_count >= 3);
    }

    #[test]
    fn verify_lowered_program_accepts_valid_output() {
        let program = Program::new(vec![Item::Function(crate::frontend::ast::FunctionItem {
            decorators: Vec::new(),
            visibility: crate::frontend::ast::Visibility::Private,
            kind: crate::frontend::ast::FunctionKind::Fn,
            name: "main".to_string(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: Some(crate::frontend::ast::TypeExpr::Path(crate::frontend::ast::PathExpr::new(
                vec!["int".to_string()],
                Vec::new(),
            ))),
            body: crate::frontend::ast::Block::new(vec![], Some(Expr::Integer(1))),
        })]);

        let hir = build_hir(&program, ModulePath::root());
        let tir = TypeChecker::new(ModulePath::root()).check_hir(&hir);
        let mir = lower_from_tir(&tir, &hir);

        let diagnostics = verify_lowered_program(&mir);
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn verify_lowered_program_reports_invalid_edges_and_joins() {
        let invalid = LoweredProgram {
            blocks: vec![
                MirBasicBlock {
                    id: 0,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(9),
                },
                MirBasicBlock {
                    id: 2,
                    statements: vec![MirStatement {
                        id: 0,
                        kind: MirStatementKind::JoinPlace {
                            place: MirPlace::Temp(0),
                            incoming_blocks: vec![0, 0],
                        },
                    }],
                    terminator: MirTerminator::Return(None),
                },
            ],
            temp_count: 1,
            item_count: 0,
            symbol_count: 0,
            type_count: 0,
        };

        let diagnostics = verify_lowered_program(&invalid);
        assert!(diagnostics.iter().any(|diag| diag.code == "AXIS-MIR-001"));
        assert!(diagnostics.iter().any(|diag| diag.code == "AXIS-MIR-002"));
        assert!(diagnostics.iter().any(|diag| diag.code == "AXIS-MIR-006"));
        assert!(diagnostics.iter().any(|diag| diag.code == "AXIS-MIR-007"));
    }

    #[test]
    fn analyze_cfg_computes_predecessors_successors_and_rpo() {
        let mir = LoweredProgram {
            blocks: vec![
                MirBasicBlock {
                    id: 0,
                    statements: Vec::new(),
                    terminator: MirTerminator::Branch {
                        condition: MirValue::Boolean(true),
                        then_block: 1,
                        else_block: 2,
                    },
                },
                MirBasicBlock {
                    id: 1,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(3),
                },
                MirBasicBlock {
                    id: 2,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(3),
                },
                MirBasicBlock {
                    id: 3,
                    statements: Vec::new(),
                    terminator: MirTerminator::Return(None),
                },
            ],
            temp_count: 0,
            item_count: 0,
            symbol_count: 0,
            type_count: 0,
        };

        let analysis = analyze_cfg(&mir);

        assert_eq!(analysis.successors[0], vec![1, 2]);
        assert_eq!(analysis.predecessors[3], vec![1, 2]);
        assert_eq!(analysis.reverse_post_order.first().copied(), Some(0));
        assert!(analysis.reverse_post_order.contains(&3));
    }

    #[test]
    fn analyze_cfg_computes_basic_dominators() {
        let mir = LoweredProgram {
            blocks: vec![
                MirBasicBlock {
                    id: 0,
                    statements: Vec::new(),
                    terminator: MirTerminator::Branch {
                        condition: MirValue::Boolean(true),
                        then_block: 1,
                        else_block: 2,
                    },
                },
                MirBasicBlock {
                    id: 1,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(3),
                },
                MirBasicBlock {
                    id: 2,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(3),
                },
                MirBasicBlock {
                    id: 3,
                    statements: Vec::new(),
                    terminator: MirTerminator::Return(None),
                },
            ],
            temp_count: 0,
            item_count: 0,
            symbol_count: 0,
            type_count: 0,
        };

        let analysis = analyze_cfg(&mir);

        assert_eq!(analysis.dominators[0], BTreeSet::from([0]));
        assert_eq!(analysis.dominators[1], BTreeSet::from([0, 1]));
        assert_eq!(analysis.dominators[2], BTreeSet::from([0, 2]));
        assert_eq!(analysis.dominators[3], BTreeSet::from([0, 3]));
    }

    #[test]
    fn build_ssa_scaffold_emits_phi_from_join_place() {
        let mir = LoweredProgram {
            blocks: vec![
                MirBasicBlock {
                    id: 0,
                    statements: vec![MirStatement {
                        id: 0,
                        kind: MirStatementKind::JoinPlace {
                            place: MirPlace::Temp(0),
                            incoming_blocks: vec![1, 2],
                        },
                    }],
                    terminator: MirTerminator::Return(None),
                },
                MirBasicBlock {
                    id: 1,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(0),
                },
                MirBasicBlock {
                    id: 2,
                    statements: Vec::new(),
                    terminator: MirTerminator::Goto(0),
                },
            ],
            temp_count: 1,
            item_count: 0,
            symbol_count: 0,
            type_count: 0,
        };

        let ssa = build_ssa_scaffold(&mir);
        assert_eq!(ssa.blocks[0].phis.len(), 1);
        assert_eq!(ssa.blocks[0].phis[0].target.place, MirPlace::Temp(0));
        let incoming_blocks: Vec<usize> = ssa.blocks[0].phis[0].incoming.iter().map(|incoming| incoming.block).collect();
        assert_eq!(incoming_blocks, vec![1, 2]);
        assert!(ssa.blocks[0].phis[0]
            .incoming
            .iter()
            .all(|incoming| matches!(incoming.value, SsaValue::UnresolvedPlace(MirPlace::Temp(0)))));
    }

    #[test]
    fn build_ssa_scaffold_versions_assignments_per_place() {
        let mir = LoweredProgram {
            blocks: vec![MirBasicBlock {
                id: 0,
                statements: vec![
                    MirStatement {
                        id: 0,
                        kind: MirStatementKind::AssignPlace {
                            place: MirPlace::Local(Some(SymbolId(1))),
                            value: MirValue::Integer(1),
                        },
                    },
                    MirStatement {
                        id: 1,
                        kind: MirStatementKind::AssignPlace {
                            place: MirPlace::Local(Some(SymbolId(1))),
                            value: MirValue::Integer(2),
                        },
                    },
                ],
                terminator: MirTerminator::Return(None),
            }],
            temp_count: 0,
            item_count: 0,
            symbol_count: 0,
            type_count: 0,
        };

        let ssa = build_ssa_scaffold(&mir);
        let assignments: Vec<&SsaStatementKind> = ssa.blocks[0].statements.iter().map(|stmt| &stmt.kind).collect();

        match assignments.as_slice() {
            [
                SsaStatementKind::Assign { target: first, .. },
                SsaStatementKind::Assign { target: second, .. },
            ] => {
                assert_eq!(first.version, 1);
                assert_eq!(second.version, 2);
            }
            other => panic!("unexpected assignments: {other:?}"),
        }
    }

    #[test]
    fn build_ssa_scaffold_maps_uses_to_current_versions() {
        let mir = LoweredProgram {
            blocks: vec![MirBasicBlock {
                id: 0,
                statements: vec![
                    MirStatement {
                        id: 0,
                        kind: MirStatementKind::AssignPlace {
                            place: MirPlace::Local(Some(SymbolId(9))),
                            value: MirValue::Integer(7),
                        },
                    },
                    MirStatement {
                        id: 1,
                        kind: MirStatementKind::Eval(MirValue::SymbolRef(Some(SymbolId(9)))),
                    },
                ],
                terminator: MirTerminator::Return(Some(MirValue::SymbolRef(Some(SymbolId(9))))),
            }],
            temp_count: 0,
            item_count: 0,
            symbol_count: 0,
            type_count: 0,
        };

        let ssa = build_ssa_scaffold(&mir);
        let block = &ssa.blocks[0];

        match &block.statements[1].kind {
            SsaStatementKind::Eval(SsaValue::Name(name)) => {
                assert_eq!(name.place, MirPlace::Local(Some(SymbolId(9))));
                assert_eq!(name.version, 1);
            }
            other => panic!("unexpected eval mapping: {other:?}"),
        }

        match &block.terminator {
            SsaTerminator::Return(Some(SsaValue::Name(name))) => {
                assert_eq!(name.place, MirPlace::Local(Some(SymbolId(9))));
                assert_eq!(name.version, 1);
            }
            other => panic!("unexpected terminator mapping: {other:?}"),
        }
    }

    #[test]
    fn verify_ssa_scaffold_flags_unresolved_non_phi_uses_as_errors() {
        let ssa = LoweredSsaProgram {
            blocks: vec![SsaBasicBlock {
                id: 0,
                phis: Vec::new(),
                statements: vec![SsaStatement {
                    id: 0,
                    kind: SsaStatementKind::Eval(SsaValue::UnresolvedPlace(MirPlace::Temp(1))),
                }],
                terminator: SsaTerminator::Return(None),
            }],
            value_count: 0,
        };

        let diagnostics = verify_ssa_scaffold(&ssa);
        assert!(diagnostics.iter().any(|diag| diag.code == "AXIS-SSA-002"));
    }

    #[test]
    fn verify_ssa_scaffold_allows_unresolved_phi_as_warning() {
        let ssa = LoweredSsaProgram {
            blocks: vec![SsaBasicBlock {
                id: 0,
                phis: vec![SsaPhi {
                    target: SsaName {
                        place: MirPlace::Temp(0),
                        version: 1,
                    },
                    incoming: vec![SsaPhiIncoming {
                        block: 1,
                        value: SsaValue::UnresolvedPlace(MirPlace::Temp(0)),
                    }],
                }],
                statements: Vec::new(),
                terminator: SsaTerminator::Return(None),
            }],
            value_count: 1,
        };

        let diagnostics = verify_ssa_scaffold(&ssa);
        assert!(diagnostics.iter().any(|diag| diag.code == "AXIS-SSA-001"));
        assert!(!diagnostics.iter().any(|diag| diag.code == "AXIS-SSA-002"));
    }
}