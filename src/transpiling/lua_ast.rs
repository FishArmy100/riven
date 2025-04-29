use std::fmt::format;

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
    Map
    {
        members: Vec<(String, LuaExpr)>
    }
}

impl LuaExpr
{
    pub fn to_string(&self, format_args: &mut LuaFormatArgs) -> String
    {
        match self 
        {
            LuaExpr::Id(id) => id.clone(),
            LuaExpr::Literal(lua_lit) => lua_lit.to_string(),
            LuaExpr::Binary { left, op, right } => format!("({} {} {})", left.to_string(format_args), op, right.to_string(format_args)),
            LuaExpr::Unary { expr, op } => format!("({}{})", op, expr.to_string(format_args)),
            LuaExpr::Index { expr, arg } => format!("{}[{}]", expr.to_string(format_args), arg.to_string(format_args)),
            LuaExpr::Call { expr, args } => {
                let args = args.iter().map(|a| a.to_string(format_args)).join(", ");
                let expr = expr.to_string(format_args);
                format!("{}({})", expr, args)
            },
            LuaExpr::FuncDef { args, body } => {
                let mut str = format!("function({})\n", args.iter().join(", "));
                format_args.indent += 1;
                for s in body.iter()
                {
                    str += &format!("{}", s.to_string(format_args));
                }
                format_args.indent -= 1;
                str += &format!("{}end", format_args.get_tab());
                str
            },
            LuaExpr::Map { members } => {
                let mut str = "{".to_string();
                format_args.indent += 1;
                for (name, expr) in members.iter()
                {
                    str += &format!("{}{} = {},\n", format_args.get_tab(), name, expr.to_string(format_args));
                }
                format_args.indent -= 1;
                str += &format!("{}}}", format_args.get_tab());
                str
            },
        }
    }

    pub fn to_box(self) -> Box<Self>
    {
        Box::new(self)
    }
}

#[derive(Debug)]
pub enum LuaStmt
{
    Block
    {
        stmts: Vec<LuaStmt>,
    },
    Call 
    {
        expr: Box<LuaExpr>,
        args: Vec<LuaExpr>,
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
    },
    Return(Option<Box<LuaExpr>>),
    Break,
    Spacer,
    Comment(String)
}

impl LuaStmt
{
    pub fn to_string(&self, fmt_args: &mut LuaFormatArgs) -> String 
    {
        match self 
        {
            LuaStmt::Block { stmts } => {
                let mut str = format!("{}do\n", fmt_args.get_tab());
                fmt_args.indent += 1;
                for s in stmts
                {
                    str += &s.to_string(fmt_args);
                }
                fmt_args.indent -= 1;
                str += &format!("{}end\n", fmt_args.get_tab());
                str
            },
            LuaStmt::Assign { pairs } => {
                let ids = pairs.iter().map(|i| i.0.clone()).join(", ");
                let vals = pairs.iter().map(|v| v.1.to_string(fmt_args)).join(", ");
                format!("{}{} = {}\n", fmt_args.get_tab(), ids, vals)
            },
            LuaStmt::LocalDecl { pairs } => {
                let ids = pairs.iter().map(|i| i.0.clone()).join(", ");
                let vals = pairs.iter().map(|v| v.1.to_string(fmt_args)).join(", ");
                format!("{}local {} = {}\n", fmt_args.get_tab(), ids, vals)
            },
            LuaStmt::While { cond, stmts } => {
                let mut str = format!("{}while {}\n", fmt_args.get_tab(), cond.to_string(fmt_args));
                fmt_args.indent += 1;
                for s in stmts
                {
                    str += &s.to_string(fmt_args);
                }
                fmt_args.indent -= 1;
                str += &format!("{}end\n", fmt_args.get_tab());
                str
            },
            LuaStmt::FuncDef { name, args, body } => {
                let mut str = format!("function {}({})\n", name, args.iter().join(", "));
                fmt_args.indent += 1;
                for s in body.iter()
                {
                    str += &s.to_string(fmt_args);
                }
                fmt_args.indent -= 1;
                str += &format!("{}end\n\n", fmt_args.get_tab());
                str
            },
            LuaStmt::Label(l) => format!("{}::{}::\n", fmt_args.get_tab(), l),
            LuaStmt::Goto { label } => format!("{}goto {}\n", fmt_args.get_tab(), label),
            LuaStmt::Return(lua_expr) => match lua_expr {
                Some(expr) => format!("{}return {}\n", fmt_args.get_tab(), expr.to_string(fmt_args)),
                None => format!("{}return\n", fmt_args.get_tab()),
            },
            LuaStmt::Break => format!("{}break\n", fmt_args.get_tab()),
            LuaStmt::Call { expr, args } => {
                let args = args.iter().map(|a| a.to_string(fmt_args)).join(", ");
                let expr = expr.to_string(fmt_args);
                format!("{}{}({})\n", fmt_args.get_tab(), expr, args)
            },
            LuaStmt::Spacer => "\n".into(),
            LuaStmt::Comment(msg) => {
                let mut str = String::new();
                for line in msg.split('\n') 
                {
                    str += &format!("{}-- {}\n", fmt_args.get_tab(), line);
                }

                str
            }
        }
    }
}

pub struct LuaProgram
{
    pub stmts: Vec<LuaStmt>
}

impl LuaProgram
{
    pub fn to_string(&self, tab: String) -> String
    {
        let mut format_args = LuaFormatArgs {
            tab,
            indent: 0,
        };

        let mut str = String::new();
        for s in &self.stmts
        {
            str += &s.to_string(&mut format_args);
        }

        str
    }
}