use avalanche_model::source::SourceSpan;
use thiserror::Error;

use crate::ast::{BinOp, Binding, Expr, ImportRef, NixFile};
use crate::lexer::{tokenize, Token, TokenKind};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token '{found}' at line {line}, column {col}, expected {expected}")]
    UnexpectedToken {
        found: String,
        expected: String,
        line: u32,
        col: u32,
    },
    #[error("unexpected end of input, expected {expected}")]
    UnexpectedEof { expected: String },
    #[error("lex error: {0}")]
    Lex(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: TokenKind, expected: &str) -> Result<Token, ParseError> {
        let tok = self.advance().clone();
        if tok.kind != kind {
            return Err(self.unexpected(&tok, expected));
        }
        Ok(tok)
    }

    fn unexpected(&self, tok: &Token, expected: &str) -> ParseError {
        if tok.kind == TokenKind::Eof {
            ParseError::UnexpectedEof {
                expected: expected.to_string(),
            }
        } else {
            ParseError::UnexpectedToken {
                found: tok.text.clone(),
                expected: expected.to_string(),
                line: tok.span.start.line,
                col: tok.span.start.column,
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_implies()
    }

    fn parse_implies(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_or()?;
        while self.peek().text == "->" && self.peek().kind != TokenKind::Eof {
            self.advance();
            let right = self.parse_or()?;
            left = Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.peek().kind == TokenKind::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.peek().kind == TokenKind::And {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Eq => BinOp::Eq,
                TokenKind::Neq => BinOp::Neq,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_update()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.parse_update()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_update(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        while self.peek().kind == TokenKind::Update {
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                op: BinOp::Update,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Sub => BinOp::Sub,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_concat(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        while self.peek().kind == TokenKind::Concat {
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                op: BinOp::Concat,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Mul => BinOp::Mul,
                TokenKind::Div => BinOp::Div,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek().kind == TokenKind::Not {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Negate(Box::new(expr)));
        }
        if self.peek().kind == TokenKind::Sub {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Negate(Box::new(expr)));
        }
        self.parse_apply()
    }

    fn parse_apply(&mut self) -> Result<Expr, ParseError> {
        let mut func = self.parse_select()?;
        loop {
            let can_be_arg = match self.peek().kind {
                TokenKind::Int | TokenKind::Float | TokenKind::Str | TokenKind::Path
                | TokenKind::Ident | TokenKind::LBrace | TokenKind::LBracket
                | TokenKind::LParen | TokenKind::SearchPath => true,
                TokenKind::Keyword => {
                    let text = self.peek().text.as_str();
                    matches!(text, "true" | "false" | "null" | "let" | "if" | "with" | "assert" | "rec" | "or")
                }
                _ => false,
            };

            if !can_be_arg {
                return Ok(func);
            }

            let arg = self.parse_select()?;
            func = Expr::Apply {
                func: Box::new(func),
                arg: Box::new(arg),
            };
        }
    }

    fn parse_select(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_atom()?;

        if self.peek().kind == TokenKind::Dot {
            self.advance();
            let mut path = vec![self.attr_name()?];
            while self.peek().kind == TokenKind::Dot {
                self.advance();
                path.push(self.attr_name()?);
            }

            if self.peek().kind == TokenKind::Question {
                self.advance();
                let attr = vec![self.attr_name()?];
                return Ok(Expr::HasAttr {
                    expr: Box::new(expr),
                    attr,
                });
            }

            return Ok(Expr::AttrPath(path));
        }

        if self.peek().kind == TokenKind::Question {
            self.advance();
            let attr = vec![self.attr_name()?];
            return Ok(Expr::HasAttr {
                expr: Box::new(expr),
                attr,
            });
        }

        Ok(expr)
    }

    fn attr_name(&mut self) -> Result<String, ParseError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::Ident | TokenKind::Keyword | TokenKind::Str | TokenKind::Path => {
                Ok(tok.text)
            }
            _ => Err(self.unexpected(&tok, "attribute name")),
        }
    }

    fn parse_attr_path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut path = vec![self.attr_name()?];
        while self.peek().kind == TokenKind::Dot {
            self.advance();
            path.push(self.attr_name()?);
        }
        Ok(path)
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Int => {
                self.advance();
                Ok(Expr::Int(tok.text.parse().unwrap_or(0)))
            }
            TokenKind::Float => {
                self.advance();
                Ok(Expr::Float(tok.text.parse().unwrap_or(0.0)))
            }
            TokenKind::Str => {
                self.advance();
                Ok(Expr::Str(tok.text.clone()))
            }
            TokenKind::Path | TokenKind::SearchPath => {
                self.advance();
                Ok(Expr::Path(tok.text.clone()))
            }
            TokenKind::Ident => {
                self.advance();
                if self.peek().kind == TokenKind::Colon {
                    self.advance();
                    let body = self.parse_expr()?;
                    return Ok(Expr::Lambda {
                        param: tok.text.clone(),
                        body: Box::new(body),
                    });
                }
                Ok(Expr::Ident(tok.text.clone()))
            }
            TokenKind::Keyword => {
                match tok.text.as_str() {
                    "true" => { self.advance(); Ok(Expr::Bool(true)) }
                    "false" => { self.advance(); Ok(Expr::Bool(false)) }
                    "null" => { self.advance(); Ok(Expr::Ident("null".into())) }
                    "let" => self.parse_let(),
                    "if" => self.parse_if(),
                    "with" => self.parse_with(),
                    "assert" => self.parse_assert(),
                    "inherit" => self.parse_inherit(),
                    "rec" => {
                        self.advance();
                        self.parse_attrs()
                    }
                    "or" => { self.advance(); Ok(Expr::Ident("or".into())) }
                    _ => Err(self.unexpected(&tok, "expression")),
                }
            }
            TokenKind::LBrace => {
                let save_pos = self.pos;
                self.advance();
                let mut scan_pos = self.pos;
                let mut brace_depth = 1;
                while scan_pos < self.tokens.len() && brace_depth > 0 {
                    match self.tokens[scan_pos].kind {
                        TokenKind::LBrace => brace_depth += 1,
                        TokenKind::RBrace => brace_depth -= 1,
                        _ => {}
                    }
                    scan_pos += 1;
                }
                let is_lambda = scan_pos < self.tokens.len()
                    && self.tokens[scan_pos].kind == TokenKind::Colon;
                self.pos = save_pos;

                if is_lambda {
                    self.parse_lambda_pattern()
                } else {
                    self.parse_attrs()
                }
            }
            TokenKind::LBracket => self.parse_list(),
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen, "')'")?;
                Ok(expr)
            }
            _ => Err(self.unexpected(&tok, "expression")),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Keyword, "'let'")?;
        let bindings = self.parse_bindings()?;
        self.expect(TokenKind::Keyword, "'in'")?;
        let body = self.parse_expr()?;
        Ok(Expr::Let {
            bindings,
            body: Box::new(body),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Keyword, "'if'")?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::Keyword, "'then'")?;
        let then_branch = self.parse_expr()?;
        self.expect(TokenKind::Keyword, "'else'")?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_with(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Keyword, "'with'")?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semi, "';'")?;
        let body = self.parse_expr()?;
        Ok(Expr::With {
            expr: Box::new(expr),
            body: Box::new(body),
        })
    }

    fn parse_assert(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Keyword, "'assert'")?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::Semi, "';'")?;
        let body = self.parse_expr()?;
        Ok(Expr::Assert {
            cond: Box::new(cond),
            body: Box::new(body),
        })
    }

    fn parse_inherit(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::Keyword, "'inherit'")?;

        let from = if self.peek().kind == TokenKind::LParen {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "')'")?;
            Some(Box::new(expr))
        } else {
            None
        };

        let mut names = Vec::new();
        while self.peek().kind == TokenKind::Ident
            || self.peek().kind == TokenKind::Str
            || self.peek().kind == TokenKind::Keyword
        {
            if self.peek().text == "in" || self.peek().kind == TokenKind::Semi {
                break;
            }
            let tok = self.advance().clone();
            if tok.kind == TokenKind::Semi {
                break;
            }
            names.push(tok.text);
        }

        Ok(Expr::Inherit { names, from })
    }

    fn parse_attrs(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LBrace, "'{'")?;
        let bindings = self.parse_bindings()?;
        self.expect(TokenKind::RBrace, "'}'")?;
        Ok(Expr::Attrs(bindings))
    }

    fn parse_lambda_pattern(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LBrace, "'{'")?;
        let mut params = Vec::new();

        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::Eof {
            if self.peek().kind == TokenKind::Ellipsis {
                self.advance();
                break;
            }
            let name = self.attr_name()?;
            params.push(name);
            if self.peek().kind == TokenKind::Comma {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace, "'}'")?;
        self.expect(TokenKind::Colon, "':'")?;
        let body = self.parse_expr()?;

        Ok(Expr::Lambda {
            param: params.join(", "),
            body: Box::new(body),
        })
    }

    fn parse_bindings(&mut self) -> Result<Vec<Binding>, ParseError> {
        let mut bindings = Vec::new();

        loop {
            let tok = self.peek().clone();
            match tok.kind {
                TokenKind::RBrace | TokenKind::Eof | TokenKind::Keyword => {
                    if tok.kind == TokenKind::Keyword
                        && (tok.text == "inherit")
                    {
                        let start = tok.span.start;
                        let expr = self.parse_inherit()?;
                        self.expect(TokenKind::Semi, "';'")?;
                        bindings.push(Binding {
                            path: Vec::new(),
                            value: Some(expr),
                            inherit: true,
                            span: SourceSpan::new(start, self.peek().span.start),
                        });
                        continue;
                    }
                    break;
                }
                TokenKind::Ident | TokenKind::Str | TokenKind::Path | TokenKind::LBracket => {
                    let start = tok.span.start;
                    if tok.kind == TokenKind::LBracket {
                        self.advance();
                        let mut dyn_parts = Vec::new();
                        while self.peek().kind != TokenKind::RBracket {
                            let part = self.parse_expr()?;
                            if let Expr::Str(s) = part {
                                dyn_parts.push(s);
                            } else if let Expr::Ident(s) = part {
                                dyn_parts.push(s);
                            } else {
                                dyn_parts.push("?".to_string());
                            }
                        }
                        self.expect(TokenKind::RBracket, "']'")?;
                        let mut path = dyn_parts;
                        while self.peek().kind == TokenKind::Dot {
                            self.advance();
                            path.push(self.attr_name()?);
                        }
                        self.expect(TokenKind::Assign, "'='")?;
                        let value = self.parse_expr()?;
                        self.expect(TokenKind::Semi, "';'")?;
                        bindings.push(Binding {
                            path,
                            value: Some(value),
                            inherit: false,
                            span: SourceSpan::new(start, self.peek().span.start),
                        });
                    } else {
                        let path = self.parse_attr_path()?;
                        self.expect(TokenKind::Assign, "'='")?;
                        let value = self.parse_expr()?;
                        self.expect(TokenKind::Semi, "';'")?;
                        bindings.push(Binding {
                            path,
                            value: Some(value),
                            inherit: false,
                            span: SourceSpan::new(start, self.peek().span.start),
                        });
                    }
                }
                _ => {
                    break;
                }
            }
        }

        Ok(bindings)
    }

    fn parse_list(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LBracket, "'['")?;
        let mut items = Vec::new();
        while self.peek().kind != TokenKind::RBracket && self.peek().kind != TokenKind::Eof {
            items.push(self.parse_select()?);
        }
        self.expect(TokenKind::RBracket, "']'")?;
        Ok(Expr::List(items))
    }

    pub fn parse(&mut self) -> Result<NixFile, ParseError> {
        let start = self.peek().span.start;
        let expr = self.parse_expr()?;
        let end = self.peek().span.end;
        let imports = collect_imports(&expr);
        Ok(NixFile {
            expr,
            span: SourceSpan::new(start, end),
            imports,
        })
    }
}

