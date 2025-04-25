use either::Either::{Left, Right};
use uuid::Uuid;
use crate::{parsing::ast::{BlockStmt, ElseBranch, FileNode, IfStmt, Statement}, utils::FileInfo, validation::{builtins::BOOL_TYPE, defs::var_def::{VarDef, VariableStack}, info::InfoContext, operators::GlobalOperators, TypeError}};

use super::{ExprCheckArgs, TypedExpression};

pub struct StmtCheckArgs<'a>
{
    pub operators: &'a GlobalOperators,
    pub context: &'a InfoContext,
    pub file: &'a FileNode,
    pub var_stack: &'a mut VariableStack,
}

impl<'a> StmtCheckArgs<'a>
{
    pub fn expr_check_args(&'a self) -> ExprCheckArgs<'a>
    {
        ExprCheckArgs { 
            operators: self.operators, 
            context: self.context,
            var_stack: &self.var_stack, 
            file: self.file, 
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
        var_id: Uuid, // id of the variable
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
    Return(Option<Box<TypedExpression>>),
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
    pub fn check_stmt(stmt: &Statement, eval_args: &mut StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        match stmt
        {
            Statement::Expression(expr) => {
                let args = eval_args.expr_check_args();
                match TypedExpression::check_expr(&expr.expression, args)
                {
                    Ok(ok) => Ok(TypedStatement::Expr(Box::new(ok))),
                    Err(err) => Err(vec![err]),
                }
            },
            Statement::Let(let_stmt) => {
                match VarDef::from_let(let_stmt, eval_args.expr_check_args())
                {
                    Ok(ok) => {
                        let id = ok.id.clone();
                        eval_args.var_stack.add_var_def(ok);
                        Ok(TypedStatement::Let(id))
                    },
                    Err(err) => Err(vec![err]),
                }
            },
            Statement::While(while_stmt) => {
                let mut errors = vec![];
                
                let condition = match TypedExpression::check_expr(&while_stmt.condition, eval_args.expr_check_args()) {
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
                let condition = match TypedExpression::check_expr(&for_stmt.expression, eval_args.expr_check_args()) {
                    Ok(ok) => Some(ok), 
                    Err(err) => {
                        errors.push(err);
                        None
                    }
                };

                if !condition.as_ref().is_some_and(|e| !e.returned().is_iter())
                {
                    let name = BOOL_TYPE.pretty_print(&eval_args.context.structs);
                    let loc = for_stmt.for_tok.get_loc(&eval_args.file.info);
                    errors.push(TypeError::ExpectedType(name, loc));
                }

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

                let var_name = for_stmt.id_tok.value_string().unwrap().clone();
                let var_type = condition.as_ref().unwrap().returned().get_iter_type().unwrap();

                Ok(TypedStatement::For { 
                    condition: Box::new(condition.unwrap()), 
                    body: Box::new(body.unwrap()),
                    var_id: eval_args.var_stack.add_var(var_name, var_type)
                })
            },
            Statement::Return(return_stmt) => {
                let args = eval_args.expr_check_args();

                let Some(expr) = &return_stmt.expression else {
                    return Ok(TypedStatement::Return(None))
                };

                match TypedExpression::check_expr(expr, args)
                {
                    Ok(ok) => Ok(TypedStatement::Expr(Box::new(ok))),
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
                let var_name = assign_stmt.value.value_string().unwrap().clone();
                let Some(id) = eval_args.var_stack.resolve_var(&var_name) else {
                    return Err(vec![TypeError::UnknownVariable(var_name, assign_stmt.value.get_loc(&eval_args.file.info))]);
                };

                let var_type = &eval_args.var_stack.get_var(&id).type_info;
                let expression = match TypedExpression::check_expr(&assign_stmt.expression, eval_args.expr_check_args()) {
                    Ok(ok) => ok,
                    Err(err) => return Err(vec![err])
                };

                if expression.returned() != var_type
                {
                    let type_name = var_type.pretty_print(&eval_args.context.structs);
                    let loc = assign_stmt.expression.get_pos().get_loc(&eval_args.file.info);
                    return Err(vec![TypeError::ExpectedType(type_name, loc)])
                }

                Ok(TypedStatement::Assign { 
                    var_id: id, 
                    expression: Box::new(expression) 
                })
            },
            Statement::If(if_stmt) => Self::check_if(if_stmt, eval_args),
            Statement::BlockStmt(block_stmt) => Self::check_block(block_stmt, eval_args),
            stmt => panic!("Statement type not implemented yet {:?}", stmt)
        }
    }

    fn check_if(stmt: &IfStmt, eval_args: &mut StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        let mut errors = vec![];
                
        let condition = match TypedExpression::check_expr(&stmt.condition, eval_args.expr_check_args()) {
            Ok(ok) => Some(ok), 
            Err(err) => {
                errors.push(err);
                None
            }
        };
        
        if !condition.as_ref().is_some_and(|e| *e.returned() != *BOOL_TYPE)
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

    fn check_else(branch: &ElseBranch, eval_args: &mut StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        match &branch.body
        {
            Left(l) => Self::check_if(&l, eval_args),
            Right(r) => Self::check_block(r, eval_args)
        }
    }

    pub fn check_block(block: &BlockStmt, args: &mut StmtCheckArgs) -> Result<TypedStatement, Vec<TypeError>>
    {
        let mut errors = vec![];
        let mut statements = vec![];

        args.var_stack.push_frame();

        for stmt in &block.statements
        {
            match Self::check_stmt(&stmt, args)
            {
                Ok(ok) => statements.push(ok),
                Err(err) => errors.extend(err),
            }
        }

        args.var_stack.pop_frame();

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(Self::Block(statements))    
        }
    }
}