use std::{borrow::Cow, cell::RefCell, collections::{HashMap, HashSet}, sync::Arc};

use itertools::Itertools;
use lua_ast::*;
use lua_builtins::{append_builtins, INVOKE_EXPR_FUNC};
use mlua::Either::{Left, Right};
use uuid::Uuid;

use crate::validation::{ast::{stmt::TypedStatement, LitVal, TypedExpression, TypedIdentifier}, builtins::{self, IS_NONE_ID, STRING_TYPE, UNWRAP_ID}, defs::{func_def::FuncDef, struct_def::StructDef, var_def::VarDef}, info::{func_info::FuncInfo, types::TypeInfo}, CheckedProgram};

pub mod lua_ast;
pub mod lua_builtins;

pub struct TranspileContext
{
    pub resolver: RefCell<IdResolver>,
    pub self_var: Option<String>,
    pub structs: HashMap<Uuid, Arc<StructDef>>,
    pub funcs: HashMap<Uuid, Arc<FuncDef>>,
    pub continue_label_stack: RefCell<LabelStack>,
}

impl TranspileContext
{
    pub fn new(structs: &Vec<Arc<StructDef>>, funcs: &Vec<Arc<FuncDef>>) -> Self 
    {
        Self 
        {
            resolver: RefCell::new(IdResolver::new()),
            self_var: None,
            structs: structs.iter()
                .map(|s| (s.id.clone(), s.clone()))
                .collect(),
            funcs: funcs.iter()
                .map(|s| (s.id.clone(), s.clone()))
                .collect(),
            continue_label_stack: RefCell::new(LabelStack::new()),
        }
    }
}

pub struct IdResolver
{
    map: HashMap<Uuid, usize>,
    count: usize,
}

impl IdResolver
{
    pub fn new() -> Self
    {
        Self { 
            map: HashMap::new(),
            count: 0,
        }
    }

    pub fn resolve(&mut self, id: Uuid) -> String 
    {
        let i = self.map.entry(id).or_insert({
            self.count += 1;
            self.count - 1
        });

        format!("v_{}", i)
    }

    pub fn make_var(&mut self) -> String 
    {
        self.resolve(Uuid::new_v4())
    }
}

pub struct LabelStack
{
    id: usize,
    stack: Vec<usize>,
}

impl LabelStack
{
    pub fn new() -> Self 
    {
        Self 
        {
            id: 0,
            stack: vec![],
        }
    }

    pub fn push(&mut self)
    {
        self.stack.push(self.id);
        self.id += 1;
    }

    pub fn pop(&mut self)
    {
        self.stack.pop();
    }

    pub fn get(&self) -> String 
    {
        format!("l_{}", self.stack.last().unwrap())
    }
}

pub fn transpile(program: &CheckedProgram) -> LuaProgram
{
    let mut context = TranspileContext::new(&program.structs, &program.funcs);
    let mut lua_stmts = vec![];

    // forward declares functions
    forward_declare_builtin_funcs(program, &mut context, &mut lua_stmts);

    append_builtins(&mut lua_stmts, context.resolver.get_mut());
    lua_stmts.push(LuaStmt::Spacer);
    lua_stmts.push(LuaStmt::Spacer);
    lua_stmts.push(LuaStmt::Comment("Main function defs".into()));
    
    program.funcs.iter().filter_map(|f| visit_func_def(&mut context, f)).for_each(|f| {
        lua_stmts.push(f);
    });

    lua_stmts.push(LuaStmt::Spacer);
    lua_stmts.push(LuaStmt::Spacer);
    lua_stmts.push(LuaStmt::Comment("Main invoker function".into()));
    create_main_invoker(&mut lua_stmts, context.resolver.get_mut(), program.main.clone());
    LuaProgram { stmts: lua_stmts }
}

