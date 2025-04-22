use uuid::Uuid;

use crate::{lexing::token::TokenValue, parsing::ast::{BinaryExpr, Expression, FileNode, UnaryExpr}};

use super::{builtins::{BOOL_TYPE, FLOAT_TYPE, INT_TYPE, STRING_TYPE}, operators::{BinaryOpType, GlobalOperators, UnaryOpType}, TypeError, TypeInfo, TypeLibrary};

pub enum TypedExpression
{
    Literal
    {
        returned: TypeInfo,
    },
    Binary 
    {
        left: Box<TypedExpression>,
        op: BinaryOpType,
        right: Box<TypedExpression>,
        returned: TypeInfo,
    },
    Unary 
    {
        operand: Box<TypedExpression>,
        op: UnaryOpType,
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
        returned: TypeInfo,
    },
}

#[derive(Clone, Copy)]
pub struct ExprCheckArgs<'a>
{
    pub operators: &'a GlobalOperators,
    pub library: &'a TypeLibrary,
    pub file: &'a FileNode,
}

impl TypedExpression 
{
    pub fn check_expr(expr: &Expression, args: ExprCheckArgs) -> Result<TypedExpression, TypeError>
    {
        match expr 
        {
            Expression::Literal(token) => {
                let returned = match token.value.as_ref().unwrap()
                {
                    TokenValue::String(_) => STRING_TYPE.clone(),
                    TokenValue::Int(_) => INT_TYPE.clone(),
                    TokenValue::Float(_) => FLOAT_TYPE.clone(),
                    TokenValue::Bool(_) => BOOL_TYPE.clone(),
                };

                Ok(TypedExpression::Literal { returned })
            },
            Expression::Binary(BinaryExpr { left, operator, right }) => {
                let left = TypedExpression::check_expr(&left, args)?;
                let right = TypedExpression::check_expr(&right, args)?;
                let op = BinaryOpType::from_token_type(operator.token_type).expect("Unknown binary operator type");

                match args.operators.evaluate_binary(left.returned(), right.returned(), op)
                {
                    Some(returned) => {
                        Ok(TypedExpression::Binary { 
                            left: Box::new(left), 
                            op, 
                            right: Box::new(right), 
                            returned 
                        })
                    },
                    None => {
                        let left_str = left.returned().pretty_print(args.library);
                        let right_str = right.returned().pretty_print(args.library);
                        Err(TypeError::NoBinaryOp(op, left_str, right_str, operator.get_loc(&args.file.info)))
                    }
                }
            },
            Expression::Unary(UnaryExpr { operator, expression }) => {
                let expression = TypedExpression::check_expr(&expression, args)?;
                let op = UnaryOpType::from_token_type(operator.token_type).expect("Unknown unary operator type");

                match args.operators.evaluate_unary(expression.returned(), op)
                {
                    Some(returned) => {
                        Ok(TypedExpression::Unary { 
                            operand: Box::new(expression), 
                            op, 
                            returned 
                        })
                    },
                    None => {
                        let expr_str = expression.returned().pretty_print(args.library);
                        Err(TypeError::NoUnaryOp(op, expr_str, operator.get_loc(&args.file.info)))
                    }
                }
            }
            _ => panic!("This expression has not been implemented yet")
        }
    }

    pub fn returned(&self) -> &TypeInfo
    {
        match self 
        {
            TypedExpression::Literal { returned } => returned,
            TypedExpression::Binary { left: _, op: _, right: _, returned } => returned,
            TypedExpression::Unary { operand: _, op: _, returned } => returned,
            TypedExpression::Call { called: _, args: _, returned } => returned,
            TypedExpression::Index { indexed: _, arg: _, returned } => returned,
            TypedExpression::Access { accessed: _, name: _, returned } => returned,
            TypedExpression::Cast { casted: _, type_info: _, returned } => returned,
            TypedExpression::Construction { type_id: _, args: _, returned } => returned,
        }
    }
}