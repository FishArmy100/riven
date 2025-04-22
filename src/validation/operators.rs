use std::collections::HashMap;
use crate::lexing::token::TokenType;

use super::type_info::TypeInfo;

pub struct GlobalOperators
{
    binary_ops: HashMap<BinaryOpType, Vec<BinaryOp>>,
    unary_ops: HashMap<UnaryOpType, Vec<UnaryOp>>,
}

impl GlobalOperators
{
    pub fn new() -> Self 
    {
        Self 
        {
            binary_ops: HashMap::new(),
            unary_ops: HashMap::new(),
        }
    }

    pub fn add_binary_op(&mut self, op: BinaryOp)
    {
        let ops = self.binary_ops.entry(op.op).or_default();
        ops.push(op);
    }

    pub fn evaluate_binary(&self, left: &TypeInfo, right: &TypeInfo, op: BinaryOpType) -> Option<TypeInfo>
    {
        let Some(ops) = self.binary_ops.get(&op) else {
            return None;
        };

        ops.iter().find(|o| (o.checker)(left, right)).map(|o| o.result.clone())
    }

    pub fn add_unary_op(&mut self, op: UnaryOp)
    {
        let ops = self.unary_ops.entry(op.op).or_default();
        ops.push(op);
    }

    pub fn evaluate_unary(&self, input: &TypeInfo, op: UnaryOpType) -> Option<TypeInfo>
    {
        let Some(ops) = self.unary_ops.get(&op) else {
            return None;
        };

        ops.iter().find(|o| o.input == *input).map(|o| o.result.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOpType
{
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    And,
    Or,
}

impl BinaryOpType
{
    pub fn from_token_type(tt: TokenType) -> Option<BinaryOpType>
    {
        match tt 
        {
            TokenType::Plus => Some(Self::Plus),
            TokenType::Minus => Some(Self::Minus),
            TokenType::Multiply => Some(Self::Multiply),
            TokenType::Divide => Some(Self::Divide),
            TokenType::Modulus => Some(Self::Modulus),
            TokenType::EqualEqual => Some(Self::Equal),
            TokenType::BangEqual => Some(Self::NotEqual),
            TokenType::GreaterThan => Some(Self::GreaterThan),
            TokenType::GreaterEqual => Some(Self::GreaterThanEqual),
            TokenType::LessThan => Some(Self::LessThan),
            TokenType::LessEqual => Some(Self::LessThanEqual),
            TokenType::AndAnd => Some(Self::And),
            TokenType::PipePipe => Some(Self::Or),
            _ => None
        }
    }
}

impl std::fmt::Display for BinaryOpType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            BinaryOpType::Plus => write!(f, "+"),
            BinaryOpType::Minus => write!(f, "-"),
            BinaryOpType::Multiply => write!(f, "*"),
            BinaryOpType::Divide => write!(f, "/"),
            BinaryOpType::Modulus => write!(f, "%"),
            BinaryOpType::Equal => write!(f, "=="),
            BinaryOpType::NotEqual => write!(f, "!="),
            BinaryOpType::GreaterThan => write!(f, ">"),
            BinaryOpType::LessThan => write!(f, "<"),
            BinaryOpType::GreaterThanEqual => write!(f, ">="),
            BinaryOpType::LessThanEqual => write!(f, "<="),
            BinaryOpType::And => write!(f, "&&"),
            BinaryOpType::Or => write!(f, "||"),
        }
    }
}

pub struct BinaryOp 
{
    pub checker: Box<dyn Fn(&TypeInfo, &TypeInfo) -> bool>,
    pub result: TypeInfo,
    pub op: BinaryOpType,
}

impl BinaryOp
{
    pub fn make_uniform(key: TypeInfo, op: BinaryOpType) -> Self 
    {
        let key_inner = key.clone();
        let checker = Box::new(move |a: &TypeInfo, b: &TypeInfo| {
            *a == key_inner.clone() && *b == key_inner.clone()
        });

        Self 
        {
            checker,
            result: key,
            op,
        }
    }

    pub fn make_isosceles(key: TypeInfo, returned: TypeInfo, op: BinaryOpType) -> Self 
    {
        let mut uniform = Self::make_uniform(key, op);
        uniform.result = returned;
        uniform
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOpType
{
    Negate,
    Invert
}

impl UnaryOpType
{
    pub fn from_token_type(tt: TokenType) -> Option<Self>
    {
        match tt 
        {
            TokenType::Minus => Some(Self::Negate),
            TokenType::Bang => Some(Self::Invert),
            _ => None
        }
    }
}

impl std::fmt::Display for UnaryOpType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            UnaryOpType::Negate => write!(f, "-"),
            UnaryOpType::Invert => write!(f, "!"),
        }
    }
}

pub struct UnaryOp 
{
    pub op: UnaryOpType,
    pub input: TypeInfo,
    pub result: TypeInfo
}