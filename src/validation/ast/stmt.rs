use std::sync::{Arc, Mutex};

use either::Either::{Left, Right};
use uuid::Uuid;
use crate::{lexing::token::TokenType, parsing::ast::{BlockStmt, ElseBranch, FileNode, IfStmt, Statement}, utils::{FileInfo, Shared, TextLoc}, validation::{builtins::{BOOL_TYPE, VOID_TYPE}, defs::var_def::{VarDef, VariableStack}, info::{types::TypeInfo, InfoContext}, operators::GlobalOperators, TypeError}};

use super::{ExprCheckArgs, TypedExpression};

pub struct StmtCheckArgs<'a>
{
    pub operators: &'a GlobalOperators,
    pub context: &'a InfoContext,
    pub file: &'a FileNode,
    pub var_stack: Shared<VariableStack>,
    pub fn_ret_type: Option<&'a TypeInfo>,
    pub self_type: Option<&'a TypeInfo>,
}

impl<'a> StmtCheckArgs<'a>
{
    pub fn expr_check_args(&'a self) -> ExprCheckArgs<'a>
    {
        ExprCheckArgs { 
            operators: self.operators, 
            context: self.context,
            var_stack: self.var_stack.clone(), 
            file: self.file, 
            self_type: self.self_type,
            fn_ret_type: self.fn_ret_type
        }
    }
}

#[derive(Debug)]
pub enum TypedStatement
{
    Let(Uuid), // id of the variable
    Expr(Box<TypedExpression>),
    Assign
    {
        assigned: Box<TypedExpression>,
        expression: Box<TypedExpression>,
    },
    Block(Vec<TypedStatement>),
    If 
    {
        expression: Box<TypedExpression>,
        body: Box<TypedStatement>,
        else_block: Option<Box<TypedStatement>>,
    },
    Break,
    Continue,
    Return
    {
        returned: Option<Box<TypedExpression>>,
        loc: TextLoc,
    },
    For 
    {
        var_id: Uuid,
        condition: Box<TypedExpression>,
        body: Box<TypedStatement>,
    },
    While 
    {
        condition: Box<TypedExpression>,
        body: Box<TypedStatement>,
    }
}

impl TypedStatement
{
    pub fn check_stmt(stmt: &Statement, eval_args: &StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        match stmt
        {
            Statement::Expression(expr) => {
                let args = eval_args.expr_check_args();
                match TypedExpression::check_expr(&expr.expression, &args, None)
                {
                    Ok(ok) => Ok(TypedStatement::Expr(Box::new(ok))),
                    Err(err) => Err(vec![err]),
                }
            },
            Statement::Let(let_stmt) => {
                match VarDef::from_let(let_stmt, &eval_args.expr_check_args())
                {
                    Ok(ok) => {
                        let id = ok.id.clone();
                        eval_args.var_stack.get_mut().add_var_def(ok);
                        Ok(TypedStatement::Let(id))
                    },
                    Err(err) => Err(vec![err]),
                }
            },
            Statement::While(while_stmt) => {
                let mut errors = vec![];
                
                let condition = match TypedExpression::check_expr(&while_stmt.condition, &eval_args.expr_check_args(), Some(&BOOL_TYPE)) {
                    Ok(ok) => Some(ok), 
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                };
                
                if !condition.as_ref().is_some_and(|e| *e.returned() != *BOOL_TYPE)
                {
                    let name = BOOL_TYPE.pretty_print(&eval_args.context.structs);
                    let loc = while_stmt.while_tok.get_loc(&eval_args.file.info);
                    errors.push(TypeError::ExpectedType(name, loc));
                }

                let body = match Self::check_block(&while_stmt.body, eval_args) {
                    Ok(ok) => Some(ok),
                    Err(errs) => {
                        errors.extend(errs);
                        None
                    }
                };
                
                if errors.len() > 0
                {
                    return Err(errors);
                }

                Ok(TypedStatement::While { 
                    condition: Box::new(condition.unwrap()), 
                    body: Box::new(body.unwrap()) 
                })
            },
            Statement::For(for_stmt) => {
                let mut errors = vec![];
                let condition = match TypedExpression::check_expr(&for_stmt.expression, &eval_args.expr_check_args(), None) {
                    Ok(ok) => Some(ok), 
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                };

                // have to check this manually, as we have to pass in None, because the value is generic
                if condition.as_ref().is_some_and(|e| !e.returned().is_iter())
                {
                    let loc = for_stmt.for_tok.get_loc(&eval_args.file.info);
                    errors.push(TypeError::ExpectedType("Expected type Fn() -> ?T".into(), loc));
                }

                let var_name = for_stmt.id_tok.value_string().unwrap().clone();
                let var_type = condition.as_ref().unwrap().returned().get_iter_type().unwrap();

                let var_id = Uuid::new_v4();
                eval_args.var_stack.get_mut().add_var(var_name, var_type, var_id);

                let body = match Self::check_block(&for_stmt.body, eval_args) {
                    Ok(ok) => Some(ok),
                    Err(errs) => {
                        errors.extend(errs);
                        None
                    }
                };
                
                if errors.len() > 0
                {
                    return Err(errors);
                }

                Ok(TypedStatement::For { 
                    condition: Box::new(condition.unwrap()), 
                    body: Box::new(body.unwrap()),
                    var_id
                })
            },
            Statement::Return(return_stmt) => {
                let args = eval_args.expr_check_args();

                let Some(expr) = &return_stmt.expression else {
                    return Ok(TypedStatement::Return{
                        returned: None,
                        loc: (return_stmt.return_tok.pos + return_stmt.semi_colon.pos).get_loc(&args.file.info)
                    })
                };

                match TypedExpression::check_expr(expr, &args, eval_args.fn_ret_type)
                {
                    Ok(ok) => Ok(TypedStatement::Return {
                        returned: Some(Box::new(ok)),
                        loc: (return_stmt.return_tok.pos + return_stmt.semi_colon.pos).get_loc(&args.file.info)
                    }),
                    Err(err) => Err(vec![err]),
                }
            },
            Statement::Continue(_) => {
                return Ok(TypedStatement::Continue);
            },
            Statement::Break(_) => {
                return Ok(TypedStatement::Break)
            },
            Statement::Assign(assign_stmt) => {
                let assigned_expr = match TypedExpression::check_expr(&assign_stmt.assigned, &eval_args.expr_check_args(), None) {
                    Ok(ok) => ok,
                    Err(e) => return Err(vec![e])
                };

                if assign_stmt.equal.token_type != TokenType::Equal
                {
                    panic!("Assignment type not implemented yet, please only assign using `=`");
                }

                if !assigned_expr.is_assignable()
                {
                    return Err(vec![TypeError::ExpressionNotAssignable(assign_stmt.assigned.get_pos().get_loc(&eval_args.file.info))]);
                }

                let expression = match TypedExpression::check_expr(&assign_stmt.expression, &eval_args.expr_check_args(), Some(assigned_expr.returned())) {
                    Ok(ok) => ok,
                    Err(err) => return Err(vec![err])
                };

                Ok(TypedStatement::Assign { 
                    assigned: Box::new(assigned_expr), 
                    expression: Box::new(expression) 
                })
            },
            Statement::If(if_stmt) => Self::check_if(if_stmt, eval_args),
            Statement::BlockStmt(block_stmt) => Self::check_block(block_stmt, eval_args),
            stmt => panic!("Statement type not implemented yet {:?}", stmt)
        }
    }

