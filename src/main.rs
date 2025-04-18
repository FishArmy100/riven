use itertools::Itertools;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;

fn main() 
{
    let file = "tests/tick-tack-toe.rvn";
    let src = utils::read_file(file)
        .unwrap()
        .chars()
        .collect_vec();

    let result = compiler::run_parser(&src, Some(file));

    match result 
    {
        Ok(ok) => utils::write_file("out/tick-tack-toe.ast", &format!("{:#?}", ok)).unwrap(),
        Err(errors) => {
            for error in errors
            {
                println!("{}", error);
            }
        },
    }
}
