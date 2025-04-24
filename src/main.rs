use std::sync::Arc;

use compiler::CompilerError;
use itertools::Itertools;
use parsing::ast::Program;
use utils::FileInfo;
use validation::{ast::{ExprCheckArgs, TypedExpression}, builtins, functions::FuncLibraryBuilder, TypeLibraryBuilder, TypeResolver, ValidationContext};
use validation::StructDef;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;

fn main() 
{

    let file = FileInfo::from_text("

        struct Test { name: String, age: Int = 5 }
        
        // fn test()
        fn test(t: Test = Test { name: \"Nate\" }) -> Int
        {

        }

        fn build(a: Int, b: Float = 4.5, c: Int = 5) -> Int
        {
            return 7;
        }
        ", "src".into());
    let file = compiler::run_parser(Arc::new(file)).unwrap();
    let file = Arc::new(file);

    let program = Program {
        files: vec![file]
    };
    
    let context = match ValidationContext::new(&program) {
        Ok(ok) => ok,
        Err(err) => {
            for e in err 
            {
                println!("{}", e.format_error())
            }

            return;
        }
    };

    println!("Context compiled");

    let src = "build(5)";
    let file = &FileInfo::from_text(src, "src".into());
    let expression = compiler::run_expression_parser(file).unwrap();

    let usings = vec![vec!["src".into()]];
    let args = context.get_check_args(&usings, &file);

    let expression = TypedExpression::check_expr(&expression, args);

    match expression
    {
        Ok(ok) => println!("{:#?}", ok),
        Err(error) => println!("{}", error.format_error()),
    }
}
