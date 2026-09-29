use avalanche_model::source::{Position, SourceSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Path(String),
    Ident(String),
    Attrs(Vec<Binding>),
    List(Vec<Expr>),
    Let { bindings: Vec<Binding>, body: Box<Expr> },
    If { cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr> },
    Apply { func: Box<Expr>, arg: Box<Expr> },
    Lambda { param: String, body: Box<Expr> },
    AttrPath(Vec<String>),
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    HasAttr { expr: Box<Expr>, attr: Vec<String> },
    Negate(Box<Expr>),
    With { expr: Box<Expr>, body: Box<Expr> },
    Assert { cond: Box<Expr>, body: Box<Expr> },
    Inherit { names: Vec<String>, from: Option<Box<Expr>> },
}

impl Expr {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expr::Int(_) | Expr::Float(_) | Expr::Str(_) | Expr::Bool(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
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
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Concat => "++",
            BinOp::Update => "//",
            BinOp::Eq => "==",
            BinOp::Neq => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub path: Vec<String>,
    pub value: Option<Expr>,
    pub inherit: bool,
    pub span: SourceSpan,
}

impl Binding {
    pub fn dot_path(&self) -> String {
        self.path.join(".")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpannedExpr {
    pub expr: Expr,
    pub span: SourceSpan,
}

impl SpannedExpr {
    pub fn new(expr: Expr, span: SourceSpan) -> Self {
        Self { expr, span }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixFile {
    pub expr: Expr,
    pub span: SourceSpan,
    pub imports: Vec<ImportRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRef {
    pub path: String,
    pub span: SourceSpan,
}

impl ImportRef {
    pub fn new(path: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            path: path.into(),
            span,
        }
    }

    pub fn is_relative(&self) -> bool {
        self.path.starts_with('.')
    }

    pub fn is_absolute(&self) -> bool {
        self.path.starts_with('/')
    }

    pub fn is_angle_bracket(&self) -> bool {
        self.path.starts_with('<') && self.path.ends_with('>')
    }
}

pub fn position(line: u32, column: u32) -> Position {
    Position { line, column }
}

pub fn span(start: Position, end: Position) -> SourceSpan {
    SourceSpan::new(start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_dot_path() {
        let b = Binding {
            path: vec!["services".into(), "pipewire".into(), "enable".into()],
            value: Some(Expr::Bool(true)),
            inherit: false,
            span: span(position(1, 1), position(1, 30)),
        };
        assert_eq!(b.dot_path(), "services.pipewire.enable");
    }

    #[test]
    fn expr_is_literal() {
        assert!(Expr::Bool(true).is_literal());
        assert!(Expr::Int(42).is_literal());
        assert!(Expr::Str("hello".into()).is_literal());
        assert!(Expr::Float(3.14).is_literal());
        assert!(!Expr::Ident("foo".into()).is_literal());
    }

    #[test]
    fn import_ref_classification() {
        let rel = ImportRef::new("./foo.nix", span(position(1, 1), position(1, 10)));
        assert!(rel.is_relative());
        assert!(!rel.is_absolute());

        let abs = ImportRef::new("/etc/nixos/foo.nix", span(position(1, 1), position(1, 18)));
        assert!(abs.is_absolute());

        let angle = ImportRef::new("<nixpkgs>", span(position(1, 1), position(1, 9)));
        assert!(angle.is_angle_bracket());
    }

    #[test]
    fn bin_op_strings() {
        assert_eq!(BinOp::Add.as_str(), "+");
        assert_eq!(BinOp::Concat.as_str(), "++");
        assert_eq!(BinOp::Update.as_str(), "//");
        assert_eq!(BinOp::And.as_str(), "&&");
        assert_eq!(BinOp::Or.as_str(), "||");
        assert_eq!(BinOp::Eq.as_str(), "==");
    }
}
