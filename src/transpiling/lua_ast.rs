use uuid::Uuid;

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

pub enum LuaUnaryOp
{
    Not,
    Len,
    Neg,
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

pub enum LuaExpr
{
    Id(Uuid),
    Literal(LuaLit),
    Binary
    {
        left: Box<Self>,
        op: LuaBinaryOp,
        right: Box<Self>,
    },
    Unary 
    {
        expr: Box<Self>
    },
    Index
    {
        
    }
}