pub fn collect_imports(expr: &Expr) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    walk_imports(expr, &mut imports);
    imports
}

fn walk_imports(expr: &Expr, imports: &mut Vec<ImportRef>) {
    match expr {
        Expr::Apply { func, arg } => {
            if let Expr::Ident(name) = func.as_ref() {
                if name == "import" {
                    if let Expr::Path(p) = arg.as_ref() {
                        imports.push(ImportRef::new(
                            p.clone(),
                            SourceSpan::new(
                                avalanche_model::source::Position { line: 0, column: 0 },
                                avalanche_model::source::Position { line: 0, column: 0 },
                            ),
                        ));
                    }
                }
            }
            walk_imports(func, imports);
            walk_imports(arg, imports);
        }
        Expr::Attrs(bindings) => {
            for b in bindings {
                if let Some(v) = &b.value {
                    let is_imports_attr = b.path.last().map(|p| p == "imports").unwrap_or(false);
                    if is_imports_attr {
                        collect_paths_from_expr(v, imports);
                    } else {
                        walk_imports(v, imports);
                    }
                }
            }
        }
        Expr::Let { bindings, body } => {
            for b in bindings {
                if let Some(v) = &b.value {
                    walk_imports(v, imports);
                }
            }
            walk_imports(body, imports);
        }
        Expr::List(items) => {
            for item in items {
                walk_imports(item, imports);
            }
        }
        Expr::If { cond, then_branch, else_branch } => {
            walk_imports(cond, imports);
            walk_imports(then_branch, imports);
            walk_imports(else_branch, imports);
        }
        Expr::Lambda { body, .. } => {
            walk_imports(body, imports);
        }
        Expr::BinaryOp { left, right, .. } => {
            walk_imports(left, imports);
            walk_imports(right, imports);
        }
        Expr::With { expr, body } | Expr::Assert { cond: expr, body } => {
            walk_imports(expr, imports);
            walk_imports(body, imports);
        }
        Expr::Negate(inner) => {
            walk_imports(inner, imports);
        }
        _ => {}
    }
}

