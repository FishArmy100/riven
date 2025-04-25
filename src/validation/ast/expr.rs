use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    lexing::token::{Token, TokenValue}, 
    parsing::ast::{BinaryExpr, CallExpr, ConstructionArg, ConstructionExpr, Expression, FileNode, UnaryExpr}, 
    utils::{FileInfo, TextPos}, 
    validation::{
        builtins::{BOOL_TYPE, FLOAT_TYPE, INT_TYPE, STRING_TYPE}, functions::{FuncLibrary, FuncResolverResult}, info::types::TypeInfo, operators::{BinaryOpType, GlobalOperators, UnaryOpType}, var::VariableStack, TypeError, TypeLibrary
    }
};

#[derive(Debug)]
pub enum TypedIdentifier
{
    Function(Uuid),
    Variable(Uuid),
}

#[derive(Debug)]
pub enum TypedExpression
{
    Literal
    {
        returned: TypeInfo,
    },
    Identifier
    {
        id: TypedIdentifier,
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
    pub type_library: &'a TypeLibrary,
    pub func_library: &'a FuncLibrary,
    pub var_stack: &'a VariableStack,
    pub file: &'a FileNode,
}

impl<'a> ExprCheckArgs<'a>
{
    pub fn resolve_name(&self, token: &Token) -> Result<TypedIdentifier, TypeError>
    {
        if let Some(var) = self.var_stack.resolve_var(token.value_string().unwrap())
        {
            return Ok(TypedIdentifier::Variable(var));
        }
        
        match self.func_library.resolver().resolve(token, self.file)
        {
            FuncResolverResult::ConflictingFunctions(token) => {
                return Err(TypeError::ConflictingFunctions(token.clone(), token.get_loc(self.file)));
            },
            FuncResolverResult::Ok(uuid) => {
                return Ok(TypedIdentifier::Function(uuid));
            },
            _ => {}
        }

        let err = TypeError::UnknownIdentifier(token.value_string().unwrap().clone(), token.get_loc(self.file));
        Err(err)
    }
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
                        let left_str = left.returned().pretty_print(args.type_library);
                        let right_str = right.returned().pretty_print(args.type_library);
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
                        let expr_str = expression.returned().pretty_print(args.type_library);
                        Err(TypeError::NoUnaryOp(op, expr_str, operator.get_loc(&args.file)))
                    }
                }
            },
            Expression::Construction(ConstructionExpr { type_name, open_brace: _, args: con_args, close_brace: _ }) => {
                let resolver = args.type_library.resolver();
                let type_info = TypeInfo::from(type_name, resolver, args.usings, args.file)?;

                let TypeInfo::Primary(id) = type_info else {
                    return Err(TypeError::CannotConstruct(type_info.pretty_print(args.type_library), type_name.get_pos().get_loc(args.file)))
                };

                let def = args.type_library.get_type(&id);
                let args = check_construction_args(def, con_args, args, type_name.get_pos())?;
                
                Ok(TypedExpression::Construction { 
                    type_id: id.clone(), 
                    args, 
                    returned: TypeInfo::Primary(id) 
                })
            },
            Expression::Identifier(id) => {
                let id = args.resolve_name(id)?;

                let returned = match &id {
                    TypedIdentifier::Function(id) => args.func_library.get_func(id).get_type_info(),
                    TypedIdentifier::Variable(id) => args.var_stack.get_var(id).type_info.clone(),
                };

                Ok(TypedExpression::Identifier { id, returned })
            },
            Expression::Call(CallExpr { expression, open_paren, args: call_args, close_paren }) => {
                let expr = TypedExpression::check_expr(&expression, args)?;

                let throw_error = |infos: Vec<TypeInfo>| -> Result<TypedExpression, TypeError> {
                    let arg_names = infos.into_iter().map(|i| i.pretty_print(args.type_library)).collect();
                    let loc = (open_paren.pos + close_paren.pos).get_loc(args.file);
                    return Err(TypeError::InvalidCallArgs(arg_names, loc));
                };

                if let TypedExpression::Identifier { id: TypedIdentifier::Function(id), returned: _ } = &expr {
                    let def = args.func_library.get_func(id);
                    let param_count = def.parameters.len();

                    if call_args.len() > param_count
                    {
                        return throw_error(def.parameters.iter().map(|p| p.type_info.clone()).collect());
                    }

                    let mut checked_args = vec![];
                    for i in 0..param_count
                    {
                        let p = &def.parameters[i];
                        if p.initializer.has_init() && i == call_args.len()
                        {
                            break;
                        }

                        if call_args.len() <= i 
                        {
                            return throw_error(def.parameters.iter().map(|p| p.type_info.clone()).collect());
                        }

                        let a = TypedExpression::check_expr(&call_args[i], args)?;

                        if p.type_info != *a.returned()
                        {
                            return throw_error(def.parameters.iter().map(|p| p.type_info.clone()).collect());
                        }

                        checked_args.push(a);
                    }
                    
                    return Ok(TypedExpression::Call { 
                        called: Box::new(expr), 
                        args: checked_args,
                        returned: def.returned.clone()
                    });
                }

                let TypeInfo::Function { args: fn_args, returned } = &expr.returned() else {
                    return Err(TypeError::ExpectedFunction(expression.get_pos().get_loc(args.file)));
                };

                if call_args.len() != fn_args.len()
                {
                    return throw_error(fn_args.clone());
                }

                let call_args = call_args.iter().map(|c| {
                    TypedExpression::check_expr(c, args)
                }).collect::<Result<Vec<_>, _>>()?;

                if !call_args.iter().zip(fn_args.iter()).all(|(c, f)| {
                    c.returned() == f
                })
                {
                    return throw_error(fn_args.clone());
                }
                
                let returned = (**returned).clone();
                Ok(TypedExpression::Call { 
                    called: Box::new(expr), 
                    args: 
                    call_args, 
                    returned
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
            TypedExpression::Identifier { id: _, returned } => returned,
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

    if !members.values().all(|(mem, init)| *init || mem.initializer.has_init())
    {
        return Err(TypeError::InvalidConstructionArgs(pos.get_loc(args.file)));
    }

    Ok(expressions)
}