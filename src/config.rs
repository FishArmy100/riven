use std::path::Path;

use clap::{Arg, Command};

#[derive(Debug)]
pub struct CompilerConfig {
    pub input_file: String,
    pub debug_lex: bool,
    pub debug_parse: bool,
    pub debug_validate: bool,
    pub run_after_compile: bool,
}

fn parse_args() -> CompilerConfig {
    let matches = Command::new("riven")
        .version("0.1")
        .about("A simple compiler frontend")
        .arg(Arg::new("input")
            .short('i')
            .long("input")
            .help("Input source file (.rvn)")
            .value_name("FILE")
            .required(true))
        .arg(Arg::new("debug_lex")
            .short('l')
            .long("debug-lex")
            .help("Print lexing debug info")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("debug_parse")
            .short('p')
            .long("debug-parse")
            .help("Print parsing debug info")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("debug_validate")
            .short('v')
            .long("debug-validate")
            .help("Print validation debug info")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("run")
            .short('r')
            .long("run")
            .help("Run after compiling")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    CompilerConfig {
        input_file: matches.get_one::<String>("input").unwrap().to_string(),
        debug_lex: matches.get_flag("debug_lex"),
        debug_parse: matches.get_flag("debug_parse"),
        debug_validate: matches.get_flag("debug_validate"),
        run_after_compile: matches.get_flag("run"),
    }
}