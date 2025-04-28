use std::{collections::HashMap, sync::{Arc, Mutex}};

use either::Either::{Left, Right};
use itertools::Itertools;
use uuid::Uuid;

use crate::{
    lexing::token::{Token, TokenType, TokenValue}, 
    parsing::ast::{AccessExpr, ArrayLiteral, BinaryExpr, CallExpr, CastExpr, ConstructionArg, ConstructionExpr, Expression, FileNode, IndexExpr, LambdaExpr, LambdaParams, UnaryExpr}, 
    utils::{FileInfo, Shared, TextLoc, TextPos}, 
    validation::{
        builtins::{BOOL_TYPE, FLOAT_TYPE, INT_TYPE, STRING_TYPE, VOID_TYPE}, defs::var_def::VariableStack, info::{struct_info::StructInfo, types::TypeInfo, FuncResolverResult, InfoContext}, operators::{BinaryOpType, GlobalOperators, UnaryOpType}, TypeError
    }
};

use super::stmt::{StmtCheckArgs, TypedStatement};

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
        loc: TextLoc,
    },
    Array 
    {
        expressions: Vec<TypedExpression>,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Identifier
    {
        id: TypedIdentifier,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Lambda 
    {
        parameters: Vec<(String, TypeInfo)>,
        body: Box<TypedStatement>,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Binary 
    {
        left: Box<TypedExpression>,
        op: BinaryOpType,
        right: Box<TypedExpression>,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Unary 
    {
        operand: Box<TypedExpression>,
        op: UnaryOpType,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Call
    {
        called: Box<TypedExpression>,
        args: Vec<TypedExpression>,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Index 
    {
        indexed: Box<TypedExpression>,
        arg: Box<TypedExpression>,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Access
    {
        accessed: Box<TypedExpression>,
        name: String,
        returned: TypeInfo,
        is_assignable: bool,
        loc: TextLoc,
    },
    TypeAccess
    {
        accessed: Box<TypeInfo>,
        name: String,
        func_id: Uuid,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Cast
    {
        casted: Box<TypedExpression>,
        type_info: TypeInfo,
        returned: TypeInfo,
        loc: TextLoc,
    },
    Construction 
    {
        type_id: Uuid,
        args: Vec<(String, TypedExpression)>,
        returned: TypeInfo,
        loc: TextLoc,
    },
}

#[derive(Clone)]
pub struct ExprCheckArgs<'a>
{
    pub operators: &'a GlobalOperators,
    pub context: &'a InfoContext,
    pub var_stack: Shared<VariableStack>,
    pub file: &'a FileNode,
    pub fn_ret_type: Option<&'a TypeInfo>,
    pub self_type: Option<&'a TypeInfo>,
    pub loop_stack: Shared<u32>,
}

impl<'a> ExprCheckArgs<'a>
{
    pub fn resolve_name(&self, token: &Token) -> Result<TypedIdentifier, TypeError>
    {
        if let Some(var) = self.var_stack.get().resolve_var(token.value_string().unwrap())
        {
            return Ok(TypedIdentifier::Variable(var));
        }
        
        match self.context.func_resolver.resolve(None, token, self.file)
        {
            FuncResolverResult::ConflictingFuncs(token) => {
                return Err(TypeError::ConflictingFunctions(token.clone(), token.get_loc(&self.file.info)));
            },
            FuncResolverResult::Ok(uuid) => {
                return Ok(TypedIdentifier::Function(uuid));
            },
            _ => {}
        }

        let err = TypeError::UnknownIdentifier(token.value_string().unwrap().clone(), token.get_loc(&self.file.info));
        Err(err)
    }

    pub fn to_stmt_args<'c>(&self, return_type: Option<&'c TypeInfo>) -> StmtCheckArgs<'c> where 'a : 'c
    {
        StmtCheckArgs { 
            operators: self.operators, 
            context: self.context, 
            file: self.file, 
            var_stack: self.var_stack.clone(), 
            fn_ret_type: return_type, 
            self_type: self.self_type,
            loop_stack: self.loop_stack.clone(),
        }
    }
}

impl TypedExpression 
{
    pub fn check_expr(expr: &Expression, args: &ExprCheckArgs, expected: Option<&TypeInfo>) -> Result<TypedExpression, TypeError>
    {
        let gotten = match expr 
        {
            Expression::Literal(token) => {
                if let TokenType::Null = token.token_type {
                    let loc = token.get_loc(&args.file.info);
                    return match expected
                    {
                        Some(s) => Ok(TypedExpression::Literal { returned: s.clone(), loc }),
                        None => Err(TypeError::CannotInferExpression(loc))
                    }
                }

                if let TokenType::SelfVal = token.token_type {
                    let loc = token.get_loc(&args.file.info);
                    return match args.self_type
                    {
                        Some(s) => Ok(TypedExpression::Literal { returned: s.clone(), loc }),
                        None => Err(TypeError::CannotInferExpression(loc))
                    }
                }

                let returned = match token.value.as_ref().unwrap()
                {
                    TokenValue::String(_) => STRING_TYPE.clone(),
                    TokenValue::Int(_) => INT_TYPE.clone(),
                    TokenValue::Float(_) => FLOAT_TYPE.clone(),
                    TokenValue::Bool(_) => BOOL_TYPE.clone(),
                };

                Ok(TypedExpression::Literal { 
                    returned, 
                    loc: token.get_loc(&args.file.info)
                })
            },
            Expression::ArrayLiteral(ArrayLiteral { open_bracket, expressions, close_bracket }) => {
                let inner_expected = match expected
                {
                    Some(TypeInfo::Array(inner)) => Some(&**inner),
                    None => None,
                    Some(t) => return Err(TypeError::ExpectedType(t.pretty_print(&args.context.structs), (open_bracket.pos + close_bracket.pos).get_loc(&args.file.info)))
                };

                let checked = expressions.iter()
                    .map(|e| Self::check_expr(e, args, inner_expected))
                    .collect::<Result<Vec<_>, _>>()?;

                for i in 1..checked.len()
                {
                    if checked[i].returned() != checked[i - 1].returned()
                    {
                        return Err(TypeError::ExpectedType(checked[i - 1].returned().pretty_print(&args.context.structs), expressions[i].get_pos().get_loc(&args.file.info)))
                    }
                }

                let loc = (open_bracket.pos + close_bracket.pos).get_loc(&args.file.info);
                let returned = if checked.len() > 0
                {
                    checked[0].returned().clone()
                }
                else if expected.is_some()
                {
                    expected.unwrap().clone()
                }
                else 
                {
                    return Err(TypeError::CannotInferExpression(loc));
                };
                
                let returned = TypeInfo::Array(Box::new(returned));
                Ok(TypedExpression::Array { expressions: checked, returned, loc})
            },
            Expression::Lambda(lambda) => {
                let loc = lambda.get_pos().get_loc(&args.file.info);
                let LambdaExpr { params, arrow: _, body } = lambda;

                let (parameters, ret_type) = match params {
                    LambdaParams::Simple(_) => return Err(TypeError::InvalidLambdaExpressionFormat(loc.clone())),
                    LambdaParams::Empty { pipes: _, return_type } => {
                        let return_type = match return_type {
                            Some((_, r)) => TypeInfo::from(r, &args.context.type_resolver, args.file)?,
                            None => (*VOID_TYPE).clone()
                        };


                        (vec![], return_type)
                    },
                    LambdaParams::Complex { open_pipe: _, parameters, close_pipe: _, arrow: _, return_type } => {
                        let return_type = match return_type {
                            Some(r) => TypeInfo::from(r, &args.context.type_resolver, args.file)?,
                            None => (*VOID_TYPE).clone()
                        };

                        let params = parameters.iter().map(|p| -> Result<(String, TypeInfo), TypeError> {
                            let name = p.name.value_string().unwrap().clone();
                            let Some((_, type_name)) = &p.type_name else {
                                return Err(TypeError::InvalidLambdaExpressionFormat(loc.clone()));
                            };

                            let type_info = TypeInfo::from(type_name, &args.context.type_resolver, args.file)?;
                            Ok((name, type_info))
                        }).collect::<Result<Vec<_>, _>>()?;

                        (params, return_type)
                    },
                };
                
                let Right(block) = body else {
                    return Err(TypeError::InvalidLambdaExpressionFormat(loc));
                };

                args.var_stack.get_mut().push_frame();
                for (name, type_info) in &parameters
                {
                    args.var_stack.get_mut().add_var(name.clone(), type_info.clone(), Uuid::new_v4());
                }

                let body = match TypedStatement::check_block(block, &args.to_stmt_args(Some(&ret_type))) {
                    Ok(ok) => ok,
                    Err(e) => return Err(e[0].clone())
                };

                args.var_stack.get_mut().pop_frame();

                let returned = TypeInfo::Function { args: parameters.iter().map(|p| p.1.clone()).collect(), returned: Box::new(ret_type) };

                Ok(Self::Lambda { 
                    parameters, 
                    body: Box::new(body), 
                    returned, 
                    loc 
                })
            },
            Expression::Grouping(g) => {
                Self::check_expr(&g.expression, args, expected)
            }
            Expression::Binary(BinaryExpr { left, operator, right }) => {
                let checked_left = TypedExpression::check_expr(&left, args, None)?;
                let checked_right = TypedExpression::check_expr(&right, args, None)?;
                let op = BinaryOpType::from_token_type(operator.token_type).expect("Unknown binary operator type");

                match args.operators.evaluate_binary(checked_left.returned(), checked_right.returned(), op)
                {
                    Some(returned) => {

                        Ok(TypedExpression::Binary { 
                            left: Box::new(checked_left), 
                            op, 
                            right: Box::new(checked_right), 
                            returned,
                            loc: (left.get_pos() + right.get_pos()).get_loc(&args.file.info)
                        })
                    },
                    None => {
                        let left_str = checked_left.returned().pretty_print(&args.context.structs);
                        let right_str = checked_right.returned().pretty_print(&args.context.structs);
                        Err(TypeError::NoBinaryOp(op, left_str, right_str, operator.get_loc(&args.file.info)))
                    }
                }
            },
            Expression::Cast(CastExpr { expression, as_tok: _, type_name }) => {

                let loc = (expression.get_pos() + type_name.get_pos()).get_loc(&args.file.info);
                let type_info = TypeInfo::from(type_name, &args.context.type_resolver, args.file)?;
                let expr = Self::check_expr(&expression, args, None)?;

                let Some(returned) = args.operators.evaluate_cast(expr.returned(), &type_info) else {
                    return Err(TypeError::NoCastOperator { 
                        loc, 
                        expr_type: expr.returned().pretty_print(&args.context.structs), 
                        cast_type: type_info.pretty_print(&args.context.structs)
                    });
                };

                Ok(Self::Cast { casted: Box::new(expr), type_info, returned, loc })
            },
            Expression::Index(IndexExpr { expression, open_bracket: _, indexer, close_bracket  }) => {

                let loc = (expression.get_pos() + close_bracket.pos).get_loc(&args.file.info);
                let indexed = Self::check_expr(&expression, args, None)?;
                let arg = Self::check_expr(&indexer, args, None)?;

                let Some(returned) = args.operators.evaluate_index(&indexed.returned(), &arg.returned()) else {
                    return Err(TypeError::NoIndexOperator { 
                        loc, 
                        indexed_type: indexed.returned().pretty_print(&args.context.structs), 
                        arg_type: arg.returned().pretty_print(&args.context.structs)
                    });
                };

                Ok(Self::Index { 
                    indexed: Box::new(indexed), 
                    arg: Box::new(arg), 
                    returned, 
                    loc 
                })
            },
            Expression::Access(AccessExpr { expression, dot: _, identifier }) => {
                let loc = (expression.get_pos() + identifier.pos).get_loc(&args.file.info);

                if let Expression::Identifier(id) = expression.as_ref()
                {
                    let name = id.value_string().unwrap().clone();
                    if !args.var_stack.get().resolve_var(&name).is_some() && 
                       !args.context.func_resolver.resolve(None, id, args.file).is_ok()
                    {
                        let type_info_id = args.context.type_resolver.resolve_result(id, args.file)?;
                        let type_info = TypeInfo::Primary(type_info_id);
                        
                        let func_id = args.context.func_resolver.resolve(Some(type_info.clone()), identifier, args.file).to_result(&args.file.info)?;
                        let func_info = args.context.funcs.get(&func_id).unwrap();

                        return Ok(Self::TypeAccess { 
                            accessed: Box::new(type_info), 
                            name, 
                            func_id, 
                            returned: func_info.get_type_info(), 
                            loc
                        })
                    }
                }

                let expr = TypedExpression::check_expr(&expression, args, None)?;

                let name = identifier.value_string().unwrap().clone();
                
                match eval_access(expr.returned(), &identifier, args)?
                {
                    Some((returned, is_assignable)) => Ok(TypedExpression::Access { 
                        accessed: Box::new(expr), 
                        name, 
                        returned, 
                        is_assignable,
                        loc
                    }), 
                    None => Err(TypeError::NoMember { 
                        type_name: expr.returned().pretty_print(&args.context.structs), 
                        member: name.clone(), 
                        loc
                    })
                }
            },
            Expression::Unary(UnaryExpr { operator, expression }) => {
                let expr = TypedExpression::check_expr(&expression, args, expected)?;
                let op = UnaryOpType::from_token_type(operator.token_type).expect("Unknown unary operator type");

                match args.operators.evaluate_unary(expr.returned(), op)
                {
                    Some(returned) => {

                        Ok(TypedExpression::Unary { 
                            operand: Box::new(expr), 
                            op, 
                            returned,
                            loc: (operator.pos + expression.get_pos()).get_loc(&args.file.info)
                        })
                    },
                    None => {
                        let expr_str = expr.returned().pretty_print(&args.context.structs);
                        Err(TypeError::NoUnaryOp(op, expr_str, operator.get_loc(&args.file.info)))
                    }
                }
            },
            Expression::Construction(ConstructionExpr { type_name, open_brace: _, args: con_args, close_brace }) => { 
                let type_info = TypeInfo::from(type_name, &args.context.type_resolver, args.file)?;

                let TypeInfo::Primary(id) = type_info else {
                    return Err(TypeError::CannotConstruct(type_info.pretty_print(&args.context.structs), type_name.get_pos().get_loc(&args.file.info)))
                };

                let def = args.context.structs.get(&id).unwrap();
                let checked_args = check_construction_args(def, con_args, args, type_name.get_pos())?;
                
                Ok(TypedExpression::Construction { 
                    type_id: id.clone(), 
                    args: checked_args, 
                    returned: TypeInfo::Primary(id),
                    loc: (type_name.get_pos() + close_brace.pos).get_loc(&args.file.info)
                })
            },
            Expression::Identifier(id) => {
                let res_id = args.resolve_name(id)?;

                let returned = match &res_id {
                    TypedIdentifier::Function(id) => args.context.funcs.get(id).unwrap().get_type_info(),
                    TypedIdentifier::Variable(id) => args.var_stack.get().get_var(id).type_info.clone(),
                };
                
                Ok(TypedExpression::Identifier { id: res_id, returned, loc: id.get_loc(&args.file.info) })
            },
            Expression::Call(CallExpr { expression, open_paren, args: call_args, close_paren }) => {
                let loc = (expression.get_pos() + close_paren.pos).get_loc(&args.file.info);
                let expr = TypedExpression::check_expr(&expression, args, None)?;

                let throw_error = |infos: Vec<TypeInfo>| -> Result<TypedExpression, TypeError> {
                    let arg_names = infos.into_iter().map(|i| i.pretty_print(&args.context.structs)).collect();
                    let loc = (open_paren.pos + close_paren.pos).get_loc(&args.file.info);
                    return Err(TypeError::InvalidCallArgs(arg_names, loc));
                };

                if let TypedExpression::Identifier { id: TypedIdentifier::Function(id), returned: _, loc: _ } = &expr {
                    let def = args.context.funcs.get(id).unwrap();
                    let param_count = def.parameters.len();

                    if call_args.len() > param_count
                    {
                        return throw_error(def.parameters.iter().map(|p| p.type_info.clone()).collect());
                    }

                    let mut checked_args = vec![];
                    for i in 0..param_count
                    {
                        let p = &def.parameters[i];
                        if p.init.is_some() && i == call_args.len()
                        {
                            break;
                        }

                        if call_args.len() <= i 
                        {
                            return throw_error(def.parameters.iter().map(|p| p.type_info.clone()).collect());
                        }

                        let a = TypedExpression::check_expr(&call_args[i], args, Some(&p.type_info))?;
                        checked_args.push(a);
                    }
                    
                    return Ok(TypedExpression::Call { 
                        called: Box::new(expr), 
                        args: checked_args,
                        returned: def.returned.clone(),
                        loc
                    });
                }

                let TypeInfo::Function { args: fn_args, returned } = &expr.returned() else {
                    return Err(TypeError::ExpectedFunction(expression.get_pos().get_loc(&args.file.info)));
                };

                if call_args.len() != fn_args.len()
                {
                    return throw_error(fn_args.clone());
                }

                let call_args = call_args.iter().zip(fn_args.iter()).map(|(c, a)| {
                    TypedExpression::check_expr(c, args, Some(a))
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
                    returned,
                    loc
                })
            }
            _ => panic!("This expression has not been implemented yet")
        };

        match gotten {
            Ok(ok) => {
                check_expected(expected, ok.returned(), args, expr.get_pos())?;
                Ok(ok)
            }
            Err(e) => Err(e),
        }
    }

    pub fn is_assignable(&self) -> bool 
    {
        match self 
        {
            Self::Access { accessed: _, name: _, returned: _, loc: _, is_assignable } => *is_assignable,
            Self::Identifier { id, returned: _, loc: _ } => match id {
                TypedIdentifier::Function(_) => false,
                TypedIdentifier::Variable(_) => true,
            }
            Self::Index { indexed: _, arg: _, returned: _, loc: _ } => true,
            _ => false,
        }
    }

    pub fn returned(&self) -> &TypeInfo
    {
        match self 
        {
            TypedExpression::Literal { returned, loc: _ } => returned,
            TypedExpression::Binary { left: _, op: _, right: _, returned, loc: _ } => returned,
            TypedExpression::Unary { operand: _, op: _, returned, loc: _ } => returned,
            TypedExpression::Call { called: _, args: _, returned, loc: _ } => returned,
            TypedExpression::Index { indexed: _, arg: _, returned, loc: _ } => returned,
            TypedExpression::Access { accessed: _, name: _, returned, loc: _, is_assignable: _ } => returned,
            TypedExpression::Cast { casted: _, type_info: _, returned, loc: _ } => returned,
            TypedExpression::Construction { type_id: _, args: _, returned, loc: _ } => returned,
            TypedExpression::Identifier { id: _, returned, loc: _ } => returned,
            TypedExpression::Array { expressions: _, returned, loc: _ } => returned,
            TypedExpression::Lambda { parameters: _, body: _, returned, loc: _ } => returned,
            TypedExpression::TypeAccess { accessed: _, name: _, func_id: _, returned, loc: _ } => returned,
        }
    }

    pub fn loc(&self) -> &TextLoc
    {
        match self 
        {
            TypedExpression::Literal { returned: _, loc } => loc,
            TypedExpression::Binary { left: _, op: _, right: _, returned: _, loc } => loc,
            TypedExpression::Unary { operand: _, op: _, returned: _, loc } => loc,
            TypedExpression::Call { called: _, args: _, returned: _, loc } => loc,
            TypedExpression::Index { indexed: _, arg: _, returned: _, loc } => loc,
            TypedExpression::Access { accessed: _, name: _, returned: _, loc, is_assignable: _ } => loc,
            TypedExpression::Cast { casted: _, type_info: _, returned: _, loc } => loc,
            TypedExpression::Construction { type_id: _, args: _, returned: _, loc } => loc,
            TypedExpression::Identifier { id: _, returned: _, loc } => loc,
            TypedExpression::Array { expressions: _, returned: _, loc } => loc,
            TypedExpression::Lambda { parameters: _, body: _, returned: _, loc } => loc,
            TypedExpression::TypeAccess { accessed: _, name: _, func_id: _, returned: _, loc } => loc,
        }
    }
}

fn check_expected(expected: Option<&TypeInfo>, gotten: &TypeInfo, args: &ExprCheckArgs<'_>, pos: TextPos) -> Result<(), TypeError> 
{
    if expected.is_some_and(|e| e != gotten)
    {
        return Err(TypeError::ExpectedType(expected.unwrap().pretty_print(&args.context.structs), pos.get_loc(&args.file.info)))
    }

    Ok(())
}

fn check_construction_args(info: &StructInfo, con_args: &[ConstructionArg], args: &ExprCheckArgs, pos: TextPos) -> Result<Vec<(String, TypedExpression)>, TypeError>
{
    let members = info.members();
    let mut members = members.iter()
        .map(|(name, type_info)| (name, (type_info, false)))
        .collect::<HashMap<_, _>>();

    let mut expressions = vec![];

    for con_arg in con_args
    {
        let name = con_arg.name.value_string().unwrap();
        let Some((member, was_init)) = members.get_mut(name) else {
            return Err(TypeError::InvalidConstructionArgs(con_arg.name.get_loc(&args.file.info)));
        };

        if *was_init 
        {
            return Err(TypeError::InvalidConstructionArgs(con_arg.name.get_loc(&args.file.info)));
        }

        let expr = TypedExpression::check_expr(&con_arg.value, args, Some(&member.type_info))?;
        if *expr.returned() != member.type_info 
        {
            return Err(TypeError::InvalidConstructionArgs(con_arg.name.get_loc(&args.file.info)));
        }

        *was_init = true;

        expressions.push((name.clone(), expr));
    }

    if !members.values().all(|(mem, init)| *init || mem.has_init)
    {
        return Err(TypeError::InvalidConstructionArgs(pos.get_loc(&args.file.info)));
    }

    Ok(expressions)
}

fn eval_access(accessed: &TypeInfo, name: &Token, args: &ExprCheckArgs) -> Result<Option<(TypeInfo, bool)>, TypeError>
{
    if let FuncResolverResult::Ok(ok) = args.context.func_resolver.resolve(Some(accessed.clone()), name, args.file)
    {
        let func_def = args.context.funcs.get(&ok).unwrap();
        if !func_def.has_self
        {
            return Err(TypeError::CallingStaticFunctionOnNot(name.get_loc(&args.file.info)));
        }
        
        let params = func_def.parameters[1..].iter().map(|p| p.type_info.clone()).collect_vec();
        let ret = Box::new(func_def.returned.clone());
        return Ok(Some((TypeInfo::Function { args: params, returned: ret }, false)));
    }

    let TypeInfo::Primary(id) = accessed else {
        return Ok(None);
    };

    let name = name.value_string().unwrap().clone();

    Ok(args.context.structs.get(id)
        .unwrap()
        .members()
        .get(&name)
        .map(|m| m.type_info.clone()).map(|m| (m, true)))
}