    fn check_if(stmt: &IfStmt, eval_args: &StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        let mut errors = vec![];
                
        let condition = match TypedExpression::check_expr(&stmt.condition, &eval_args.expr_check_args(), Some(&BOOL_TYPE)) {
            Ok(ok) => Some(ok), 
            Err(err) => {
                errors.push(err);
                None
            }
        };
        
        if condition.as_ref().is_some_and(|e| *e.returned() != *BOOL_TYPE)
        {
            let name = BOOL_TYPE.pretty_print(&eval_args.context.structs);
            let loc = stmt.if_tok.get_loc(&eval_args.file.info);
            errors.push(TypeError::ExpectedType(name, loc));
        }

        let body = match Self::check_block(&stmt.block, eval_args) {
            Ok(ok) => Some(ok),
            Err(errs) => {
                errors.extend(errs);
                None
            }
        };

        let else_block = stmt.else_branch.as_ref().map(|branch| match Self::check_else(branch, eval_args) {
            Ok(s) => Some(Box::new(s)),
            Err(err) => {
                errors.extend(err);
                None
            }
        });
        
        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(TypedStatement::If { 
            expression: Box::new(condition.unwrap()), 
            body: Box::new(body.unwrap()),
            else_block: else_block.unwrap(),
        })
    }

    fn check_else(branch: &ElseBranch, eval_args: &StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        match &branch.body
        {
            Left(l) => Self::check_if(&l, eval_args),
            Right(r) => Self::check_block(r, eval_args)
        }
    }

    pub fn check_block(block: &BlockStmt, args: &StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        let mut errors = vec![];
        let mut statements = vec![];

        args.var_stack.get_mut().push_frame();

        for stmt in &block.statements
        {
            match Self::check_stmt(&stmt, args)
            {
                Ok(ok) => statements.push(ok),
                Err(err) => errors.extend(err),
            }
        }

        args.var_stack.get_mut().pop_frame();

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(Self::Block(statements))    
        }
    }

    pub fn check_return(&self, info: &TypeInfo, args: &StmtCheckArgs) -> Result<bool, Vec<TypeError>>
    {
        match self 
        {
            TypedStatement::Block(typed_statements) => {
                let mut errors = vec![];
                let mut returns = false;
                for stmt in typed_statements
                {
                    match stmt.check_return(info, args) 
                    {
                        Ok(ok) => returns = returns || ok,
                        Err(e) => errors.extend(e),
                    }
                }

                if errors.len() > 0
                {
                    return Err(errors)
                }

                Ok(returns)
            },
            TypedStatement::If { expression: _, body, else_block } => {
                let body_returns = body.check_return(info, args)?;

                let Some(else_block) = else_block else {
                    return Ok(false)
                };

                let else_returns = else_block.check_return(info, args)?;
                Ok(body_returns && else_returns)
            },
            TypedStatement::Return{ returned, loc } => {
                let returned = returned.as_ref().map_or(&*VOID_TYPE, |e| e.returned());
                if returned != info
                {
                    let name = info.pretty_print(&args.context.structs);
                    return Err(vec![TypeError::ExpectedType(name, loc.clone())]);
                }

                Ok(true)
            },
            _ => Ok(false)
        }
    }
}