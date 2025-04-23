use std::collections::HashMap;

use uuid::Uuid;

use crate::{lexing::token::TokenValue, parsing::ast::{BinaryExpr, ConstructionArg, ConstructionExpr, Expression, FileNode, UnaryExpr}, utils::{FileInfo, TextPos}, validation::{builtins::{BOOL_TYPE, FLOAT_TYPE, INT_TYPE, STRING_TYPE}, operators::{BinaryOpType, GlobalOperators, UnaryOpType}, type_info::TypeInfo, StructDef, TypeError, TypeLibrary}};

#[derive(Debug)]
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
    pub file: &'a FileInfo,
    pub usings: &'a [Vec<String>],
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
                        Err(TypeError::NoBinaryOp(op, left_str, right_str, operator.get_loc(&args.file)))
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
                        Err(TypeError::NoUnaryOp(op, expr_str, operator.get_loc(&args.file)))
                    }
                }
            },
            Expression::Construction(ConstructionExpr { type_name, open_brace: _, args: con_args, close_brace: _ }) => {
                let resolver = args.library.resolver();
                let type_info = TypeInfo::from(type_name, resolver, args.usings, args.file)?;

                let TypeInfo::Primary(id) = type_info else {
                    return Err(TypeError::CannotConstruct(type_info.pretty_print(args.library), type_name.get_pos().get_loc(args.file)))
                };

                let def = args.library.get_type(&id);
                let args = check_construction_args(def, con_args, args, type_name.get_pos())?;
                
                Ok(TypedExpression::Construction { 
                    type_id: id.clone(), 
                    args, 
                    returned: TypeInfo::Primary(id) 
                })
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

fn check_construction_args(def: &StructDef, con_args: &[ConstructionArg], args: ExprCheckArgs, pos: TextPos) -> Result<Vec<(String, TypedExpression)>, TypeError>
{
    let mut members = def.members.iter()
        .map(|(name, type_info)| (name, (type_info, false)))
        .collect::<HashMap<_, _>>();

    let mut expressions = vec![];

    for con_arg in con_args
    {
        let name = con_arg.name.value_string().unwrap();
        let Some((member, was_init)) = members.get_mut(name) else {
            return Err(TypeError::InvalidConstructionArgs(con_arg.name.get_loc(args.file)));
        };

        if *was_init 
        {
            return Err(TypeError::InvalidConstructionArgs(con_arg.name.get_loc(args.file)));
        }

        let expr = TypedExpression::check_expr(&con_arg.value, args)?;
        if *expr.returned() != member.type_info 
        {
            return Err(TypeError::InvalidConstructionArgs(con_arg.name.get_loc(args.file)));
        }

        *was_init = true;

        expressions.push((name.clone(), expr));
    }

    if !members.values().all(|(_, init)| *init)
    {
        return Err(TypeError::InvalidConstructionArgs(pos.get_loc(args.file)));
    }

    Ok(expressions)
}