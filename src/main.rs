use compiler::CompilerError;
use itertools::Itertools;
use validation::StructDef;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;

fn main() 
{
    let file = "tests/test.rvn";
    let src = utils::read_file(file)
        .unwrap()
        .chars()
        .collect_vec();

    let result = compiler::run_parser(&src, Some(file));

    match result 
    {
        Ok(ok) => {
            match StructDef::get_defs(&ok)
            {
                Ok(ok) => println!("{:#?}", ok),
                Err(errors) => {
                    for error in errors.iter().map(|e| e.format_error(&src, Some(file)))
                    {
                        println!("{}", error);
                    }
                }
            }
        },
        Err(errors) => {
            for error in errors
            {
                println!("{}", error);
            }
        },
    }
}
