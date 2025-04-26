use std::sync::Arc;
use compiler::CompilerError;
use parsing::ast::{FileNode, Program};
use utils::{write_file, FileInfo};
use validation::CheckedProgram;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;

fn main() 
{
    let file_1 = parse_file("tests/test.rvn", "src/test");

    let program = Program {
        files: vec![file_1]
    };

    println!("Context compiled");

    let checked = CheckedProgram::new(&program);
    match checked
    {
        Ok(ok) => write_file("out/checked.txt", &format!("{:#?}", ok)).unwrap(),
        Err(errs) => {
            for e in errs
            {
                println!("{}", e.format_error())
            }
        },
    }
}

fn parse_file(path: &str, namespace: &str) -> Arc<FileNode>
{
    let file = FileInfo::read(path, namespace).unwrap();
    let file = compiler::run_parser(Arc::new(file)).unwrap();
    Arc::new(file)
}
