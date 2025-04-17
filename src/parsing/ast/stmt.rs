use std::sync::Arc;

use either::Either;

use crate::lexing::token::Token;

use super::{BlockExpr, Expression, GenericParams, IfExpr, LetCondition, Pattern, TypeName};

#[derive(Debug, Clone)]
pub struct UseStmt
{
    pub use_tok: Token,
    pub ids: Vec<Token>,
    pub star: Option<Token>,
    pub semi_colon: Token
}

#[derive(Debug, Clone)]
pub struct ExpressionStmt
{
    pub expression: Expression,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub enum LetBinding
{
    Variable
    {
        mut_tok: Option<Token>,
        id: Token
    },
    Pattern(Pattern)
}

#[derive(Debug, Clone)]
pub struct LetStmt
{
    pub let_tok: Token,
    pub binding: LetBinding,
    pub type_name: Option<(Token, TypeName)>, // Token is the colon
    pub equal: Token,
    pub expression: Expression,
    pub else_clause: Option<(Token, BlockExpr)>, // token is "else"
    pub semi_colon: Token
}

#[derive(Debug, Clone)]
pub struct AssignStmt
{
    pub value: Expression,
    pub equal: Token,
    pub expression: Expression,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct WhereSubClause
{
    pub id: Token,
    pub colon: Token,
    pub types: Vec<TypeName>,
}

#[derive(Debug, Clone)]
pub struct WhereClause
{
    pub where_tok: Token,
    pub sub_clauses: Vec<WhereSubClause>,
}

#[derive(Debug, Clone)]
pub enum FnParam
{
    Normal
    {
        mut_tok: Option<Token>,
        id: Token,
        colon: Token,
        type_name: TypeName,
        default_value: Option<(Token, Expression)>
    },
    SelfParam
    {
        mut_tok: Option<Token>,
        self_tok: Token,
    }
}

#[derive(Debug, Clone)]
pub struct FnDecl
{
    pub fn_tok: Token,
    pub id: Token,
    pub generic_params: Option<GenericParams>,
    pub open_paren: Token,
    pub params: Vec<FnParam>,
    pub close_paren: Token,
    pub arrow: Token,
    pub return_type: TypeName,
    pub where_clause: Option<WhereClause>,
    pub body: Either<BlockExpr, Token>, // either has a body or a ';'
}

#[derive(Debug, Clone)]
pub struct StructMember
{
    pub pub_tok: Option<Token>,
    pub mut_tok: Option<Token>,
    pub id: Token,
    pub colon: Token,
    pub type_name: TypeName,
    pub initializer: Option<(Token, Expression)>,
}

#[derive(Debug, Clone)]
pub struct StructDecl
{
    pub struct_tok: Token,
    pub id: Token,
    pub generic_params: Option<GenericParams>,
    pub where_clause: Option<WhereClause>,
    pub open_brace: Token,
    pub members: Vec<StructMember>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl
{
    pub interface_tok: Token,
    pub id: Token,
    pub generic_params: Option<GenericParams>,
    pub where_clause: Option<WhereClause>,
    pub open_brace: Token,
    pub members: Vec<Statement>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct EnumStructMember
{
    pub mut_tok: Option<Token>,
    pub id: Token,
    pub colon: Token,
    pub type_name: TypeName,
    pub initializer: Option<(Token, Expression)>
}

#[derive(Debug, Clone)]
pub enum EnumMember
{
    Basic(Token),
    Single
    {
        id: Token,
        open_paren: Token,
        type_name: TypeName,
        close_paren: Token
    },
    Struct 
    {
        id: Token,
        open_brace: Token,
        members: Vec<EnumStructMember>,
        close_brace: Token,
    }
}

#[derive(Debug, Clone)]
pub struct EnumDecl
{
    pub enum_tok: Token,
    pub id: Token,
    pub generic_params: Option<GenericParams>,
    pub where_clause: Option<WhereClause>,
    pub open_brace: Token,
    pub members: Vec<EnumMember>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct TypeDecl
{
    pub type_tok: Token,
    pub id: Token,
    pub generic_params: Option<GenericParams>,
    pub equal: Token,
    pub type_name: TypeName,
    pub semi_colon: Token,
}

#[derive(Debug, Clone)]
pub struct ImplStmt
{
    pub impl_tok: Token,
    pub generic_params: Option<GenericParams>,
    pub type_name: TypeName,
    pub for_clause: Option<(Token, TypeName)>,
    pub where_clause: Option<WhereClause>,
    pub open_brace: Token,
    pub members: Vec<(Option<Token>, Statement)>, // Optional pub on each statement
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
    pub pattern: Pattern,
    pub in_tok: Token,
    pub expression: Expression,
    pub body: BlockExpr,
}

#[derive(Debug, Clone)]
pub struct WhileStmt
{
    pub while_tok: Token,
    pub condition: LetCondition,
    pub body: BlockExpr,
}

#[derive(Debug, Clone)]
pub enum Declaration
{
    Fn(Option<Token>, Arc<FnDecl>),
    Struct(Option<Token>, Arc<StructDecl>),
    Interface(Option<Token>, Arc<InterfaceDecl>),
    Enum(Option<Token>, Arc<EnumDecl>),
    Type(Option<Token>, Arc<TypeDecl>),
    Let(Option<Token>, Arc<LetStmt>),
    Use(Option<Token>, Arc<UseStmt>),
    Impl(Arc<ImplStmt>),
}


#[derive(Debug, Clone)]
pub enum Statement
{
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Continue(ContinueStmt),
    Break(BreakStmt),
    TypeDecl(TypeDecl),
    EnumDecl(EnumDecl),
    InterfaceDecl(InterfaceDecl),
    StructDecl(StructDecl),
    FnDecl(FnDecl),
    Let(LetStmt),
    Assign(AssignStmt),
    If(IfExpr),
    Match(MatchExpr),
    Block(BlockExpr),
    Expression(ExpressionStmt),
    Use(UseStmt),
}

#[derive(Debug, Clone)]
pub struct Program
{
    pub declarations: Vec<Declaration>,
    pub eof: Token,
}