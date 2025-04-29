use uuid::Uuid;

use crate::validation::builtins::{PRINTLN_ID, PRINT_ID};

use super::{lua_ast::*, IdResolver};

pub const INVOKE_EXPR_FUNC: &str = "_invoke_expr";
pub fn create_invoke_expr_func() -> LuaStmt
{
    LuaStmt::FuncDef { 
        name: INVOKE_EXPR_FUNC.to_string(), 
        args: [
            "arg".into()
        ].into(), 
        body: [
            LuaStmt::Return(Some(LuaExpr::Id("arg".into()).to_box()))
        ].into()
    }
}

pub fn append_builtins(stmts: &mut Vec<LuaStmt>, resolver: &mut IdResolver)
{
    stmts.push(create_invoke_expr_func());
    stmts.push(create_println_builtin(resolver));
    stmts.push(create_print_builtin(resolver));
}

fn create_print_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let msg_arg = resolver.make_var();
    let body = [
        LuaStmt::Call { 
            expr: Box::new(LuaExpr::Id("print".into())), 
            args: [
                LuaExpr::Id(msg_arg.clone())
            ].into()
        }
    ].into();

    create_builtin(PRINT_ID.clone(), body, vec![msg_arg], resolver)
}

fn create_println_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let msg_arg = resolver.make_var();
    let body = [
        LuaStmt::Call { 
            expr: Box::new(LuaExpr::Id("print".into())), 
            args: [
                LuaExpr::Binary { 
                    left: LuaExpr::Id(msg_arg.clone()).to_box(), 
                    op: LuaBinaryOp::Concat, 
                    right: LuaExpr::Literal(LuaLit::String("\"\\n\"".into())).to_box() 
                }
            ].into()
        }
    ].into();

    create_builtin(PRINTLN_ID.clone(), body, vec![msg_arg], resolver)
}

fn create_builtin(id: Uuid, body: Vec<LuaStmt>, args: Vec<String>, resolver: &mut IdResolver) -> LuaStmt
{
    let var_name = resolver.resolve(id.clone());

    let value = Box::new(LuaExpr::FuncDef { 
        args,
        body,
    });

    LuaStmt::Assign { pairs: vec![(var_name, value)] }
}