use std::sync::Arc;

use compiler::CompilerError;
use itertools::Itertools;
use utils::FileInfo;
use validation::{ast::{ExprCheckArgs, TypedExpression}, builtins, TypeLibraryBuilder, TypeResolver};
// use validation::StructDef;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;

fn main() 
{
    let mut builder = TypeLibraryBuilder::new();
    builder.append_builtins(vec![], builtins::get_builtins());

    let file = FileInfo::from_text("struct Test { name: String, age: Int }", "src".into());
    let file = compiler::run_parser(Arc::new(file)).unwrap();
    builder.append_file(Arc::new(file));

    let library = builder.build().unwrap();
    let operators = builtins::get_operators();

    let src = "Test { name: \"Nate Craver\", age: 21 }";
    let file = &FileInfo::from_text(src, "src".into());
    let expression = compiler::run_expression_parser(file).unwrap();

    let expression = TypedExpression::check_expr(&expression, ExprCheckArgs {
        library: &library,
        operators: &operators,
        file,
        usings: &vec![vec!["src".into()]]
    });

    match expression
    {
        Ok(ok) => println!("{:#?}", ok),
        Err(error) => println!("{}", error.format_error()),
    }
}
