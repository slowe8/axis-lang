#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    pub fn new(items: Vec<Item>) -> Self {
        Self { items }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Function(FunctionItem),
    Struct(StructItem),
    Enum(EnumItem),
    TypeAlias(TypeAliasItem),
    Impl(ImplItem),
    Module(ModuleItem),
    Use(UseItem),
    Arena(ArenaItem),
    Stmt(Stmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionKind {
    Fn,
    Task,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionItem {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub kind: FunctionKind,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructItem {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumItem {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasItem {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub target: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplItem {
    pub decorators: Vec<Decorator>,
    pub target: TypeExpr,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleItem {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseItem {
    pub decorators: Vec<Decorator>,
    pub path: PathExpr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArenaItem {
    pub name: String,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decorator {
    pub name: String,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Option<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<TypeExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathExpr {
    pub segments: Vec<String>,
    pub arguments: Vec<TypeExpr>,
}

impl PathExpr {
    pub fn new(segments: Vec<String>, arguments: Vec<TypeExpr>) -> Self {
        Self { segments, arguments }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Path(PathExpr),
    Reference {
        mutable: bool,
        target: Box<TypeExpr>,
    },
    Tuple(Vec<TypeExpr>),
    Array {
        element: Box<TypeExpr>,
        length: usize,
    },
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
}

impl Block {
    pub fn new(statements: Vec<Stmt>, tail: Option<Expr>) -> Self {
        Self {
            statements,
            tail: tail.map(Box::new),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        name: String,
        mutable: bool,
        value: Option<Expr>,
    },
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub value: Expr,
}

impl MatchArm {
    pub fn new(pattern: Pattern, value: Expr) -> Self {
        Self { pattern, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Identifier(String),
    Wildcard,
    Integer(i64),
    Boolean(bool),
    String(String),
    Tuple(Vec<Pattern>),
    EnumVariant {
        name: String,
        bindings: Vec<Pattern>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Range,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Identifier(String),
    Integer(i64),
    Float(String),
    Boolean(bool),
    String(String),
    Char(char),
    Path(PathExpr),
    Block(Block),
    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    While {
        condition: Box<Expr>,
        body: Block,
    },
    For {
        binding: String,
        iterable: Box<Expr>,
        body: Block,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Arena {
        name: String,
        body: Block,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Field {
        target: Box<Expr>,
        name: String,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Try(Box<Expr>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Tuple(Vec<Expr>),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_constructor_preserves_items() {
        let program = Program::new(vec![Item::Stmt(Stmt::Expr(Expr::Integer(1)))]);

        assert_eq!(program.items.len(), 1);
        assert_eq!(program.items[0], Item::Stmt(Stmt::Expr(Expr::Integer(1))));
    }

    #[test]
    fn block_constructor_preserves_tail_expression() {
        let block = Block::new(vec![Stmt::Expr(Expr::Integer(1))], Some(Expr::Identifier("done".to_string())));

        assert_eq!(block.statements.len(), 1);
        assert_eq!(block.tail.as_deref(), Some(&Expr::Identifier("done".to_string())));
    }

    #[test]
    fn path_expression_preserves_segments() {
        let path = PathExpr::new(vec!["fs".to_string(), "read_to_string".to_string()], vec![]);

        assert_eq!(path.segments, vec!["fs".to_string(), "read_to_string".to_string()]);
    }

    #[test]
    fn match_arm_constructor_preserves_pattern_and_value() {
        let arm = MatchArm::new(Pattern::Identifier("Some".to_string()), Expr::Integer(10));

        assert_eq!(arm.pattern, Pattern::Identifier("Some".to_string()));
        assert_eq!(arm.value, Expr::Integer(10));
    }

    #[test]
    fn pattern_variants_remain_comparable() {
        let pattern = Pattern::EnumVariant {
            name: "Pair".to_string(),
            bindings: vec![Pattern::Integer(1), Pattern::Wildcard],
        };

        assert_eq!(
            pattern,
            Pattern::EnumVariant {
                name: "Pair".to_string(),
                bindings: vec![Pattern::Integer(1), Pattern::Wildcard],
            }
        );
    }

    #[test]
    fn function_item_can_be_constructed() {
        let function = FunctionItem {
            decorators: vec![Decorator {
                name: "simd".to_string(),
                arguments: Vec::new(),
            }],
            visibility: Visibility::Public,
            kind: FunctionKind::Fn,
            name: "add".to_string(),
            generics: vec![GenericParam {
                name: "T".to_string(),
                bounds: vec![TypeExpr::Path(PathExpr::new(vec!["Ord".to_string()], Vec::new()))],
            }],
            params: vec![Param {
                name: "a".to_string(),
                ty: Some(TypeExpr::Path(PathExpr::new(vec!["T".to_string()], Vec::new()))),
            }],
            return_type: Some(TypeExpr::Path(PathExpr::new(vec!["T".to_string()], Vec::new()))),
            body: Block::new(vec![Stmt::Expr(Expr::Identifier("a".to_string()))], None),
        };

        assert_eq!(function.kind, FunctionKind::Fn);
        assert_eq!(function.generics.len(), 1);
        assert_eq!(function.params.len(), 1);
    }

    #[test]
    fn struct_enum_and_alias_items_can_be_constructed() {
        let struct_item = StructItem {
            decorators: Vec::new(),
            visibility: Visibility::Private,
            name: "Point".to_string(),
            generics: Vec::new(),
            fields: vec![Field {
                name: "x".to_string(),
                ty: TypeExpr::Path(PathExpr::new(vec!["f32".to_string()], Vec::new())),
            }],
        };

        let enum_item = EnumItem {
            decorators: Vec::new(),
            visibility: Visibility::Public,
            name: "Option".to_string(),
            generics: vec![GenericParam {
                name: "T".to_string(),
                bounds: Vec::new(),
            }],
            variants: vec![EnumVariant {
                name: "Some".to_string(),
                fields: vec![TypeExpr::Path(PathExpr::new(vec!["T".to_string()], Vec::new()))],
            }],
        };

        let alias_item = TypeAliasItem {
            decorators: Vec::new(),
            visibility: Visibility::Public,
            name: "Id".to_string(),
            generics: Vec::new(),
            target: TypeExpr::Path(PathExpr::new(vec!["u64".to_string()], Vec::new())),
        };

        assert_eq!(struct_item.fields.len(), 1);
        assert_eq!(enum_item.variants.len(), 1);
        assert_eq!(alias_item.name, "Id");
    }

    #[test]
    fn module_use_impl_and_arena_items_can_be_constructed() {
        let arena_item = ArenaItem {
            name: "frame".to_string(),
            body: Block::new(vec![Stmt::Expr(Expr::Integer(1))], Some(Expr::Identifier("done".to_string()))),
        };

        let module_item = ModuleItem {
            decorators: Vec::new(),
            visibility: Visibility::Public,
            name: "math".to_string(),
            items: vec![Item::Stmt(Stmt::Expr(Expr::Identifier("x".to_string())))],
        };

        let use_item = UseItem {
            decorators: Vec::new(),
            path: PathExpr::new(vec!["fs".to_string(), "read_to_string".to_string()], Vec::new()),
            alias: Some("read".to_string()),
        };

        let impl_item = ImplItem {
            decorators: Vec::new(),
            target: TypeExpr::Path(PathExpr::new(vec!["Point".to_string()], Vec::new())),
            items: vec![Item::Stmt(Stmt::Expr(Expr::Identifier("noop".to_string())))],
        };

        assert_eq!(arena_item.name, "frame");
        assert_eq!(module_item.items.len(), 1);
        assert_eq!(use_item.alias.as_deref(), Some("read"));
        assert_eq!(impl_item.items.len(), 1);
    }

    #[test]
    fn type_expression_variants_can_be_constructed() {
        let reference = TypeExpr::Reference {
            mutable: false,
            target: Box::new(TypeExpr::Path(PathExpr::new(vec!["str".to_string()], Vec::new()))),
        };
        let tuple = TypeExpr::Tuple(vec![
            TypeExpr::Path(PathExpr::new(vec!["i32".to_string()], Vec::new())),
            TypeExpr::Path(PathExpr::new(vec!["f32".to_string()], Vec::new())),
        ]);
        let array = TypeExpr::Array {
            element: Box::new(TypeExpr::Path(PathExpr::new(vec!["u8".to_string()], Vec::new()))),
            length: 4,
        };

        assert!(matches!(reference, TypeExpr::Reference { .. }));
        assert!(matches!(tuple, TypeExpr::Tuple(values) if values.len() == 2));
        assert!(matches!(array, TypeExpr::Array { length: 4, .. }));
    }
}
