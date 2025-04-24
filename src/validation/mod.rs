pub mod ast;
pub mod type_info;
pub mod builtins;
pub mod operators;
pub mod functions;
pub mod types;
pub mod var;
pub mod type_error;

use std::{collections::HashSet, sync::Arc};

use ast::{stmt::StmtCheckArgs, ExprCheckArgs, TypedExpression};
use functions::{FuncLibrary, FuncLibraryBuilder};
use type_error::TypeError;
use type_info::TypeInfo;
pub use types::*;

use itertools::Itertools;
use operators::GlobalOperators;
use uuid::Uuid;
use var::VariableStack;

use crate::{parsing::ast::{Expression, FileNode, Program}, utils::{FileInfo, TextLoc, TextPos}};

pub struct ValidationContext
{
    pub type_library: TypeLibrary,
    pub func_library: FuncLibrary,
    pub operators: GlobalOperators,
}

impl ValidationContext
{
    pub fn new(program: &Program) -> Result<Self, Vec<TypeError>>
    {
        let type_library = build_type_library(program)?;
        let operators = builtins::get_operators();
        let func_library = build_func_library(program, type_library.resolver())?;
        let var_stack = VariableStack::new();

        let mut errors = vec![];
        let all_paths = get_all_file_paths(program);

        for file in &program.files
        {
            let (usings, e) = check_usings(file, &all_paths);
            errors.extend(e.into_iter());

            let args = ExprCheckArgs {
                type_library: &type_library,
                operators: &operators,
                file: &file.info,
                usings: &usings,
                func_library: &func_library,
                var_stack: &var_stack,
            };

            if let Err(e) = type_library.build_initializers(args)
            {
                errors.extend(e.into_iter());
            }

            if let Err(e) = func_library.build_initializers(args)
            {
                errors.extend(e.into_iter());
            }
        }

        if errors.len() > 0
        {
            return Err(errors)
        }

        Ok(ValidationContext { 
            type_library, 
            func_library, 
            operators 
        })
    }

    pub fn get_check_args<'a>(&'a self, usings: &'a [Vec<String>], file: &'a FileInfo, var_stack: &'a VariableStack) -> ExprCheckArgs<'a>
    {
        ExprCheckArgs { 
            operators: &self.operators, 
            type_library: &self.type_library, 
            func_library: &self.func_library,
            file, 
            usings,
            var_stack,
        }
    }

    pub fn check_stmt_args<'a>(&'a self, usings: &'a [Vec<String>], file: &'a FileInfo, var_stack: &'a mut VariableStack) -> StmtCheckArgs<'a>
    {
        StmtCheckArgs { 
            operators: &self.operators, 
            type_library: &self.type_library, 
            func_library: &self.func_library,
            file, 
            usings,
            var_stack,
        }
    }
}

fn build_func_library(program: &Program, type_resolver: TypeResolver) -> Result<FuncLibrary, Vec<TypeError>>
{
    let mut builder = FuncLibraryBuilder::new();
    for file in &program.files
    {
        builder.append_file(file.clone());
    }

    builder.build(type_resolver)
}

fn build_type_library(program: &Program) -> Result<TypeLibrary, Vec<TypeError>>
{
    let mut builder = TypeLibraryBuilder::new();
    builder.append_builtins(vec![], builtins::get_builtins());
    for file in &program.files
    {
        builder.append_file(file.clone());
    }

    builder.build()
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

#[derive(Debug, Clone)]
pub enum Initializer
{
    None,
    AST(Arc<Expression>),
    Built(Arc<TypedExpression>),
}

impl Initializer
{
    pub fn has_init(&self) -> bool 
    {
        match self 
        {
            Self::None => false,
            _ => true,
        }
    }

    pub fn built(&self) -> Option<Arc<TypedExpression>>
    {
        match self 
        {
            Self::Built(b) => Some(b.clone()),
            _ => None,
        }
    }

    pub fn build(&mut self, args: ExprCheckArgs, expected: &TypeInfo) -> Result<(), TypeError>
    {
        if let Self::AST(expr) = self 
        {
            let built = TypedExpression::check_expr(expr, args)?;
            if built.returned() != expected
            {
                return Err(TypeError::ExpectedType(expected.pretty_print(args.type_library), expr.get_pos().get_loc(args.file)));
            }
            *self = Self::Built(Arc::new(built))
        }

        Ok(())
    }
}