fn forward_declare_builtin_funcs(program: &CheckedProgram, context: &mut TranspileContext, lua_stmts: &mut Vec<LuaStmt>) 
{
    let mut pairs = program.funcs.iter().filter_map(|f| {
        let name = context.resolver.get_mut().resolve(f.id.clone());
        let init = Box::new(LuaExpr::Literal(LuaLit::Nil));
        Some((name, init))
    }).collect_vec();

    pairs.push((
        context.resolver.get_mut().resolve(IS_NONE_ID.clone()),
        Box::new(LuaExpr::Literal(LuaLit::Nil))
    ));

    pairs.push((
        context.resolver.get_mut().resolve(UNWRAP_ID.clone()),
        Box::new(LuaExpr::Literal(LuaLit::Nil))
    ));

    lua_stmts.push(LuaStmt::LocalDecl { pairs });
}

fn visit_func_def(context: &mut TranspileContext, def: &FuncDef) -> Option<LuaStmt>
{
    let Some(def_body) = &def.body else { // skips builtin functions
        return None;
    };

    let func_var_name = context.resolver.borrow_mut().resolve(def.id.clone());
    
    if def.has_self
    {
        context.self_var = Some(context.resolver.borrow_mut().resolve(def.params[0].id))
    }

    let func_vars = def_body.vars.values()
        .map(|var| {
            let name = context.resolver.borrow_mut().resolve(var.id.clone());
            (
                name.clone(), 
                LuaExpr::Binary { 
                    left: LuaExpr::Id(name.clone()).to_box(), 
                    op: LuaBinaryOp::Or, 
                    right: LuaExpr::Literal(LuaLit::Nil).to_box() 
                }.to_box(),
                var.initializer.is_some(),
            )
        })
        .collect_vec();

    let func_body = if def_body.vars.values().find(|v| v.initializer.is_some()).is_some()
    {
        let pairs = func_vars.iter()
            .filter(|(_, _, has_init)| *has_init)
            .map(|(name, expr, _)| (name.clone(), expr.clone())).collect();

        LuaStmt::Block { stmts: [
            LuaStmt::LocalDecl { 
                pairs
            },
            visit_stmt(&def_body.block, context, &def_body.vars)
        ].into()}
    }
    else 
    {
        visit_stmt(&def_body.block, context, &def_body.vars)
    };

    context.self_var = None; // make sure to reset


    let value = Box::new(LuaExpr::FuncDef { 
        args: def.params.iter().map(|p| context.resolver.borrow_mut().resolve(p.id.clone())).collect(), 
        body: vec![func_body]
    });

    Some(LuaStmt::Assign { pairs: vec![(func_var_name, value)] })
}

