use utils::{write_file, FileInfo};

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;
pub mod transpiling;

fn main() 
{
    let file = FileInfo::read("tests/tick-tack-toe.rvn", "src").unwrap().as_arc();
    match compiler::run_validator(&[file])
    {
        Ok(ok) => {
            println!("Program compiled successfully!");
            write_file("out/checked.txt", &format!("{:#?}", ok)).unwrap()
        },
        Err(errs) => {
            println!("Program compiled with errors:");
            for e in errs
            {
                println!(" - {}", e)
            }
        },
    }
}
