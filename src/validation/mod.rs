pub mod ast;
pub mod type_info;
pub mod builtins;
pub mod operators;
pub mod functions;
pub mod types;
pub mod var;

use std::{collections::HashSet, sync::Arc};

use ast::{ExprCheckArgs, TypedExpression};
use functions::{FuncLibrary, FuncLibraryBuilder};
use type_info::TypeInfo;
pub use types::*;

use itertools::Itertools;
use operators::{BinaryOpType, GlobalOperators, UnaryOpType};

use crate::{compiler::CompilerError, lexing::token::Token, parsing::ast::{Expression, FileNode, Program}, utils::{FileInfo, TextLoc, TextPos}};

#[derive(Debug, Clone)]
pub enum TypeError
{
    UnknownType(Token, TextLoc),
    DuplicateTypeDef(Token, TextLoc),
    UnknownUsing(Vec<String>, TextLoc),
    ConflictingTypes(Token, TextLoc),
    NoBinaryOp(BinaryOpType, String, String, TextLoc),
    NoUnaryOp(UnaryOpType, String, TextLoc),
    CannotConstruct(String, TextLoc),
    InvalidConstructionArgs(TextLoc),
    ConflictingFunctions(Token, TextLoc),
    UndefinedFunction(Token, TextLoc),
    ExpectedType(String, TextLoc),
    FunctionArgumentNeedsInitializer(TextLoc),
    UnknownIdentifier(String, TextLoc),
    ExpectedFunction(TextLoc),
    InvalidCallArgs(Vec<String>, TextLoc),
}

impl CompilerError for TypeError
{
    fn msg(&self) -> String 
    {
        match self 
        {
            TypeError::UnknownType(token, _) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::DuplicateTypeDef(token, _) => format!("Duplicate type {}", token.value_string().unwrap()),
            TypeError::UnknownUsing(path, _) => format!("Using path {} does not exist", path.iter().join(".")),
            TypeError::ConflictingTypes(token, _) => format!("Conflicting type definitions for {}", token.value_string().unwrap()),
            TypeError::NoBinaryOp(op, left, right, _) => format!("No binary operator {} for types {} and {}", op.to_string(), left, right),
            TypeError::NoUnaryOp(op, t, _) => format!("No unary operator {} for type {}", op.to_string(), t),
            TypeError::CannotConstruct(t, _) => format!("Cannot construct type {}", t),
            TypeError::InvalidConstructionArgs(_) => format!("Invalid construction args"),
            TypeError::ConflictingFunctions(token, _) => format!("Conflicting function definitions for {}", token.value_string().unwrap()),
            TypeError::UndefinedFunction(token, _) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::ExpectedType(t, _) => format!("Expected type {}", t),
            TypeError::FunctionArgumentNeedsInitializer(_) => format!("Must have a function initializer"),
            TypeError::UnknownIdentifier(id, _) => format!("Unknown identifier {}", id),
            TypeError::ExpectedFunction(_) => format!("Expected a function"),
            TypeError::InvalidCallArgs(items, _) => format!("Expected call args: {}", items.iter().join(", ")),
        }
    }

    fn loc(&self) -> TextLoc 
    {
        match self 
        {
            TypeError::UnknownType(_, loc) => loc.clone(),
            TypeError::DuplicateTypeDef(_, loc) => loc.clone(),
            TypeError::UnknownUsing(_, loc) => loc.clone(),
            TypeError::ConflictingTypes(_, loc) => loc.clone(),
            TypeError::NoBinaryOp(_, _, _, loc) => loc.clone(),
            TypeError::NoUnaryOp(_, _, loc) => loc.clone(),
            TypeError::CannotConstruct(_, loc) => loc.clone(),
            TypeError::InvalidConstructionArgs(loc) => loc.clone(),
            TypeError::ConflictingFunctions(_, loc) => loc.clone(),
            TypeError::UndefinedFunction(_, loc) => loc.clone(),
            TypeError::ExpectedType(_, loc) => loc.clone(),
            TypeError::FunctionArgumentNeedsInitializer(loc) => loc.clone(),
            TypeError::UnknownIdentifier(_, loc) => loc.clone(),
            TypeError::ExpectedFunction(loc) => loc.clone(),
            TypeError::InvalidCallArgs(_, loc) => loc.clone(),
        }
    }
}

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

    pub fn get_check_args<'a>(&'a self, usings: &'a [Vec<String>], file: &'a FileInfo) -> ExprCheckArgs<'a>
    {
        ExprCheckArgs { 
            operators: &self.operators, 
            type_library: &self.type_library, 
            func_library: &self.func_library,
            file, 
            usings
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