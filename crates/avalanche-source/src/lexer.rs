use avalanche_model::source::{Position, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("unexpected character '{ch}' at line {line}, column {col}")]
    UnexpectedChar { ch: char, line: u32, col: u32 },
    #[error("unterminated string at line {line}, column {col}")]
    UnterminatedString { line: u32, col: u32 },
    #[error("unterminated comment at line {line}, column {col}")]
    UnterminatedComment { line: u32, col: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Int,
    Float,
    Str,
    Path,
    Ident,
    Keyword,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Semi,
    Colon,
    Comma,
    Dot,
    Ellipsis,
    Assign,
    At,
    Question,
    Plus,
    Sub,
    Mul,
    Div,
    Concat,
    Update,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Imply,
    HasAttr,
    SearchPath,
    Interpolation,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(kind: TokenKind, text: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            kind,
            text: text.into(),
            span,
        }
    }

    pub fn is_keyword(&self, kw: &str) -> bool {
        self.kind == TokenKind::Keyword && self.text == kw
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn source(&self) -> &str {
        self.input
    }

    fn current_pos(&self) -> Position {
        Position {
            line: self.line,
            column: self.col,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('#') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('*') => {
                    let start = self.current_pos();
                    self.advance();
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 {
                        match self.advance() {
                            Some('/') if self.peek() == Some('*') => {
                                self.advance();
                                depth += 1;
                            }
                            Some('*') if self.peek() == Some('/') => {
                                self.advance();
                                depth -= 1;
                            }
                            Some(_) => {}
                            None => {
                                return Err(LexError::UnterminatedComment {
                                    line: start.line,
                                    col: start.column,
                                })
                            }
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn read_string(&mut self) -> Result<String, LexError> {
        let start = self.current_pos();
        self.advance(); // opening quote
        let mut result = String::new();
        let mut escape_next = false;

        loop {
            match self.advance() {
                None => {
                    return Err(LexError::UnterminatedString {
                        line: start.line,
                        col: start.column,
                    })
                }
                Some('\\') if !escape_next => {
                    escape_next = true;
                }
                Some('"') if !escape_next => {
                    break;
                }
                Some(c) if escape_next => {
                    match c {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '$' => result.push('$'),
                        other => {
                            result.push('\\');
                            result.push(other);
                        }
                    }
                    escape_next = false;
                }
                Some(c) => {
                    result.push(c);
                }
            }
        }
        Ok(result)
    }

    fn read_number(&mut self) -> (String, TokenKind) {
        let mut text = String::new();
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else if c == '.' && !is_float && self.peek_ahead(1).map_or(false, |n| n.is_ascii_digit()) {
                is_float = true;
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = if is_float { TokenKind::Float } else { TokenKind::Int };
        (text, kind)
    }

    fn read_path_or_ident(&mut self, first: char) -> (String, TokenKind) {
        let mut text = String::new();
        text.push(first);
        self.advance();

        let is_path_start = first == '.' || first == '/' || first == '~';

        while let Some(c) = self.peek() {
            if is_path_start {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/' || c == '\'' {
                    text.push(c);
                    self.advance();
                } else {
                    break;
                }
            } else {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '\'' {
                    text.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if is_path_start {
            (text, TokenKind::Path)
        } else if text == "true" || text == "false" || text == "null"
            || text == "let" || text == "in" || text == "if" || text == "then"
            || text == "else" || text == "with" || text == "assert" || text == "or"
            || text == "inherit" || text == "rec"
        {
            (text, TokenKind::Keyword)
        } else {
            (text, TokenKind::Ident)
        }
    }

    fn read_search_path(&mut self) -> Result<String, LexError> {
        let start = self.current_pos();
        self.advance(); // '<'
        let mut text = String::from('<');
        loop {
            match self.advance() {
                Some('>') => {
                    text.push('>');
                    return Ok(text);
                }
                Some(c) => text.push(c),
                None => {
                    return Err(LexError::UnterminatedString {
                        line: start.line,
                        col: start.column,
                    })
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;
            let start = self.current_pos();

            let ch = match self.peek() {
                None => {
                    tokens.push(Token::new(TokenKind::Eof, "", SourceSpan::new(start, start)));
                    return Ok(tokens);
                }
                Some(c) => c,
            };

            let token = match ch {
                '{' => { self.advance(); Token::new(TokenKind::LBrace, "{", SourceSpan::new(start, self.current_pos())) }
                '}' => { self.advance(); Token::new(TokenKind::RBrace, "}", SourceSpan::new(start, self.current_pos())) }
                '[' => { self.advance(); Token::new(TokenKind::LBracket, "[", SourceSpan::new(start, self.current_pos())) }
                ']' => { self.advance(); Token::new(TokenKind::RBracket, "]", SourceSpan::new(start, self.current_pos())) }
                '(' => { self.advance(); Token::new(TokenKind::LParen, "(", SourceSpan::new(start, self.current_pos())) }
                ')' => { self.advance(); Token::new(TokenKind::RParen, ")", SourceSpan::new(start, self.current_pos())) }
                ';' => { self.advance(); Token::new(TokenKind::Semi, ";", SourceSpan::new(start, self.current_pos())) }
                ':' => { self.advance(); Token::new(TokenKind::Colon, ":", SourceSpan::new(start, self.current_pos())) }
                ',' => { self.advance(); Token::new(TokenKind::Comma, ",", SourceSpan::new(start, self.current_pos())) }
                '@' => { self.advance(); Token::new(TokenKind::At, "@", SourceSpan::new(start, self.current_pos())) }
                '~' => {
                    let (text, kind) = self.read_path_or_ident(ch);
                    Token::new(kind, text, SourceSpan::new(start, self.current_pos()))
                }
                '/' if self.peek_ahead(1) == Some('/') => {
                    self.advance();
                    self.advance();
                    Token::new(TokenKind::Update, "//", SourceSpan::new(start, self.current_pos()))
                }
                '/' => {
                    let (text, kind) = self.read_path_or_ident(ch);
                    Token::new(kind, text, SourceSpan::new(start, self.current_pos()))
                }
                '"' => {
                    let text = self.read_string()?;
                    Token::new(TokenKind::Str, text, SourceSpan::new(start, self.current_pos()))
                }
                '<' if self.peek_ahead(1).map_or(false, |c| c.is_alphabetic() || c == '/') => {
                    let text = self.read_search_path()?;
                    Token::new(TokenKind::SearchPath, text, SourceSpan::new(start, self.current_pos()))
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::Le, "<=", SourceSpan::new(start, self.current_pos()))
                    } else {
                        Token::new(TokenKind::Lt, "<", SourceSpan::new(start, self.current_pos()))
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::Ge, ">=", SourceSpan::new(start, self.current_pos()))
                    } else {
                        Token::new(TokenKind::Gt, ">", SourceSpan::new(start, self.current_pos()))
                    }
                }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::Eq, "==", SourceSpan::new(start, self.current_pos()))
                    } else {
                        Token::new(TokenKind::Assign, "=", SourceSpan::new(start, self.current_pos()))
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::new(TokenKind::Neq, "!=", SourceSpan::new(start, self.current_pos()))
                    } else {
                        Token::new(TokenKind::Not, "!", SourceSpan::new(start, self.current_pos()))
                    }
                }
                '&' if self.peek_ahead(1) == Some('&') => {
                    self.advance();
                    self.advance();
                    Token::new(TokenKind::And, "&&", SourceSpan::new(start, self.current_pos()))
                }
                '|' if self.peek_ahead(1) == Some('|') => {
                    self.advance();
                    self.advance();
                    Token::new(TokenKind::Or, "||", SourceSpan::new(start, self.current_pos()))
                }
                '+' if self.peek_ahead(1) == Some('+') => {
                    self.advance();
                    self.advance();
                    Token::new(TokenKind::Concat, "++", SourceSpan::new(start, self.current_pos()))
                }
                '+' => {
                    self.advance();
                    Token::new(TokenKind::Plus, "+", SourceSpan::new(start, self.current_pos()))
                }
                '-' => {
                    self.advance();
                    if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                        let (text, kind) = self.read_number();
                        Token::new(kind, format!("-{text}"), SourceSpan::new(start, self.current_pos()))
                    } else {
                        Token::new(TokenKind::Sub, "-", SourceSpan::new(start, self.current_pos()))
                    }
                }
                '*' => {
                    self.advance();
                    Token::new(TokenKind::Mul, "*", SourceSpan::new(start, self.current_pos()))
                }
                '?' => {
                    self.advance();
                    Token::new(TokenKind::Question, "?", SourceSpan::new(start, self.current_pos()))
                }
                '.' if self.peek_ahead(1) == Some('.') && self.peek_ahead(2) == Some('.') => {
                    self.advance();
                    self.advance();
                    self.advance();
                    Token::new(TokenKind::Ellipsis, "...", SourceSpan::new(start, self.current_pos()))
                }
                '.' if self.peek_ahead(1) == Some('/') => {
                    let (text, kind) = self.read_path_or_ident(ch);
                    Token::new(kind, text, SourceSpan::new(start, self.current_pos()))
                }
                '.' if self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) => {
                    self.advance();
                    let mut text = String::from("0.");
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            text.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    Token::new(TokenKind::Float, text, SourceSpan::new(start, self.current_pos()))
                }
                '.' => {
                    self.advance();
                    Token::new(TokenKind::Dot, ".", SourceSpan::new(start, self.current_pos()))
                }
                c if c.is_ascii_digit() => {
                    let (text, kind) = self.read_number();
                    Token::new(kind, text, SourceSpan::new(start, self.current_pos()))
                }
                c if c.is_alphabetic() || c == '_' => {
                    let (text, kind) = self.read_path_or_ident(c);
                    Token::new(kind, text, SourceSpan::new(start, self.current_pos()))
                }
                c => {
                    return Err(LexError::UnexpectedChar {
                        ch: c,
                        line: start.line,
                        col: start.column,
                    })
                }
            };

            tokens.push(token);
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_attrs() {
        let tokens = tokenize("{ enable = true; }").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LBrace);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text, "enable");
        assert_eq!(tokens[2].kind, TokenKind::Assign);
        assert_eq!(tokens[3].kind, TokenKind::Keyword);
        assert_eq!(tokens[3].text, "true");
        assert_eq!(tokens[4].kind, TokenKind::Semi);
        assert_eq!(tokens[5].kind, TokenKind::RBrace);
    }

    #[test]
    fn lex_numbers() {
        let tokens = tokenize("42 3.14").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].text, "42");
        assert_eq!(tokens[1].kind, TokenKind::Float);
        assert_eq!(tokens[1].text, "3.14");
    }

    #[test]
    fn lex_string() {
        let tokens = tokenize(r#""hello world""#).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Str);
        assert_eq!(tokens[0].text, "hello world");
    }

    #[test]
    fn lex_string_escapes() {
        let tokens = tokenize(r#""hello\nworld""#).unwrap();
        assert_eq!(tokens[0].text, "hello\nworld");
    }

    #[test]
    fn lex_path() {
        let tokens = tokenize("./foo.nix").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Path);
        assert_eq!(tokens[0].text, "./foo.nix");
    }

    #[test]
    fn lex_search_path() {
        let tokens = tokenize("<nixpkgs>").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::SearchPath);
        assert_eq!(tokens[0].text, "<nixpkgs>");
    }

    #[test]
    fn lex_operators() {
        let tokens = tokenize("== != < <= > >= && || ++ // ?").unwrap();
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Eq,
                TokenKind::Neq,
                TokenKind::Lt,
                TokenKind::Le,
                TokenKind::Gt,
                TokenKind::Ge,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::Concat,
                TokenKind::Update,
                TokenKind::Question,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lex_keywords() {
        let tokens = tokenize("let in if then else with assert inherit rec").unwrap();
        let keywords: Vec<&str> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Keyword)
            .map(|t| t.text.as_str())
            .collect();
        assert_eq!(
            keywords,
            vec!["let", "in", "if", "then", "else", "with", "assert", "inherit", "rec"]
        );
    }

    #[test]
    fn lex_comments_skipped() {
        let tokens = tokenize("# comment\n42").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].text, "42");
    }

    #[test]
    fn lex_block_comment_skipped() {
        let tokens = tokenize("/* comment */ 42").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int);
    }

    #[test]
    fn lex_unterminated_string() {
        assert!(tokenize(r#""unterminated"#).is_err());
    }

    #[test]
    fn lex_multiline_positions() {
        let tokens = tokenize("{\n  x = 1;\n}").unwrap();
        let x_tok = &tokens[1];
        assert_eq!(x_tok.span.start.line, 2);
        assert_eq!(x_tok.span.start.column, 3);
    }

    #[test]
    fn lex_negative_number() {
        let tokens = tokenize("-5").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].text, "-5");
    }

    #[test]
    fn lex_ident_with_quotes() {
        let tokens = tokenize("foo'").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "foo'");
    }

    #[test]
    fn lex_ellipsis() {
        let tokens = tokenize("...").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ellipsis);
    }
}
