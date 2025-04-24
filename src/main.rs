use std::sync::Arc;

use compiler::CompilerError;
use itertools::Itertools;
use parsing::ast::Program;
use utils::FileInfo;
use validation::{ast::{stmt::TypedStatement, ExprCheckArgs, TypedExpression}, builtins, functions::FuncLibraryBuilder, var::VariableStack, TypeLibraryBuilder, TypeResolver, ValidationContext};
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

    let src = "
    {
        let x = 5;
        let y = x * 2;
        let t = Test {
            name: \"Nate Craver\",
            age: y
        };
    }
    ";
    let file = &FileInfo::from_text(src, "src".into());
    let block = compiler::run_block_parser(file).unwrap();

    let usings = vec![vec!["src".into()]];
    let mut var_stack = VariableStack::new();
    let mut args = context.check_stmt_args(&usings, &file, &mut var_stack);

    let expression = TypedStatement::check_block(&block, &mut args);

    match expression
    {
        Ok(ok) => {
            println!("Variables: \n{:#?}", var_stack);
            println!("{:#?}", ok)
        },
        Err(errors) => {
            for e in errors
            {
                println!("{}", e.format_error())
            }
        },
    }
}
