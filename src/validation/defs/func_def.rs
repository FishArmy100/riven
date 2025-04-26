use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{parsing::ast::FileNode, utils::{Shared, TextLoc}, validation::{ast::{stmt::{StmtCheckArgs, TypedStatement}, ExprCheckArgs, TypedExpression}, builtins::VOID_TYPE, defs::var_def::VariableStack, info::{func_info::{FuncDeclData, FuncInfo}, types::TypeInfo, InfoContext}, operators::GlobalOperators, type_error::TypeError}};

use super::var_def::VarDef;

#[derive(Debug)]
pub struct FuncDef 
{
    pub id: Uuid,
    pub name: String,
    pub is_pub: bool,
    pub returned: TypeInfo,
    pub params: Vec<FuncDefParam>,
    pub body: Option<FuncDefBody>,
    pub name_loc: Option<TextLoc>,
}

#[derive(Debug, Clone)]
pub struct FuncDefParam
{
    pub name: String,
    pub type_info: TypeInfo,
    pub init: Option<Arc<TypedExpression>>,
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

        let var_stack = Shared::new(VariableStack::new());

        let (decl, file) = match &info.decl_data
        {
            FuncDeclData::Decl { decl, file } => (decl, file),
            FuncDeclData::Builtin { params } => {
                return Ok(Self { 
                    id: info.id.clone(), 
                    name: info.name.clone(), 
                    is_pub: info.is_pub, 
                    returned: info.returned.clone(), 
                    params: params.clone(), 
                    body: None,
                    name_loc: None,
                })
            },
        };

        let check_args = ExprCheckArgs {
            operators,
            context,
            var_stack: var_stack.clone(),
            file: &file,
            self_type: info.parent.as_ref(),
            fn_ret_type: None,
        };

        for param in &info.parameters
        {
            let init = if let Some(init) = &param.init {
                let checked_init = match TypedExpression::check_expr(init, &check_args, Some(&param.type_info)) {
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

                Some(Arc::new(checked_init))

            } else { None };

            params.push(FuncDefParam {
                name: param.name.clone(),
                type_info: param.type_info.clone(),
                init,
            });
        }

        let body_var_stack = Shared::new(VariableStack::new());
        body_var_stack.get_mut().push_frame();
        for p in &info.parameters
        {
            if p.name == "self" { continue; }

            body_var_stack.get_mut().add_var(p.name.clone(), p.type_info.clone(), p.id.clone());
        }

        let mut stmt_check_args = StmtCheckArgs {
            operators,
            context,
            file: &file,
            var_stack: body_var_stack.clone(),
            fn_ret_type: Some(&info.returned),
            self_type: info.parent.as_ref(),
        };

        let body = match TypedStatement::check_block(&decl.body, &mut stmt_check_args){
            Ok(ok) => {
                let returns = ok.check_return(&info.returned, &stmt_check_args);
                match returns
                {
                    Ok(returns) => {
                        if !returns && info.returned != *VOID_TYPE
                        {
                            errors.push(TypeError::FunctionMustReturn(decl.fn_tok.get_loc(&file.info)));
                        }
                    },
                    Err(err) => errors.extend(err),
                }

                let body = FuncDefBody {
                    vars: body_var_stack.get().get_vars(),
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
            body: Some(body.unwrap()),
            returned: info.returned.clone(),
            name_loc: Some(decl.id.get_loc(&file.info))
        })
    }
}