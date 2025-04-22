use compiler::CompilerError;
use itertools::Itertools;
use utils::FileInfo;
// use validation::StructDef;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;

fn main() 
{
    let file = "tests/test.rvn";

    let file = FileInfo::read(file).unwrap();

    let result = compiler::run_parser(&file);

    match result 
    {
        Ok(ok) => {
            println!("{:#?}", ok);
            // match StructDef::get_defs(&ok)
            // {
            //     Ok(ok) => println!("{:#?}", ok),
            //     Err(errors) => {
            //         for error in errors.iter().map(|e| e.format_error(&src, Some(file)))
            //         {
            //             println!("{}", error);
            //         }
            //     }
            // }
        },
        Err(errors) => {
            for error in errors
            {
                println!("{}", error);
            }
        },
    }
}
