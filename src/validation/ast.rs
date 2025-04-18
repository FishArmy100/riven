use uuid::Uuid;

use crate::lexing::token::TokenType;

use super::TypeInfo;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp
{
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    And,
    Or,
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Equal,
    NotEqual,
}

impl BinaryOp
{
    pub fn from(token_type: TokenType) -> Option<Self>
    {
        match token_type
        {
            TokenType::Plus         => Some(Self::Plus),
            TokenType::Minus        => Some(Self::Minus),
            TokenType::Multiply     => Some(Self::Multiply),
            TokenType::Divide       => Some(Self::Divide),
            TokenType::Modulus      => Some(Self::Modulus),
            TokenType::AndAnd       => Some(Self::And),
            TokenType::PipePipe     => Some(Self::Or),
            TokenType::GreaterThan  => Some(Self::GreaterThan),
            TokenType::LessThan     => Some(Self::LessThan),
            TokenType::GreaterEqual => Some(Self::GreaterThanEqual),
            TokenType::LessEqual    => Some(Self::LessThanEqual),
            TokenType::EqualEqual   => Some(Self::Equal),
            TokenType::BangEqual    => Some(Self::NotEqual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp
{
    Bang,
    Negate,
}

impl UnaryOp
{
    pub fn from(token_type: TokenType) -> Option<Self>
    {
        match token_type
        {
            TokenType::Bang => Some(Self::Bang),
            TokenType::Minus => Some(Self::Negate),
            _ => None,
        }
    }
}

pub enum TypedExpression
{
    Literal
    {
        returned: TypeInfo,
    },
    Binary 
    {
        left: Box<TypedExpression>,
        op: BinaryOp,
        right: Box<TypedExpression>,
        returned: TypeInfo,
    },
    Unary 
    {
        operand: Box<TypedExpression>,
        op: UnaryOp,
        returned: TypeInfo,
    },
    Call
    {
        called: Box<TypedExpression>,
        args: Vec<TypedExpression>,
        returned: TypeInfo,
    },
    Index 
    {
        indexed: Box<TypedExpression>,
        arg: Box<TypedExpression>,
        returned: TypeInfo,
    },
    Access
    {
        accessed: Box<TypedExpression>,
        name: String,
        returned: TypeInfo,
    },
    Cast
    {
        casted: Box<TypedExpression>,
        type_info: TypeInfo,
        returned: TypeInfo,
    },
    Construction 
    {
        type_id: Uuid,
        args: Vec<(String, TypedExpression)>,
    },
}