use std::sync::Arc;

use builtins::STRING_TYPE;
use defs::{func_def::FuncDef, struct_def::StructDef};
use info::{types::TypeInfo, InfoContext};
use type_error::TypeError;
use uuid::Uuid;

use crate::parsing::ast::Program;

pub mod ast;
pub mod builtins;
pub mod operators;
pub mod type_error;
pub mod info;
pub mod defs;

#[derive(Debug)]
pub struct CheckedProgram
{
    pub structs: Vec<Arc<StructDef>>,
    pub funcs: Vec<Arc<FuncDef>>,
    pub main: Uuid,
}

impl CheckedProgram
{
    pub fn new(program: &Program) -> Result<Self, Vec<TypeError>>
    {
        let context = InfoContext::new(program)?;
        let operators = builtins::get_operators();

        let mut errors = vec![];
        let mut structs = vec![];
        let mut funcs = vec![];
        
        for (_, struct_info) in &context.structs
        {
            match StructDef::new(struct_info, &context, &operators)
            {
                Ok(ok) => structs.push(Arc::new(ok)),
                Err(e) => errors.extend(e),
            }
        }

        for (_, func_info) in &context.funcs
        {
            match FuncDef::new(func_info, &context, &operators)
            {
                Ok(ok) => funcs.push(Arc::new(ok)),
                Err(e) => errors.extend(e),
            }
        }

        let mut main_fn = None;
        for f in &funcs
        {
            if f.name == "main"
            {
                if f.params.len() != 0 && f.
                   params[0].type_info != TypeInfo::Array(Box::new(STRING_TYPE.clone()))
                {
                    errors.push(TypeError::InvalidMainArgs(f.name_loc.as_ref().unwrap().clone()));
                }
                if main_fn.is_none()
                {
                    main_fn = Some(f.id.clone())
                }
                else 
                {
                    errors.push(TypeError::DuplicateMainFn(f.name_loc.as_ref().unwrap().clone()));    
                }
            }
        }

        if main_fn.is_none()
        {
            errors.push(TypeError::NoMainFn);
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(CheckedProgram { 
            structs, 
            funcs,
            main: main_fn.unwrap()
        })
    }
}