use std::sync::Arc;

use either::Either;

use crate::{lexing::token::Token, utils::FileInfo};

use super::{Expression, TypeName};

#[derive(Debug, Clone)]
pub struct UseStmt
{
    pub use_tok: Token,
    pub ids: Vec<Token>,
    pub semi_colon: Token
}

#[derive(Debug, Clone)]
pub struct ExpressionStmt
{
    pub expression: Expression,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct LetStmt
{
    pub let_tok: Token,
    pub id: Token,
    pub type_name: Option<(Token, TypeName)>, // Token is the colon
    pub equal: Token,
    pub expression: Expression,
    pub semi_colon: Token
}

#[derive(Debug, Clone)]
pub struct ConstStmt
{
    pub pub_tok: Option<Token>,
    pub const_tok: Token,
    pub id: Token,
    pub colon: Token,
    pub type_name: TypeName, // Token is the colon
    pub equal: Token,
    pub expression: Expression,
    pub semi_colon: Token
}

#[derive(Debug, Clone)]
pub struct AssignStmt
{
    pub assigned: Box<Expression>,
    pub equal: Token,
    pub expression: Expression,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct BlockStmt
{
    pub open_brace: Token,
    pub statements: Vec<Statement>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct IfStmt
{
    pub if_tok: Token,
    pub open_paren: Token,
    pub condition: Expression,
    pub close_paren: Token,
    pub block: BlockStmt,
    pub else_branch: Option<ElseBranch>
}

#[derive(Debug, Clone)]
pub struct ElseBranch
{
    pub else_tok: Token,
    pub body: Either<Box<IfStmt>, BlockStmt>,
}

#[derive(Debug, Clone)]
pub struct FnParam
{
    pub id: Token,
    pub colon: Token,
    pub type_name: TypeName,
    pub default_value: Option<(Token, Arc<Expression>)>
}

#[derive(Debug, Clone)]
pub struct FnDecl
{
    pub pub_tok: Option<Token>,
    pub fn_tok: Token,
    pub type_name: Option<(Token, TypeName)>,
    pub id: Token,
    pub open_paren: Token,
    pub self_param: Option<Token>,
    pub params: Vec<FnParam>,
    pub close_paren: Token,
    pub return_type: Option<(Token, TypeName)>,
    pub body: Arc<BlockStmt>, // either has a body or a ';'
}

#[derive(Debug, Clone)]
pub struct StructDeclMember
{
    pub id: Token,
    pub colon: Token,
    pub type_name: TypeName,
    pub initializer: Option<(Token, Arc<Expression>)>,
}

#[derive(Debug, Clone)]
pub struct StructDecl
{
    pub pub_tok: Option<Token>,
    pub struct_tok: Token,
    pub id: Token,
    pub open_brace: Token,
    pub members: Vec<StructDeclMember>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct BreakStmt
{
    pub break_tok: Token,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt
{
    pub continue_tok: Token,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt
{
    pub return_tok: Token,
    pub expression: Option<Expression>,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct ForStmt
{
    pub for_tok: Token,
    pub open_paren: Token,
    pub id_tok: Token,
    pub in_tok: Token,
    pub expression: Expression,
    pub close_paren: Token,
    pub body: Box<BlockStmt>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt
{
    pub while_tok: Token,
    pub open_paren: Token,
    pub condition: Expression,
    pub close_paren: Token,
    pub body: Box<BlockStmt>,
}

#[derive(Debug, Clone)]
pub enum Statement
{
    Expression(ExpressionStmt),
    Let(LetStmt),
    Const(ConstStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Continue(ContinueStmt),
    Break(BreakStmt),
    Assign(AssignStmt),
    If(IfStmt),
    BlockStmt(BlockStmt),
}

#[derive(Debug, Clone)]
pub enum Declaration
{
    Fn(Arc<FnDecl>),
    Struct(Arc<StructDecl>),
    Const(Arc<ConstStmt>),
}

#[derive(Debug)]
pub struct FileNode
{
    pub usings: Vec<UseStmt>,
    pub using_paths: Vec<Vec<String>>,
    pub declarations: Vec<Declaration>,
    pub eof: Token,
    pub info: Arc<FileInfo>,
}

#[derive(Debug)]
pub struct Program
{
    pub files: Vec<Arc<FileNode>>,
}