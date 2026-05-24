use super::ast::{
    ArenaItem, BinaryOp, Block, Decorator, EnumItem, EnumVariant, Expr, Field, FunctionKind, FunctionItem,
    GenericParam, ImplItem, Item, MatchArm, ModuleItem, Param, PathExpr, Pattern, Program, Stmt,
    StructItem, TypeAliasItem, TypeExpr, UnaryOp, UseItem, Visibility,
};
use super::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Parser<'source> {
    lexer: Lexer<'source>,
    current: Token,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        Self { lexer, current }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut items = Vec::new();

        while self.current != Token::EndOfFile {
            items.push(self.parse_item());
        }

        Program::new(items)
    }

    fn parse_item(&mut self) -> Item {
        let decorators = self.parse_decorators();
        let visibility = self.parse_visibility();

        match self.current.clone() {
            Token::Identifier(ref name) if name == "fn" => self.parse_function_item(decorators, visibility, FunctionKind::Fn),
            Token::Identifier(ref name) if name == "task" => self.parse_function_item(decorators, visibility, FunctionKind::Task),
            Token::Identifier(ref name) if name == "struct" => self.parse_struct_item(decorators, visibility),
            Token::Identifier(ref name) if name == "enum" => self.parse_enum_item(decorators, visibility),
            Token::Identifier(ref name) if name == "type" => self.parse_type_alias_item(decorators, visibility),
            Token::Identifier(ref name) if name == "impl" => self.parse_impl_item(decorators),
            Token::Identifier(ref name) if name == "mod" => self.parse_module_item(decorators, visibility),
            Token::Identifier(ref name) if name == "use" => self.parse_use_item(decorators),
            Token::Identifier(ref name) if name == "arena" => self.parse_arena_item(),
            _ => Item::Stmt(self.parse_statement()),
        }
    }

    fn parse_decorators(&mut self) -> Vec<Decorator> {
        let mut decorators = Vec::new();

        while self.current == Token::At {
            self.advance();
            let name = self.expect_identifier();
            let arguments = if self.current == Token::LParen {
                self.advance();
                let mut arguments = Vec::new();
                if self.current != Token::RParen {
                    loop {
                        arguments.push(self.parse_expression());
                        if self.current == Token::Comma {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                self.expect_token(Token::RParen);
                arguments
            } else {
                Vec::new()
            };
            decorators.push(Decorator { name, arguments });
        }

        decorators
    }

    fn parse_visibility(&mut self) -> Visibility {
        if self.is_identifier("pub") {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    fn parse_function_item(&mut self, decorators: Vec<Decorator>, visibility: Visibility, kind: FunctionKind) -> Item {
        self.advance();
        let name = self.expect_identifier();
        let generics = self.parse_generic_params();
        let params = self.parse_params();
        let return_type = if self.current == Token::Arrow {
            self.advance();
            Some(self.parse_type_expr())
        } else {
            None
        };
        let body = self.parse_block();

        Item::Function(FunctionItem {
            decorators,
            visibility,
            kind,
            name,
            generics,
            params,
            return_type,
            body,
        })
    }

    fn parse_struct_item(&mut self, decorators: Vec<Decorator>, visibility: Visibility) -> Item {
        self.advance();
        let name = self.expect_identifier();
        let generics = self.parse_generic_params();
        let fields = if self.current == Token::LBrace {
            self.advance();
            let mut fields = Vec::new();
            while self.current != Token::RBrace && self.current != Token::EndOfFile {
                let field_name = self.expect_identifier();
                self.expect_token(Token::Colon);
                let ty = self.parse_type_expr();
                fields.push(Field { name: field_name, ty });
                if self.current == Token::Comma {
                    self.advance();
                }
            }
            self.expect_token(Token::RBrace);
            fields
        } else {
            Vec::new()
        };

        Item::Struct(StructItem {
            decorators,
            visibility,
            name,
            generics,
            fields,
        })
    }

    fn parse_enum_item(&mut self, decorators: Vec<Decorator>, visibility: Visibility) -> Item {
        self.advance();
        let name = self.expect_identifier();
        let generics = self.parse_generic_params();
        self.expect_token(Token::LBrace);
        let mut variants = Vec::new();
        while self.current != Token::RBrace && self.current != Token::EndOfFile {
            let variant_name = self.expect_identifier();
            let fields = if self.current == Token::LParen {
                self.advance();
                let mut fields = Vec::new();
                if self.current != Token::RParen {
                    loop {
                        fields.push(self.parse_type_expr());
                        if self.current == Token::Comma {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                self.expect_token(Token::RParen);
                fields
            } else {
                Vec::new()
            };
            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });
            if self.current == Token::Comma {
                self.advance();
            }
        }
        self.expect_token(Token::RBrace);

        Item::Enum(EnumItem {
            decorators,
            visibility,
            name,
            generics,
            variants,
        })
    }

    fn parse_type_alias_item(&mut self, decorators: Vec<Decorator>, visibility: Visibility) -> Item {
        self.advance();
        let name = self.expect_identifier();
        let generics = self.parse_generic_params();
        self.expect_token(Token::Equal);
        let target = self.parse_type_expr();
        self.expect_token(Token::Semicolon);

        Item::TypeAlias(TypeAliasItem {
            decorators,
            visibility,
            name,
            generics,
            target,
        })
    }

    fn parse_impl_item(&mut self, decorators: Vec<Decorator>) -> Item {
        self.advance();
        let target = self.parse_type_expr();
        self.expect_token(Token::LBrace);
        let mut items = Vec::new();
        while self.current != Token::RBrace && self.current != Token::EndOfFile {
            items.push(self.parse_item());
        }
        self.expect_token(Token::RBrace);

        Item::Impl(ImplItem {
            decorators,
            target,
            items,
        })
    }

    fn parse_module_item(&mut self, decorators: Vec<Decorator>, visibility: Visibility) -> Item {
        self.advance();
        let name = self.expect_identifier();
        let items = if self.current == Token::LBrace {
            self.advance();
            let mut items = Vec::new();
            while self.current != Token::RBrace && self.current != Token::EndOfFile {
                items.push(self.parse_item());
            }
            self.expect_token(Token::RBrace);
            items
        } else {
            self.expect_token(Token::Semicolon);
            Vec::new()
        };

        Item::Module(ModuleItem {
            decorators,
            visibility,
            name,
            items,
        })
    }

    fn parse_use_item(&mut self, decorators: Vec<Decorator>) -> Item {
        self.advance();
        let path = self.parse_path_expr();
        let alias = if self.is_identifier("as") {
            self.advance();
            Some(self.expect_identifier())
        } else {
            None
        };
        self.expect_token(Token::Semicolon);

        Item::Use(UseItem {
            decorators,
            path,
            alias,
        })
    }

    fn parse_arena_item(&mut self) -> Item {
        self.advance();
        let name = self.expect_identifier();
        let body = self.parse_block();

        Item::Arena(ArenaItem { name, body })
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.current.clone() {
            Token::Identifier(ref name) if name == "let" => self.parse_let_statement(false),
            Token::Identifier(ref name) if name == "var" => self.parse_let_statement(true),
            Token::Identifier(ref name) if name == "return" => self.parse_return_statement(),
            _ => {
                let expr = self.parse_expression();
                self.expect_token(Token::Semicolon);
                Stmt::Expr(expr)
            }
        }
    }

    fn parse_let_statement(&mut self, mutable: bool) -> Stmt {
        self.advance();
        let name = self.expect_identifier();
        let value = if self.current == Token::Equal {
            self.advance();
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_token(Token::Semicolon);
        Stmt::Let { name, mutable, value }
    }

    fn parse_return_statement(&mut self) -> Stmt {
        self.advance();
        let value = if self.current == Token::Semicolon {
            None
        } else {
            Some(self.parse_expression())
        };
        self.expect_token(Token::Semicolon);
        Stmt::Return(value)
    }

    fn parse_block(&mut self) -> Block {
        self.expect_token(Token::LBrace);
        let mut statements = Vec::new();
        let mut tail = None;

        while self.current != Token::RBrace && self.current != Token::EndOfFile {
            match self.current.clone() {
                Token::Identifier(ref name) if name == "let" => {
                    statements.push(self.parse_let_statement(false));
                }
                Token::Identifier(ref name) if name == "var" => {
                    statements.push(self.parse_let_statement(true));
                }
                Token::Identifier(ref name) if name == "return" => {
                    statements.push(self.parse_return_statement());
                }
                _ => {
                    let expr = self.parse_expression();
                    if self.current == Token::Semicolon {
                        self.advance();
                        statements.push(Stmt::Expr(expr));
                    } else {
                        tail = Some(expr);
                        break;
                    }
                }
            }
        }

        self.expect_token(Token::RBrace);
        Block::new(statements, tail)
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_range()
    }

    fn parse_range(&mut self) -> Expr {
        let start = self.parse_comparison();
        if self.current == Token::DoubleDot {
            self.advance();
            let end = self.parse_comparison();
            Expr::Range {
                start: Box::new(start),
                end: Box::new(end),
            }
        } else {
            start
        }
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_additive();
        loop {
            let op = match self.current {
                Token::Less => Some(BinaryOp::Less),
                Token::LessEqual => Some(BinaryOp::LessEqual),
                Token::Greater => Some(BinaryOp::Greater),
                Token::GreaterEqual => Some(BinaryOp::GreaterEqual),
                Token::EqualEqual => Some(BinaryOp::EqualEqual),
                Token::BangEqual => Some(BinaryOp::NotEqual),
                _ => None,
            };

            let Some(op) = op else {
                break;
            };

            self.advance();
            let right = self.parse_additive();
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_additive(&mut self) -> Expr {
        let mut expr = self.parse_multiplicative();
        loop {
            let op = match self.current {
                Token::Plus => Some(BinaryOp::Add),
                Token::Minus => Some(BinaryOp::Sub),
                _ => None,
            };

            let Some(op) = op else {
                break;
            };

            self.advance();
            let right = self.parse_multiplicative();
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_multiplicative(&mut self) -> Expr {
        let mut expr = self.parse_unary();
        loop {
            let op = match self.current {
                Token::Star => Some(BinaryOp::Mul),
                Token::Slash => Some(BinaryOp::Div),
                _ => None,
            };

            let Some(op) = op else {
                break;
            };

            self.advance();
            let right = self.parse_unary();
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_unary(&mut self) -> Expr {
        if self.current == Token::Minus {
            self.advance();
            Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(self.parse_unary()),
            }
        } else {
            let primary = self.parse_primary();
            self.parse_postfix(primary)
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current.clone() {
            Token::Identifier(ref name) if name == "if" => self.parse_if_expression(),
            Token::Identifier(ref name) if name == "while" => self.parse_while_expression(),
            Token::Identifier(ref name) if name == "for" => self.parse_for_expression(),
            Token::Identifier(ref name) if name == "match" => self.parse_match_expression(),
            Token::Identifier(ref name) if name == "arena" => self.parse_arena_expression(),
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "true" => Expr::Boolean(true),
                    "false" => Expr::Boolean(false),
                    _ => {
                        if self.current == Token::ColonColon {
                            let mut segments = vec![name];
                            while self.current == Token::ColonColon {
                                self.advance();
                                segments.push(self.expect_identifier());
                            }
                            Expr::Path(PathExpr::new(segments, Vec::new()))
                        } else {
                            Expr::Identifier(name)
                        }
                    }
                }
            }
            Token::Integer(value) => {
                self.advance();
                Expr::Integer(value)
            }
            Token::Float(value) => {
                self.advance();
                Expr::Float(value)
            }
            Token::StringLiteral(value) => {
                self.advance();
                Expr::String(value)
            }
            Token::CharLiteral(value) => {
                self.advance();
                Expr::Char(value)
            }
            Token::LParen => self.parse_parenthesized_expression(),
            Token::LBrace => Expr::Block(self.parse_block()),
            _ => {
                self.advance();
                Expr::Identifier(String::new())
            }
        }
    }

    fn parse_parenthesized_expression(&mut self) -> Expr {
        self.expect_token(Token::LParen);
        if self.current == Token::RParen {
            self.advance();
            return Expr::Tuple(Vec::new());
        }

        let first = self.parse_expression();
        if self.current == Token::Comma {
            let mut values = vec![first];
            while self.current == Token::Comma {
                self.advance();
                if self.current == Token::RParen {
                    break;
                }
                values.push(self.parse_expression());
            }
            self.expect_token(Token::RParen);
            Expr::Tuple(values)
        } else {
            self.expect_token(Token::RParen);
            first
        }
    }

    fn parse_if_expression(&mut self) -> Expr {
        self.advance();
        let condition = self.parse_expression();
        let then_branch = self.parse_block();
        let else_branch = if self.is_identifier("else") {
            self.advance();
            Some(self.parse_block())
        } else {
            None
        };

        Expr::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        }
    }

    fn parse_while_expression(&mut self) -> Expr {
        self.advance();
        let condition = self.parse_expression();
        let body = self.parse_block();

        Expr::While {
            condition: Box::new(condition),
            body,
        }
    }

    fn parse_for_expression(&mut self) -> Expr {
        self.advance();
        let binding = self.expect_identifier();
        self.expect_identifier_value("in");
        let iterable = self.parse_expression();
        let body = self.parse_block();

        Expr::For {
            binding,
            iterable: Box::new(iterable),
            body,
        }
    }

    fn parse_match_expression(&mut self) -> Expr {
        self.advance();
        let value = self.parse_expression();
        self.expect_token(Token::LBrace);

        let mut arms = Vec::new();
        while self.current != Token::RBrace && self.current != Token::EndOfFile {
            let pattern = self.parse_pattern();
            self.expect_token(Token::FatArrow);
            let arm_value = self.parse_expression();
            if self.current == Token::Comma || self.current == Token::Semicolon {
                self.advance();
            }
            arms.push(MatchArm::new(pattern, arm_value));
        }

        self.expect_token(Token::RBrace);
        Expr::Match {
            value: Box::new(value),
            arms,
        }
    }

    fn parse_arena_expression(&mut self) -> Expr {
        self.advance();
        let name = self.expect_identifier();
        let body = self.parse_block();
        Expr::Arena { name, body }
    }

    fn parse_pattern(&mut self) -> Pattern {
        match self.current.clone() {
            Token::Identifier(name) if name == "_" => {
                self.advance();
                Pattern::Wildcard
            }
            Token::Identifier(name) if name == "true" => {
                self.advance();
                Pattern::Boolean(true)
            }
            Token::Identifier(name) if name == "false" => {
                self.advance();
                Pattern::Boolean(false)
            }
            Token::Identifier(name) => {
                self.advance();
                if self.current == Token::LParen {
                    self.advance();
                    let mut bindings = Vec::new();
                    if self.current != Token::RParen {
                        loop {
                            bindings.push(self.parse_pattern());
                            if self.current == Token::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_token(Token::RParen);
                    Pattern::EnumVariant { name, bindings }
                } else {
                    Pattern::Identifier(name)
                }
            }
            Token::Integer(value) => {
                self.advance();
                Pattern::Integer(value)
            }
            Token::StringLiteral(value) => {
                self.advance();
                Pattern::String(value)
            }
            Token::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                if self.current != Token::RParen {
                    loop {
                        patterns.push(self.parse_pattern());
                        if self.current == Token::Comma {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                self.expect_token(Token::RParen);
                Pattern::Tuple(patterns)
            }
            _ => {
                self.advance();
                Pattern::Wildcard
            }
        }
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Expr {
        loop {
            match self.current.clone() {
                Token::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.current != Token::RParen {
                        loop {
                            args.push(self.parse_expression());
                            if self.current == Token::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_token(Token::RParen);
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Token::Dot => {
                    self.advance();
                    let name = self.expect_identifier();
                    expr = Expr::Field {
                        target: Box::new(expr),
                        name,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expression();
                    self.expect_token(Token::RBracket);
                    expr = Expr::Index {
                        target: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Question => {
                    self.advance();
                    expr = Expr::Try(Box::new(expr));
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_generic_params(&mut self) -> Vec<GenericParam> {
        if self.current != Token::Less {
            return Vec::new();
        }

        self.advance();
        let mut params = Vec::new();
        while self.current != Token::Greater && self.current != Token::EndOfFile {
            let name = self.expect_identifier();
            let mut bounds = Vec::new();
            if self.current == Token::Colon {
                self.advance();
                bounds.push(self.parse_type_expr());
                while self.current == Token::Plus {
                    self.advance();
                    bounds.push(self.parse_type_expr());
                }
            }
            params.push(GenericParam { name, bounds });
            if self.current == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_token(Token::Greater);
        params
    }

    fn parse_params(&mut self) -> Vec<Param> {
        self.expect_token(Token::LParen);
        let mut params = Vec::new();
        while self.current != Token::RParen && self.current != Token::EndOfFile {
            let name = self.expect_identifier();
            let ty = if self.current == Token::Colon {
                self.advance();
                Some(self.parse_type_expr())
            } else {
                None
            };
            params.push(Param { name, ty });
            if self.current == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_token(Token::RParen);
        params
    }

    fn parse_type_expr(&mut self) -> TypeExpr {
        match self.current.clone() {
            Token::Ampersand => {
                self.advance();
                let mutable = if self.is_identifier("mut") {
                    self.advance();
                    true
                } else {
                    false
                };
                TypeExpr::Reference {
                    mutable,
                    target: Box::new(self.parse_type_expr()),
                }
            }
            Token::LParen => {
                self.advance();
                if self.current == Token::RParen {
                    self.advance();
                    TypeExpr::Unit
                } else {
                    let mut types = Vec::new();
                    loop {
                        types.push(self.parse_type_expr());
                        if self.current == Token::Comma {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                    self.expect_token(Token::RParen);
                    TypeExpr::Tuple(types)
                }
            }
            Token::LBracket => {
                self.advance();
                let element = self.parse_type_expr();
                self.expect_token(Token::Semicolon);
                let length = match self.current.clone() {
                    Token::Integer(value) => {
                        self.advance();
                        value as usize
                    }
                    _ => {
                        self.advance();
                        0
                    }
                };
                self.expect_token(Token::RBracket);
                TypeExpr::Array {
                    element: Box::new(element),
                    length,
                }
            }
            Token::Identifier(_) => TypeExpr::Path(self.parse_path_expr_with_type_args()),
            _ => {
                self.advance();
                TypeExpr::Unit
            }
        }
    }

    fn parse_path_expr_with_type_args(&mut self) -> PathExpr {
        let mut segments = vec![self.expect_identifier()];
        while self.current == Token::ColonColon {
            self.advance();
            segments.push(self.expect_identifier());
        }

        let arguments = if self.current == Token::Less {
            self.advance();
            let mut arguments = Vec::new();
            while self.current != Token::Greater && self.current != Token::EndOfFile {
                arguments.push(self.parse_type_expr());
                if self.current == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_token(Token::Greater);
            arguments
        } else {
            Vec::new()
        };

        PathExpr::new(segments, arguments)
    }

    fn parse_path_expr(&mut self) -> PathExpr {
        self.parse_path_expr_with_type_args()
    }

    fn expect_token(&mut self, expected: Token) {
        if self.current == expected {
            self.advance();
        }
    }

    fn expect_identifier(&mut self) -> String {
        match self.current.clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            }
            _ => {
                self.advance();
                String::new()
            }
        }
    }

    fn expect_identifier_value(&mut self, expected: &str) {
        if self.is_identifier(expected) {
            self.advance();
        }
    }

    fn is_identifier(&self, expected: &str) -> bool {
        matches!(&self.current, Token::Identifier(name) if name == expected)
    }

    fn advance(&mut self) {
        self.current = self.lexer.next_token();
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::frontend::ast::{
        ArenaItem, BinaryOp, Block, EnumItem, Expr, FunctionKind, ImplItem, Item, ModuleItem,
        PathExpr, Pattern, Stmt, StructItem, TypeAliasItem, TypeExpr, UseItem, Visibility,
    };

    #[test]
    fn parses_identifier_statement() {
        let mut parser = Parser::new("alpha;");
        let program = parser.parse_program();

        assert_eq!(program.items.len(), 1);
        assert_eq!(program.items[0], Item::Stmt(Stmt::Expr(Expr::Identifier("alpha".to_string()))));
    }

    #[test]
    fn parses_integer_statement() {
        let mut parser = Parser::new("42;");
        let program = parser.parse_program();

        assert_eq!(program.items.len(), 1);
        assert_eq!(program.items[0], Item::Stmt(Stmt::Expr(Expr::Integer(42))));
    }

    #[test]
    fn parses_let_and_return_statements() {
        let mut parser = Parser::new("let value = 3; return value;");
        let program = parser.parse_program();

        assert_eq!(program.items.len(), 2);
        assert_eq!(
            program.items[0],
            Item::Stmt(Stmt::Let {
                name: "value".to_string(),
                mutable: false,
                value: Some(Expr::Integer(3)),
            })
        );
        assert_eq!(
            program.items[1],
            Item::Stmt(Stmt::Return(Some(Expr::Identifier("value".to_string()))))
        );
    }

    #[test]
    fn parses_function_item_with_types_and_generics() {
        let mut parser = Parser::new("pub fn add<T: Ord>(a: T, b: T) -> T { a }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Function(function) => {
                assert_eq!(function.visibility, Visibility::Public);
                assert_eq!(function.kind, FunctionKind::Fn);
                assert_eq!(function.name, "add");
                assert_eq!(function.generics.len(), 1);
                assert_eq!(function.params.len(), 2);
                assert_eq!(function.return_type, Some(TypeExpr::Path(PathExpr::new(vec!["T".to_string()], vec![]))));
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_task_function_item() {
        let mut parser = Parser::new("task fetch(url: &str) -> Result<Response, IoError> { url }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Function(function) => {
                assert_eq!(function.kind, FunctionKind::Task);
                assert_eq!(function.name, "fetch");
                assert_eq!(function.params.len(), 1);
                assert!(matches!(function.params[0].ty, Some(TypeExpr::Reference { .. })));
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_struct_and_enum_items() {
        let mut parser = Parser::new("struct Point { x: f32, y: f32 } enum Option<T> { Some(T), None }");
        let program = parser.parse_program();

        assert!(matches!(program.items[0], Item::Struct(StructItem { .. })));
        assert!(matches!(program.items[1], Item::Enum(EnumItem { .. })));
    }

    #[test]
    fn parses_type_alias_and_impl_items() {
        let mut parser = Parser::new("type Alias = Result<i32, String>; impl Point { fn length(self) -> f32 { 0; } }");
        let program = parser.parse_program();

        assert!(matches!(program.items[0], Item::TypeAlias(TypeAliasItem { .. })));
        assert!(matches!(program.items[1], Item::Impl(ImplItem { .. })));
    }

    #[test]
    fn parses_module_and_use_items() {
        let mut parser = Parser::new("mod math { pub fn add(a: i32, b: i32) -> i32 { a + b } } use fs::read_to_string as read;");
        let program = parser.parse_program();

        assert!(matches!(program.items[0], Item::Module(ModuleItem { .. })));
        assert!(matches!(program.items[1], Item::Use(UseItem { .. })));
    }

    #[test]
    fn parses_block_with_tail_expression() {
        let mut parser = Parser::new("{ let x = 1; x }");
        let program = parser.parse_program();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::Block(Block { statements, tail }))) => {
                assert_eq!(statements.len(), 1);
                assert_eq!(tail.as_deref(), Some(&Expr::Identifier("x".to_string())));
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_if_else_expression() {
        let mut parser = Parser::new("if true { 1; } else { 2; }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::If {
                condition,
                then_branch,
                else_branch,
            })) => {
                assert_eq!(**condition, Expr::Boolean(true));
                assert_eq!(then_branch.statements.len(), 1);
                assert!(else_branch.is_some());
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_while_expression() {
        let mut parser = Parser::new("while true { return 1; }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::While { condition, body })) => {
                assert_eq!(**condition, Expr::Boolean(true));
                assert_eq!(body.statements.len(), 1);
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_for_expression() {
        let mut parser = Parser::new("for item in items { item; }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::For {
                binding,
                iterable,
                body,
            })) => {
                assert_eq!(binding, "item");
                assert_eq!(**iterable, Expr::Identifier("items".to_string()));
                assert_eq!(body.statements.len(), 1);
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_match_expression() {
        let mut parser = Parser::new("match value { Some(x) => x, None => 0 }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::Match { value, arms })) => {
                assert_eq!(**value, Expr::Identifier("value".to_string()));
                assert_eq!(arms.len(), 2);
                assert_eq!(
                    arms[0].pattern,
                    Pattern::EnumVariant {
                        name: "Some".to_string(),
                        bindings: vec![Pattern::Identifier("x".to_string())],
                    }
                );
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_call_field_index_and_try_postfixes() {
        let mut parser = Parser::new("value.call(1).field[2]?;");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::Try(inner))) => match &**inner {
                Expr::Index { target, index } => {
                    assert_eq!(**index, Expr::Integer(2));
                    match &**target {
                        Expr::Field { target, name } => {
                            assert_eq!(name, "field");
                            match &**target {
                                Expr::Call { callee, args } => {
                                    assert_eq!(args, &vec![Expr::Integer(1)]);
                                    assert_eq!(**callee, Expr::Field {
                                        target: Box::new(Expr::Identifier("value".to_string())),
                                        name: "call".to_string(),
                                    });
                                }
                                other => panic!("unexpected call AST: {other:?}"),
                            }
                        }
                        other => panic!("unexpected field AST: {other:?}"),
                    }
                }
                other => panic!("unexpected index AST: {other:?}"),
            },
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_arena_expression() {
        let mut parser = Parser::new("arena frame { let buf = 1; buf }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Arena(ArenaItem { name, body }) => {
                assert_eq!(name, "frame");
                assert_eq!(body.statements.len(), 1);
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_float_and_char_literals() {
        let mut parser = Parser::new("1.5; 'x';");
        let program = parser.parse_program();

        assert_eq!(program.items.len(), 2);
        assert!(matches!(program.items[0], Item::Stmt(Stmt::Expr(Expr::Float(_)))));
        assert_eq!(program.items[1], Item::Stmt(Stmt::Expr(Expr::Char('x'))));
    }

    #[test]
    fn parses_binary_expression_precedence_and_range() {
        let mut parser = Parser::new("1 + 2 * 3..10;");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Stmt(Stmt::Expr(Expr::Range { start, end })) => {
                assert!(matches!(**start, Expr::Binary { op: BinaryOp::Add, .. }));
                assert_eq!(**end, Expr::Integer(10));
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }

    #[test]
    fn parses_decorated_function_item() {
        let mut parser = Parser::new("@trusted_aliasing(\"explanation\") pub fn copy() { return; }");
        let program = parser.parse_program();

        match &program.items[0] {
            Item::Function(function) => {
                assert_eq!(function.decorators.len(), 1);
                assert_eq!(function.decorators[0].name, "trusted_aliasing");
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }
}