fn visit_stmt(stmt: &TypedStatement, context: &TranspileContext, vars: &HashMap<Uuid, VarDef>) -> LuaStmt
{
    match stmt
    {
        TypedStatement::Let(uuid) => {
            let name = context.resolver.borrow_mut().resolve(uuid.clone());
            let expr = visit_expr(vars.get(uuid).unwrap().initializer.as_ref().unwrap(), context, vars).to_box();
            LuaStmt::Assign { pairs: [(name, expr)].into() }
        },
        TypedStatement::Expr(expr) => {
            let expr = visit_expr(expr, context, vars);
            LuaStmt::Call { 
                expr: LuaExpr::Id(INVOKE_EXPR_FUNC.into()).to_box(), 
                args: vec![expr]
            }
        },
        TypedStatement::Assign { assigned, expression } => {
            LuaStmt::AssignExpr { 
                assigned: visit_expr(assigned, context, vars).to_box(), 
                value: visit_expr(&expression, context, vars).to_box() 
            }
        },
        TypedStatement::Block(stmts) => {
            LuaStmt::Block { 
                stmts: stmts.iter().map(|s| visit_stmt(s, context, vars)).collect() 
            }
        },
        TypedStatement::If { expression, body, else_block } => {
            LuaStmt::If { 
                condition: visit_expr(&expression, context, vars).to_box(), 
                body: [
                    visit_stmt(&body, context, vars)
                ].to_vec(), 
                else_body: else_block.as_ref().map(|e| [visit_stmt(&e, context, vars)].into()) 
            }
        },
        TypedStatement::Break => LuaStmt::Break,
        TypedStatement::Continue => LuaStmt::Goto { label: context.continue_label_stack.borrow().get() },
        TypedStatement::Return { returned, loc: _ } => {
            LuaStmt::Return(returned.as_ref().map(|r| visit_expr(&r, context, vars).to_box()))
        },
        TypedStatement::For { var_id, condition, body } => {
            let var_name = context.resolver.borrow_mut().resolve(var_id.clone());
            let iter_var = context.resolver.borrow_mut().make_var();

            let iter_val = visit_expr(&condition, context, vars).to_box();
            let iter_expr = LuaExpr::Call { 
                expr: LuaExpr::Id(iter_var.clone()).to_box(), 
                args: vec![] 
            };

            context.continue_label_stack.borrow_mut().push();

            let loop_stmt = LuaStmt::Block { 
                stmts: [
                    LuaStmt::LocalDecl { pairs: [(iter_var.clone(), iter_val.clone().to_box())].to_vec() },
                    LuaStmt::LocalDecl { pairs: [(var_name.clone(), iter_expr.clone().to_box())].to_vec() },
                    LuaStmt::While { 
                        cond: LuaExpr::Binary { 
                            left: LuaExpr::Id(var_name.clone()).to_box(), 
                            op: LuaBinaryOp::NoEq, 
                            right: LuaExpr::Literal(LuaLit::Nil).to_box()
                        }.to_box(), 
                        stmts: [
                            visit_stmt(&body, context, vars),
                            LuaStmt::Label(context.continue_label_stack.borrow().get()),
                            LuaStmt::Assign { pairs: [(var_name.clone(), iter_expr.clone().to_box())].to_vec() }
                        ].to_vec()
                    },
                ].into()
            };

            context.continue_label_stack.borrow_mut().push();
            loop_stmt
        },
        TypedStatement::While { condition, body } => {
            context.continue_label_stack.borrow_mut().push();

            let while_stmt = LuaStmt::While { 
                cond: visit_expr(&condition, context, vars).to_box(), 
                stmts: [
                    visit_stmt(body, context, vars),
                    LuaStmt::Label(context.continue_label_stack.borrow().get())
                ].to_vec()
            };

            context.continue_label_stack.borrow_mut().pop();
            while_stmt
        },
    }
}

