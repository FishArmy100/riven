use lua_runtime::run_lua;
use transpiling::{lua_ast::{LuaExpr, LuaLit, LuaProgram, LuaStmt}, transpile};
use utils::{write_file, FileInfo};

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;
pub mod transpiling;
pub mod lua_runtime;

fn main() 
{
    let file = FileInfo::read("tests/tick-tack-toe.rvn", "src").unwrap().as_arc();
    let program = match compiler::run_validator(&[file])
    {
        Ok(ok) => ok,
        Err(errs) => {
            println!("Program compiled with errors:");
            for e in errs
            {
                println!(" - {}", e)
            }
            return;
        },
    };

    
    write_file("out/tick-tack-toe.ast", &format!("{:#?}", program)).unwrap();
    
    let lua_program = transpile(&program);

    let lua = lua_program.to_string("\t".into());
    write_file("out/tick-tack-toe.lua", &lua).unwrap();
    match run_lua(&lua)
    {
        Err(e) => {
            println!("{}", e.to_string())
        }
        Ok(_) => {},
    }
}
