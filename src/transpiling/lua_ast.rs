use itertools::Itertools;

#[derive(Debug)]
pub enum LuaBinaryOp
{
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    FloorDiv,

    Eq,
    NoEq,
    Gr,
    Lt,
    GtEq,
    LtEq,

    Concat,

    And,
    Or,
}

impl std::fmt::Display for LuaBinaryOp
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self 
        {
            LuaBinaryOp::Add => write!(f, "+"),
            LuaBinaryOp::Sub => write!(f, "-"),
            LuaBinaryOp::Mul => write!(f, "*"),
            LuaBinaryOp::Div => write!(f, "/"),
            LuaBinaryOp::Mod => write!(f, "%"),
            LuaBinaryOp::Pow => write!(f, "^"),
            LuaBinaryOp::FloorDiv => write!(f, "//"),
            LuaBinaryOp::Eq => write!(f, "=="),
            LuaBinaryOp::NoEq => write!(f, "~="),
            LuaBinaryOp::Gr => write!(f, ">"),
            LuaBinaryOp::Lt => write!(f, "<"),
            LuaBinaryOp::GtEq => write!(f, ">="),
            LuaBinaryOp::LtEq => write!(f, "<="),
            LuaBinaryOp::Concat => write!(f, ".."),
            LuaBinaryOp::And =>write!(f, "and"),
            LuaBinaryOp::Or => write!(f, "or"),
        }
    }
}

#[derive(Debug)]
pub enum LuaUnaryOp
{
    Not,
    Len,
    Neg,
}

impl std::fmt::Display for LuaUnaryOp
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self 
        {
            LuaUnaryOp::Not => write!(f, "not "),
            LuaUnaryOp::Len => write!(f, "#"),
            LuaUnaryOp::Neg => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LuaLit
{
    String(String),
    Number(f64),
    True,
    False,
    Nil,
}

impl std::fmt::Display for LuaLit
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            LuaLit::String(s) => write!(f, "{}", s),
            LuaLit::Number(n) => write!(f, "{}", n),
            LuaLit::True => write!(f, "true"),
            LuaLit::False => write!(f, "false"),
            LuaLit::Nil => write!(f, "nil"),
        }
    }
}

pub struct LuaFormatArgs
{
    pub indent: usize,
    pub tab: String,
}

impl LuaFormatArgs
{
    pub fn get_tab(&self) -> String 
    {
        self.tab.repeat(self.indent)
    }
}

#[derive(Debug)]
pub enum LuaExpr
{
    Id(String),
    Literal(LuaLit),
    Binary
    {
        left: Box<Self>,
        op: LuaBinaryOp,
        right: Box<Self>,
    },
    Unary 
    {
        expr: Box<Self>,
        op: LuaUnaryOp,
    },
    Index
    {
        expr: Box<Self>,
        arg: Box<Self>,
    },
    Call
    {
        expr: Box<Self>,
        args: Vec<Self>,
    },
    FuncDef 
    {
        args: Vec<String>,
        body: Vec<LuaStmt>,
    },
}

impl LuaExpr
{
    pub fn to_string(&self, args: &mut LuaFormatArgs) -> String
    {
        0
    }
}

#[derive(Debug)]
pub enum LuaStmt
{
    Empty,
    Block
    {
        stmts: Vec<LuaStmt>,
    },
    Assign
    {
        pairs: Vec<(String, Box<LuaExpr>)>
    },
    LocalDecl
    {
        
        pairs: Vec<(String, Box<LuaExpr>)>
    },
    While
    {
        cond: Box<LuaExpr>,
        stmts: Vec<LuaStmt>,
    },
    FuncDef
    {
        name: String,
        args: Vec<String>,
        body: Vec<LuaStmt>,
    },
    Label(String),
    Goto
    {
        label: String,
    }
}