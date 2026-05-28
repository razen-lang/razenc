use crate::lexer::token::TokenKind;

pub type Span = (usize, usize);

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Use(Vec<String>),
    Mod(String, Option<Vec<Decl>>),
    Fn(FnDecl),
    Struct(StructDecl),
    Union(UnionDecl),
    Enum(EnumDecl),
    Error_(String, Vec<EnumVariant>),
    Behave(BehaveDecl),
    Var(VarDecl),
    Const(ConstDecl),
    TypeAlias(String, Type),
    Test(String, Block),
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub pub_: bool,
    pub external: bool,
    pub attrs: Vec<Annotation>,
    pub params: Vec<Param>,
    pub return_: Option<Type>,
    pub body: Option<Block>,
    pub is_const: bool,
    pub is_variable_fn: bool,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub impl_behave: Option<String>,
    pub pub_: bool,
    pub attrs: Vec<Annotation>,
    pub fields: Vec<Field>,
    pub methods: Vec<FnDecl>,
}

#[derive(Debug, Clone)]
pub struct UnionDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub pub_: bool,
    pub attrs: Vec<Annotation>,
    pub variants: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub impl_behave: Option<String>,
    pub pub_: bool,
    pub attrs: Vec<Annotation>,
    pub variants: Vec<EnumVariant>,
    pub methods: Vec<FnDecl>,
}

#[derive(Debug, Clone)]
pub struct BehaveDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub pub_: bool,
    pub attrs: Vec<Annotation>,
    pub methods: Vec<FnDecl>,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub mutable: bool,
    pub pub_: bool,
    pub attrs: Vec<Annotation>,
    pub type_: Option<Type>,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub attrs: Vec<Annotation>,
    pub type_: Option<Type>,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub mutable: bool,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub pub_: bool,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub type_: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(TokenKind),
    Named(String),
    Ref(bool, Box<Type>),
    Pointer(Box<Type>),
    Optional(Box<Type>),
    ErrorUnion(Option<Box<Type>>, Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Option<Box<Expr>>),
    Fn(Vec<Type>, Box<Type>),
    Builtin(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(TokenKind, String),
    Ident(String),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Field(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Slice(Box<Expr>, Box<Expr>, Box<Expr>, bool),
    StructInit(String, Vec<FieldInit>),
    Deref(Box<Expr>),
    Block(Block),
    Paren(Box<Expr>),
    AtMethod(Box<Expr>, String),
    Catch(Box<Expr>, Vec<String>, Box<Block>),
    Ret(Option<Box<Expr>>),
    Fn(FnDecl),
    MapLiteral(Vec<(Expr, Expr)>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Assign,
    ColonEq,
    Range,
    RangeInclusive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Ref,
    RefMut,
    Optional,
    Deref,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignOp {
    Eq,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    ModEq,
    ColonEq,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Var(VarDecl),
    Ret(Option<Expr>),
    Stop,
    Next,
    If(If),
    Match(Match),
    Loop(Loop),
    Defer(Box<Expr>),
    TryCatch(TryCatch),
    Assign(Expr, AssignOp, Expr),
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expr,
    pub capture: Vec<String>,
    pub then_block: Block,
    pub else_block: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub target: Expr,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub capture: Vec<String>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub conds: Vec<Expr>,
    pub captures: Vec<Capture>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct TryCatch {
    pub try_body: Block,
    pub capture: Vec<String>,
    pub catch_body: Block,
}

#[derive(Debug, Clone)]
pub struct Capture {
    pub name: String,
    pub mutable: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Ident(String),
    Literal(TokenKind, String),
    EnumVariant(String, String, Option<String>),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}
