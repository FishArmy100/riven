use std::collections::HashMap;

use uuid::Uuid;

use crate::{parsing::ast::FileNode, validation::{ast::{stmt::{StmtCheckArgs, TypedStatement}, ExprCheckArgs, TypedExpression}, defs::var_def::VariableStack, info::{func_info::FuncInfo, types::TypeInfo, InfoContext}, operators::GlobalOperators, type_error::TypeError}};

use super::var_def::VarDef;

#[derive(Debug)]
pub struct FuncDef 
{
    pub id: Uuid,
    pub name: String,
    pub is_pub: bool,
    pub params: Vec<FuncDefParam>,
    pub body: FuncDefBody
}

#[derive(Debug)]
pub struct FuncDefParam
{
    pub name: String,
    pub type_info: TypeInfo,
    pub init: Option<Box<TypedExpression>>,
}

#[derive(Debug)]
pub struct FuncDefBody
{
    pub vars: HashMap<Uuid, VarDef>,
    pub block: Box<TypedStatement>,
}

impl FuncDef
{
    pub fn new(info: &FuncInfo, context: &InfoContext, operators: &GlobalOperators) -> Result<Self, Vec<TypeError>>
    {
        let mut errors = vec![];
        let mut params = vec![];

        let var_stack = VariableStack::new();

        let check_args = ExprCheckArgs {
            operators,
            context,
            var_stack: &var_stack,
            file: &info.file
        };

        for param in &info.parameters
        {
            let init = if let Some(init) = &param.init {
                let checked_init = match TypedExpression::check_expr(init, check_args) {
                    Ok(ok) => ok,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    },
                };

                if checked_init.returned() != &param.type_info
                {
                    errors.push(TypeError::ExpectedType(param.type_info.pretty_print(&check_args.context.structs), init.get_pos().get_loc(&check_args.file.info)));
                    continue;
                }

                Some(Box::new(checked_init))

            } else { None };

            params.push(FuncDefParam {
                name: param.name.clone(),
                type_info: param.type_info.clone(),
                init,
            });
        }

        let mut body_var_stack = VariableStack::new();
        let mut stmt_check_args = StmtCheckArgs {
            operators,
            context,
            file: &info.file,
            var_stack: &mut body_var_stack,
        };

        let body = match TypedStatement::check_block(&info.decl.body, &mut stmt_check_args){
            Ok(ok) => {
                let body = FuncDefBody {
                    vars: body_var_stack.get_vars(),
                    block: Box::new(ok),
                };

                Some(body)
            },
            Err(e) => {
                errors.extend(e);
                None
            },
        };

        if errors.len() > 0
        {
            return Err(errors)
        }

        Ok(FuncDef { 
            id: info.id.clone(), 
            name: info.name.clone(), 
            is_pub: info.is_pub, 
            params, 
            body: body.unwrap() 
        })
    }
}