fn collect_paths_from_expr(expr: &Expr, imports: &mut Vec<ImportRef>) {
    match expr {
        Expr::Path(p) => {
            imports.push(ImportRef::new(
                p.clone(),
                SourceSpan::new(
                    avalanche_model::source::Position { line: 0, column: 0 },
                    avalanche_model::source::Position { line: 0, column: 0 },
                ),
            ));
        }
        Expr::List(items) => {
            for item in items {
                collect_paths_from_expr(item, imports);
            }
        }
        Expr::Apply { func, arg } => {
            if let Expr::Ident(name) = func.as_ref() {
                if name == "import" {
                    if let Expr::Path(p) = arg.as_ref() {
                        imports.push(ImportRef::new(
                            p.clone(),
                            SourceSpan::new(
                                avalanche_model::source::Position { line: 0, column: 0 },
                                avalanche_model::source::Position { line: 0, column: 0 },
                            ),
                        ));
                        return;
                    }
                }
            }
            collect_paths_from_expr(func, imports);
            collect_paths_from_expr(arg, imports);
        }
        _ => {}
    }
}

pub fn parse(input: &str) -> Result<NixFile, ParseError> {
    let tokens = tokenize(input).map_err(|e| ParseError::Lex(e.to_string()))?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_attrs() {
        let file = parse("{ enable = true; count = 42; }").unwrap();
        match &file.expr {
            Expr::Attrs(bindings) => {
                assert_eq!(bindings.len(), 2);
                assert_eq!(bindings[0].path, vec!["enable"]);
                assert_eq!(bindings[0].value, Some(Expr::Bool(true)));
                assert_eq!(bindings[1].path, vec!["count"]);
                assert_eq!(bindings[1].value, Some(Expr::Int(42)));
            }
            other => panic!("expected attrs, got {:?}", other),
        }
    }

    #[test]
    fn parse_nested_attr_path() {
        let file = parse("{ services.pipewire.enable = true; }").unwrap();
        match &file.expr {
            Expr::Attrs(bindings) => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].path, vec!["services", "pipewire", "enable"]);
                assert_eq!(bindings[0].value, Some(Expr::Bool(true)));
            }
            other => panic!("expected attrs, got {:?}", other),
        }
    }

    #[test]
    fn parse_let_in() {
        let file = parse("let x = 1; in x").unwrap();
        match &file.expr {
            Expr::Let { bindings, body } => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].path, vec!["x"]);
                assert_eq!(**body, Expr::Ident("x".into()));
            }
            other => panic!("expected let, got {:?}", other),
        }
    }

    #[test]
    fn parse_if_then_else() {
        let file = parse("if true then 1 else 2").unwrap();
        match &file.expr {
            Expr::If { .. } => {}
            other => panic!("expected if, got {:?}", other),
        }
    }

    #[test]
    fn parse_lambda() {
        let file = parse("{ pkgs }: pkgs.hello").unwrap();
        match &file.expr {
            Expr::Lambda { .. } => {}
            other => panic!("expected lambda, got {:?}", other),
        }
    }

    #[test]
    fn parse_list() {
        let file = parse("[ 1 2 3 ]").unwrap();
        match &file.expr {
            Expr::List(items) => assert_eq!(items.len(), 3),
            other => panic!("expected list, got {:?}", other),
        }
    }

    #[test]
    fn parse_string_value() {
        let file = parse(r#"{ name = "hello"; }"#).unwrap();
        match &file.expr {
            Expr::Attrs(bindings) => {
                assert_eq!(bindings[0].value, Some(Expr::Str("hello".into())));
            }
            other => panic!("expected attrs, got {:?}", other),
        }
    }

    #[test]
    fn parse_import_detection() {
        let file = parse("{ imports = [ ./foo.nix ./bar.nix ]; }").unwrap();
        assert_eq!(file.imports.len(), 2);
        assert_eq!(file.imports[0].path, "./foo.nix");
        assert_eq!(file.imports[1].path, "./bar.nix");
    }

    #[test]
    fn parse_binary_ops() {
        let file = parse("1 + 2 * 3").unwrap();
        match &file.expr {
            Expr::BinaryOp { op, .. } => assert_eq!(*op, BinOp::Add),
            other => panic!("expected binary op, got {:?}", other),
        }
    }

    #[test]
    fn parse_function_application() {
        let file = parse("import ./foo.nix").unwrap();
        match &file.expr {
            Expr::Apply { func, arg } => {
                assert_eq!(**func, Expr::Ident("import".into()));
                assert_eq!(**arg, Expr::Path("./foo.nix".into()));
            }
            other => panic!("expected apply, got {:?}", other),
        }
    }

    #[test]
    fn parse_with_expr() {
        let file = parse("with pkgs; [ hello ]").unwrap();
        match &file.expr {
            Expr::With { expr, body } => {
                assert_eq!(**expr, Expr::Ident("pkgs".into()));
                assert!(matches!(**body, Expr::List(_)));
            }
            other => panic!("expected with, got {:?}", other),
        }
    }

    #[test]
    fn parse_has_attr() {
        let file = parse("attrs ? foo").unwrap();
        match &file.expr {
            Expr::HasAttr { attr, .. } => assert_eq!(attr, &["foo".to_string()]),
            other => panic!("expected has-attr, got {:?}", other),
        }
    }

    #[test]
    fn parse_inherit() {
        let file = parse("{ inherit (pkgs) hello; }").unwrap();
        match &file.expr {
            Expr::Attrs(bindings) => {
                assert_eq!(bindings.len(), 1);
                assert!(bindings[0].inherit);
            }
            other => panic!("expected attrs, got {:?}", other),
        }
    }

    #[test]
    fn parse_search_path() {
        let file = parse("<nixpkgs>").unwrap();
        assert_eq!(file.expr, Expr::Path("<nixpkgs>".into()));
    }

    #[test]
    fn parse_negative_number() {
        let file = parse("-42").unwrap();
        match &file.expr {
            Expr::Negate(inner) => assert_eq!(**inner, Expr::Int(42)),
            Expr::Int(-42) => {}
            other => panic!("expected negate or negative int, got {:?}", other),
        }
    }

    #[test]
    fn parse_flake_outputs_pattern() {
        let input = r#"{
  outputs = { self, nixpkgs }: {
    packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;
  };
}"#;
        let file = parse(input);
        assert!(file.is_ok());
    }

    #[test]
    fn parse_error_on_invalid() {
        let result = parse("{ = }");
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_attrs() {
        let file = parse("{}").unwrap();
        match &file.expr {
            Expr::Attrs(bindings) => assert!(bindings.is_empty()),
            other => panic!("expected empty attrs, got {:?}", other),
        }
    }

    #[test]
    fn parse_nested_attrs() {
        let file = parse("{ a = { b = { c = 1; }; }; }").unwrap();
        match &file.expr {
            Expr::Attrs(bindings) => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].path, vec!["a"]);
                if let Some(Expr::Attrs(inner)) = &bindings[0].value {
                    assert_eq!(inner[0].path, vec!["b"]);
                } else {
                    panic!("expected nested attrs");
                }
            }
            other => panic!("expected attrs, got {:?}", other),
        }
    }
}
