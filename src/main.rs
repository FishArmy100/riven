use config::parse_args;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;
pub mod transpiling;
pub mod lua_runtime;
pub mod config;

fn main() 
{
    let config = parse_args();
    match compiler::compile_program(&config)
    {
        Ok(_) => println!("Program compiled successfully!"),
        Err(errors) => {
            println!("Program compiled with errors:");
            for e in errors
            {
                println!(" - Error: {}", e);
            }
        }
    }
}
