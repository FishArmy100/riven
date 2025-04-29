use std::{cell::RefCell, collections::{HashMap, HashSet}, sync::Arc};

use itertools::Itertools;
use lua_ast::*;
use lua_builtins::{append_builtins, INVOKE_EXPR_FUNC};
use uuid::Uuid;

use crate::validation::{ast::{stmt::TypedStatement, LitVal, TypedExpression, TypedIdentifier}, defs::{func_def::FuncDef, struct_def::StructDef, var_def::VarDef}, CheckedProgram};

pub mod lua_ast;
pub mod lua_builtins;

pub struct IdResolver
{
    map: HashMap<Uuid, usize>,
    count: usize,
}

pub struct TranspileContext
{
    pub resolver: RefCell<IdResolver>,
    pub self_var: Option<String>,
    pub structs: HashMap<Uuid, Arc<StructDef>>
}

impl TranspileContext
{
    pub fn new(structs: &Vec<Arc<StructDef>>) -> Self 
    {
        Self 
        {
            resolver: RefCell::new(IdResolver::new()),
            self_var: None,
            structs: structs.iter()
                .map(|s| (s.id.clone(), s.clone()))
                .collect()
        }
    }
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

pub fn transpile(program: &CheckedProgram) -> LuaProgram
{
    let mut context = TranspileContext::new(&program.structs);
    let mut lua_stmts = vec![];

    // forward declares functions
    let pairs = program.funcs.iter().filter_map(|f| {
        let name = context.resolver.get_mut().resolve(f.id.clone());
        let init = Box::new(LuaExpr::Literal(LuaLit::Nil));
        Some((name, init))
    }).collect_vec();

    lua_stmts.push(LuaStmt::LocalDecl { pairs });

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

fn visit_func_def(context: &TranspileContext, def: &FuncDef) -> Option<LuaStmt>
{
    let Some(def_body) = &def.body else { // skips builtin functions
        return None;
    };

    let var_name = context.resolver.borrow_mut().resolve(def.id.clone());

    let func_body = if def_body.vars.len() > 0
    {
        LuaStmt::Block { stmts: [
            LuaStmt::LocalDecl { 
                pairs: def_body.vars.keys().map(|id| (context.resolver.borrow_mut().resolve(id.clone()), Box::new(LuaExpr::Literal(LuaLit::Nil)))).collect()
            },
            visit_stmt(&def_body.block, context, &def_body.vars)
        ].into()}
    }
    else 
    {
        visit_stmt(&def_body.block, context, &def_body.vars)
    };


    let value = Box::new(LuaExpr::FuncDef { 
        args: def.params.iter().map(|p| context.resolver.borrow_mut().resolve(p.id.clone())).collect(), 
        body: vec![func_body]
    });

    Some(LuaStmt::Assign { pairs: vec![(var_name, value)] })
}

fn visit_stmt(stmt: &TypedStatement, context: &TranspileContext, vars: &HashMap<Uuid, VarDef>) -> LuaStmt
{
    match stmt
    {
        TypedStatement::Let(uuid) => {
            let name = context.resolver.borrow_mut().resolve(uuid.clone());
            let expr = visit_expr(vars.get(uuid).unwrap().initializer.as_ref().unwrap(), context).to_box();
            LuaStmt::Assign { pairs: [(name, expr)].into() }
        },
        TypedStatement::Expr(expr) => {
            let expr = visit_expr(expr, context);
            LuaStmt::Call { 
                expr: LuaExpr::Id(INVOKE_EXPR_FUNC.into()).to_box(), 
                args: vec![expr]
            }
        },
        TypedStatement::Assign { assigned, expression } => todo!(),
        TypedStatement::Block(stmts) => {
            LuaStmt::Block { 
                stmts: stmts.iter().map(|s| visit_stmt(s, context, vars)).collect() 
            }
        },
        TypedStatement::If { expression, body, else_block } => todo!(),
        TypedStatement::Break => todo!(),
        TypedStatement::Continue => todo!(),
        TypedStatement::Return { returned, loc } => todo!(),
        TypedStatement::For { var_id, condition, body } => todo!(),
        TypedStatement::While { condition, body } => todo!(),
    }
}

fn visit_expr(expr: &TypedExpression, context: &TranspileContext) -> LuaExpr
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
        TypedExpression::Array { expressions, returned, loc } => todo!(),
        TypedExpression::Identifier { id, returned: _, loc: _ } => {
            let id = match id {
                TypedIdentifier::Function(id) => id,
                TypedIdentifier::Variable(id) => id,
            };

            let name = context.resolver.borrow_mut().resolve(id.clone());
            LuaExpr::Id(name)
        },
        TypedExpression::Lambda { parameters, body, returned, loc } => todo!(),
        TypedExpression::Binary { left, op, right, returned: _, loc: _ } => {
            let left = visit_expr(&left, context).to_box();
            let op = LuaBinaryOp::from_op(*op);
            let right = visit_expr(&right, context).to_box();

            LuaExpr::Binary { left, op, right }
        },
        TypedExpression::Unary { operand, op, returned: _, loc: _ } => {
            let expr = visit_expr(&operand, context).to_box();
            let op = LuaUnaryOp::from_op(*op);
            LuaExpr::Unary { expr, op }
        },
        TypedExpression::Call { called, args, returned: _, loc: _ } => {
            let expr = visit_expr(&called, context).to_box();
            let args = args.iter().map(|a| visit_expr(a, context)).collect();
            LuaExpr::Call { expr, args }
        },
        TypedExpression::Index { indexed, arg, returned, loc } => todo!(),
        TypedExpression::Access { accessed, name, returned, is_assignable, loc } => todo!(),
        TypedExpression::TypeAccess { accessed, name, func_id, returned, loc } => todo!(),
        TypedExpression::Cast { casted, type_info: _, returned: _, loc: _ } => {
            visit_expr(&casted, context)
        },
        TypedExpression::Construction { type_id, args, returned, loc } => {
            let s = context.structs.get(type_id).unwrap();

            // arguments that have been initialized
            let init: HashSet<String> = args.iter().map(|a| a.0.clone()).collect();
            let mut members = args.iter().map(|(a, e)| (a.clone(), visit_expr(e, context))).collect_vec();
            for m in &s.members
            {
                if !init.contains(&m.name)
                {
                    members.push((m.name.clone(), visit_expr(m.init.as_ref().unwrap(), context)));
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