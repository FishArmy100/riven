use std::sync::Arc;

use compiler::CompilerError;
use itertools::Itertools;
use utils::FileInfo;
use validation::{TypeLibraryBuilder, TypeResolver};
// use validation::StructDef;

pub mod lexing;
pub mod utils;
pub mod compiler;
pub mod parsing;
pub mod validation;

fn main() 
{
    let files = get_files().into_iter().map(|(src, path)| {
        let file = FileInfo::from_text(&src, path).as_arc();
        let node = compiler::run_parser(file).unwrap();
        Arc::new(node)
    }).collect_vec();
    
    let library = files.iter().fold(TypeLibraryBuilder::new(), |mut b, f| { b.append_file(f.clone()); b }).build();

    let library = match library {
        Ok(ok) => ok,
        Err(errors) => {
            for e in errors
            {
                println!("{}", e.format_error())
            }
            return;
        }
    };

    let usings = vec![vec!["src".to_string()]];
    let id = library.resolver().resolve_name("Test", &usings).unwrap();
    let t = library.get_type(&id);
    println!("{:#?}", t);
}

fn get_files() -> Vec<(String, String)>
{
    let f1 = "
        struct Int {}
        struct Float {}
    ".to_string();

    let f2 = "struct String {}".to_string();

    let f3 = "struct Void {}".to_string();

    let f4 = "
        use lib.numbers;
        use lib.string;

        struct Test { 
            number: Int, 
            grants: []String 
        }
    ".to_string();

    vec![
        (f1, "lib/numbers".into()),
        (f2, "lib/string".into()),
        (f3, "".into()),
        (f4, "src".into())
    ]
}
