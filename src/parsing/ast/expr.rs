use either::Either;

use crate::{lexing::token::Token, utils::TextPos};

use super::{LetCondition, Pattern, Statement, TypeName};

#[derive(Debug, Clone)]
pub struct LambdaParam
{
    pub name: Token,
    pub colon: Option<Token>,
    pub type_name: Option<TypeName>
}

#[derive(Debug, Clone)]
pub enum LambdaParams
{
    Simple(Token),
    Complex
    {
        open_pipe: Token,
        parameters: Vec<LambdaParam>,
        close_pipe: Token,
        arrow: Option<Token>,
        return_type: Option<TypeName>,
    }
}

#[derive(Debug, Clone)]
pub struct LambdaExpr
{
    pub params: LambdaParams,
    pub arrow: Token,
    pub expression: Box<Expression>
}

#[derive(Debug, Clone)]
pub struct CallExpr
{
    pub expression: Box<Expression>,
    pub open_paren: Token,
    pub args: Vec<Expression>,
    pub close_paren: Token,
}

#[derive(Debug, Clone)]
pub struct IndexExpr
{
    pub expression: Box<Expression>,
    pub open_bracket: Token,
    pub indexer: Box<Expression>,
    pub close_bracket: Token,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr
{
    pub open_paren: Token,
    pub expression: Box<Expression>,
    pub close_paren: Token,
}

#[derive(Debug, Clone)]
pub struct AccessExpr
{
    pub expression: Box<Expression>,
    pub dot: Token,
    pub identifier: Token,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr
{
    pub expression: Box<Expression>,
    pub operator: Token,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr
{
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct BlockExpr
{
    pub open_brace: Token,
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct ConstructionArg
{
    pub name: Token,
    pub colon: Token,
    pub value: Box<Expression>
}

#[derive(Debug, Clone)]
pub struct ConstructionExpr
{
    pub type_name: TypeName,
    pub open_brace: Token,
    pub args: Vec<ConstructionArg>,
    pub close_brace: Token,
}

#[derive(Debug, Clone)]
pub struct TypeValueExpr
{
    pub type_name: TypeName,
    pub dot: Token,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct IfExpr
{
    pub if_tok: Token,
    pub condition: LetCondition,
    pub block: BlockExpr,
    pub else_branch: Option<ElseBranch>
}

#[derive(Debug, Clone)]
pub struct ElseBranch
{
    pub else_tok: Token,
    pub body: Either<Box<IfExpr>, BlockExpr>,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral
{
    pub open_bracket: Token,
    pub expressions: Vec<Expression>,
    pub close_bracket: Token,
}

#[derive(Debug, Clone)]
pub struct CastExpr
{
    pub expression: Box<Expression>,
    pub as_tok: Token,
    pub type_name: TypeName,
}

#[derive(Debug, Clone)]
pub enum Expression
{
    Lambda(LambdaExpr),
    Literal(Token),
    ArrayLiteral(ArrayLiteral),
    Identifier(Token),
    Grouping(GroupingExpr),
    SelfExpr(Token),
    BlockExpr(BlockExpr),
    TypeValue(TypeValueExpr),
    Construction(ConstructionExpr),
    Call(CallExpr),
    Access(AccessExpr),
    Index(IndexExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    IfExpr(IfExpr),
    Cast(CastExpr)
}

impl Expression
{
    pub fn get_pos(&self) -> TextPos
    {
        match self 
        {
            Expression::Lambda(lambda_expr) => {
                match &lambda_expr.params
                {
                    LambdaParams::Simple(token) => {
                        token.pos + lambda_expr.expression.get_pos()
                    },
                    LambdaParams::Complex { open_pipe, parameters: _, close_pipe: _, arrow: _, return_type: _ } => {
                        open_pipe.pos + lambda_expr.expression.get_pos()
                    },
                }
            },
            Expression::Literal(token) => token.pos,
            Expression::ArrayLiteral(lit) => lit.open_bracket.pos + lit.close_bracket.pos,
            Expression::Identifier(token) => token.pos,
            Expression::Grouping(group) => group.open_paren.pos + group.close_paren.pos,
            Expression::SelfExpr(token) => token.pos,
            Expression::BlockExpr(block_expr) => block_expr.open_brace.pos + block_expr.close_brace.pos,
            Expression::TypeValue(expr) => expr.name.pos + expr.type_name.get_pos(),
            Expression::Construction(expr) => expr.type_name.get_pos() + expr.close_brace.pos,
            Expression::EnumConstruction(expr) => expr.type_name.get_pos() + expr.close_paren.pos,
            Expression::Call(call) => call.expression.get_pos() + call.close_paren.pos,
            Expression::Access(access) => access.expression.get_pos() + access.identifier.pos,
            Expression::Index(index) => index.expression.get_pos() + index.close_bracket.pos,
            Expression::Unary(unary) => unary.expression.get_pos() + unary.operator.pos,
            Expression::Binary(binary) => binary.left.get_pos() + binary.right.get_pos(),
            Expression::IfExpr(if_expr) => if_expr.if_tok.pos + if_expr.block.close_brace.pos,
            Expression::MatchExpr(m) => m.expression.get_pos() + m.close_brace.pos,
            Expression::Cast(cast) => cast.expression.get_pos() + cast.type_name.get_pos(),
        }
    }
}