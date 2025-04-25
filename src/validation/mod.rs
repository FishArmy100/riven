use std::{collections::HashSet, sync::Arc};

use defs::{func_def::FuncDef, struct_def::StructDef};
use info::InfoContext;
use itertools::Itertools;
use operators::GlobalOperators;
use type_error::TypeError;

use crate::{parsing::ast::{FileNode, Program}, utils::TextPos};

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
}

impl CheckedProgram
{
    pub fn new(program: &Program) -> Result<Self, Vec<TypeError>>
    {
        let context = InfoContext::new(program)?;
        let operators = GlobalOperators::new();

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

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(CheckedProgram { 
            structs, 
            funcs 
        })
    }
}


fn check_usings(file: &FileNode, all_paths: &HashSet<Vec<String>>) -> (Vec<Vec<String>>, Vec<TypeError>)
{
    let mut usings = file.usings.iter()
        .map(|u| {
            let path = u.ids.iter()
                .map(|id| id.value_string().unwrap().clone())
                .collect_vec();

            let mut pos = u.ids[0].pos;
            for i in 1..u.ids.len()
            {
                pos = pos + u.ids[i].pos;
            }

            (path, pos)
        })
        .collect_vec();

    usings.push((file.info.path.split_relative(), TextPos::uniform(0)));
    usings.push((vec![], TextPos::uniform(0)));

    let errors = usings.iter().filter_map(|(path, pos)| {
        if !all_paths.contains(path)
        {
            Some(TypeError::UnknownUsing(path.clone(), pos.get_loc(&file.info)))
        }
        else 
        {
            None
        }
    }).collect_vec();
    
    let usings = usings.into_iter().map(|u| u.0).dedup().collect_vec();
    (usings, errors)
}

fn get_all_file_paths(program: &Program) -> HashSet<Vec<String>>
{
    let mut paths = program.files.iter().map(|f| f.info.path.split_relative()).collect::<HashSet<_>>();
    paths.insert(vec![]);
    paths
}