fn visit_expr(expr: &TypedExpression, context: &TranspileContext, vars: &HashMap<Uuid, VarDef>) -> LuaExpr
{
    match expr {
        TypedExpression::Literal { returned: _, loc: _, value } => {
            match value 
            {
                LitVal::String(s) => LuaExpr::Literal(LuaLit::String(s.clone())),
                LitVal::Int(i) => LuaExpr::Literal(LuaLit::Number(*i as f64)),
                LitVal::Float(f) => LuaExpr::Literal(LuaLit::Number(*f)),
                LitVal::Bool(b) => match b {
                    true => LuaExpr::Literal(LuaLit::True),
                    false => LuaExpr::Literal(LuaLit::False),
                },
                LitVal::Null => LuaExpr::Literal(LuaLit::Nil),
                LitVal::SelfVal => LuaExpr::Id(context.self_var.as_ref().unwrap().clone()),
            }
        },
        TypedExpression::Array { expressions, returned: _, loc: _ } => {
            LuaExpr::Array(expressions.iter().map(|e| visit_expr(e, context, vars)).collect())
        },
        TypedExpression::Identifier { id, returned: _, loc: _ } => {
            let id = match id {
                TypedIdentifier::Function(id) => id,
                TypedIdentifier::Variable(id) => id,
            };

            let name = context.resolver.borrow_mut().resolve(id.clone());
            LuaExpr::Id(name)
        },
        TypedExpression::Lambda { parameters, body, returned: _, loc: _ } => {
            let args = parameters.iter().map(|p| context.resolver.borrow_mut().resolve(p.id.clone())).collect();

            LuaExpr::Group(LuaExpr::FuncDef { 
                args, 
                body: vec![visit_stmt(&body, context, vars)]
            }.to_box())
        },
        TypedExpression::Binary { left, op, right, returned: _, loc: _ } => {
            let mut op = LuaBinaryOp::from_op(*op);
            if (left.returned() == &*STRING_TYPE || right.returned() == &*STRING_TYPE) && op == LuaBinaryOp::Add
            {
                op = LuaBinaryOp::Concat;
            }

            let left = visit_expr(&left, context, vars).to_box();
            let right = visit_expr(&right, context, vars).to_box();

            LuaExpr::Binary { left, op, right }
        },
        TypedExpression::Unary { operand, op, returned: _, loc: _ } => {
            let expr = visit_expr(&operand, context, vars).to_box();
            let op = LuaUnaryOp::from_op(*op);
            LuaExpr::Unary { expr, op }
        },
        TypedExpression::Call { called, args, returned: _, loc: _ } => {
            let expr = visit_expr(&called, context, vars).to_box();
            let args = args.iter().map(|a| visit_expr(a, context, vars)).collect();
            LuaExpr::Call { expr, args }
        },
        TypedExpression::Index { indexed, arg, returned: _, loc: _ } => {
            LuaExpr::Index { 
                expr: visit_expr(&indexed, context, vars).to_box(), 
                arg: LuaExpr::Binary { 
                    left: visit_expr(&arg, context, vars).to_box(), 
                    op: LuaBinaryOp::Add, 
                    right: LuaExpr::Literal(LuaLit::Number(1.0)).to_box(),
                }.to_box()
            }
        },
        TypedExpression::Access { accessed, name, returned: _, is_assignable: _, loc: _ } => {
            let accessed_type = accessed.returned().clone();
            let accessed = visit_expr(&accessed, context, vars);
            match name
            {
                Left(member) => {
                    LuaExpr::Access { expr: accessed.to_box(), name: to_member_name(&member) }
                },
                Right(fn_id) => {

                    let args = if let Some(def) = context.funcs.get(fn_id) {
                        (1..def.params.len())
                            .map(|_| context.resolver.borrow_mut().make_var())
                            .collect_vec()
                    }
                    else // is a builtin function
                    {
                        let info = builtins::get_builtin_member_func(Some(&accessed_type), fn_id).unwrap();
                        (1..info.parameters.len())
                            .map(|_| context.resolver.borrow_mut().make_var())
                            .collect_vec()
                    };

                    let mut forwarded_args = vec![];
                    forwarded_args.push(accessed);
                    forwarded_args.extend(args.iter().map(|a| LuaExpr::Id(a.clone())));
                    
                    LuaExpr::Group(LuaExpr::FuncDef { 
                        args: args.clone(), 
                        body: [
                            LuaStmt::Return(Some(LuaExpr::Call { 
                                expr: LuaExpr::Id(context.resolver.borrow_mut().resolve(fn_id.clone())).to_box(), 
                                args: forwarded_args,
                            }.to_box()))
                        ].into()
                    }.to_box())
                }
            }
        },
        TypedExpression::TypeAccess { accessed: _, name: _, func_id, returned: _, loc: _ } => {
            LuaExpr::Id(context.resolver.borrow_mut().resolve(func_id.clone()))
        },
        TypedExpression::Cast { casted, type_info: _, returned: _, loc: _ } => {
            visit_expr(&casted, context, vars)
        },
        TypedExpression::Construction { type_id, args, returned: _, loc: _ } => {
            let s = context.structs.get(type_id).unwrap();

            // arguments that have been initialized
            let init: HashSet<String> = args.iter().map(|a| a.0.clone()).collect();
            let mut members = args.iter().map(|(a, e)| (a.clone(), visit_expr(e, context, vars))).collect_vec();
            for m in &s.members
            {
                if !init.contains(&m.name)
                {
                    members.push((m.name.clone(), visit_expr(m.init.as_ref().unwrap(), context, vars)));
                }
            }

            let members = members.into_iter().map(|(a, e)| (to_member_name(&a), e)).collect();
            LuaExpr::Map { members }
        },
    }
}

fn create_main_invoker(stmts: &mut Vec<LuaStmt>, resolver: &mut IdResolver, main_id: Uuid)
{
    let expr = LuaExpr::Id(resolver.resolve(main_id));

    let invoker_func = LuaStmt::Call { 
        expr: Box::new(expr), 
        args: vec![] 
    };

    stmts.push(invoker_func);
}

fn to_member_name(name: &str) -> String 
{
    format!("m_{}", name)
}