use uuid::Uuid;

use crate::validation::builtins::{IS_NONE_ID, PRINTLN_ID, PRINT_ID, READ_LINE_ID, STRING_TO_INT_ID, UNWRAP_ID};

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
    stmts.push(create_is_none_builtin(resolver));
    stmts.push(create_unwrap_builtin(resolver));
    stmts.push(create_to_int_builtin(resolver));
    stmts.push(create_read_line_builtin(resolver));
}

fn create_print_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let msg_arg = resolver.make_var();
    let body = [
        LuaStmt::Call { 
            expr: Box::new(LuaExpr::Id("io.write".into())), 
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
                LuaExpr::Id(msg_arg.clone())
            ].into()
        }
    ].into();

    create_builtin(PRINTLN_ID.clone(), body, vec![msg_arg], resolver)
}

fn create_read_line_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let body = [
        LuaStmt::Return(Some(LuaExpr::Call { 
            expr: Box::new(LuaExpr::Id("io.read".into())), 
            args: vec![]
        }.to_box()))
    ].into();

    create_builtin(READ_LINE_ID.clone(), body, vec![], resolver)
}

fn create_is_none_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let self_arg = resolver.make_var();
    let body = [
        LuaStmt::Return(Some(LuaExpr::Binary { 
            left: LuaExpr::Id(self_arg.clone()).to_box(), 
            op: LuaBinaryOp::Eq, 
            right: LuaExpr::Literal(LuaLit::Nil).to_box() 
        }.to_box()))
    ].to_vec();

    create_builtin(IS_NONE_ID.clone(), body, vec![self_arg], resolver)
}

fn create_unwrap_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let self_arg = resolver.make_var();
    let body = [
        LuaStmt::Return(Some(
            LuaExpr::Id(self_arg.clone()).to_box()
        ))
    ].to_vec();

    create_builtin(UNWRAP_ID.clone(), body, vec![self_arg], resolver)
}

fn create_to_int_builtin(resolver: &mut IdResolver) -> LuaStmt
{
    let self_arg = resolver.make_var();
    let get_input_expr = LuaExpr::Call { expr: LuaExpr::Id("tonumber".into()).to_box(), args: vec![LuaExpr::Id(self_arg.clone())] };
    let test_int_arg = resolver.make_var();
    let body = [
        LuaStmt::LocalDecl { pairs: vec![(test_int_arg.clone(), get_input_expr.to_box())] },
        LuaStmt::If { 
            condition: LuaExpr::Binary { 
                left: LuaExpr::Id(test_int_arg.clone()).to_box(), 
                op: LuaBinaryOp::And, 
                right: LuaExpr::Binary { 
                    left: LuaExpr::Id(test_int_arg.clone()).to_box(), 
                    op: LuaBinaryOp::Eq, 
                    right: LuaExpr::Call { 
                        expr: LuaExpr::Id("math.floor".into()).to_box(), 
                        args: vec![LuaExpr::Id(test_int_arg.clone())] 
                    }.to_box()
                }.to_box()
            }.to_box(), 
            body: [
                LuaStmt::Return(Some(LuaExpr::Id(test_int_arg).to_box()))
            ].to_vec(), 
            else_body: None,
        },
        LuaStmt::Return(Some(LuaExpr::Literal(LuaLit::Nil).to_box()))
    ].to_vec();

    create_builtin(STRING_TO_INT_ID.clone(), body, vec![self_arg], resolver)
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