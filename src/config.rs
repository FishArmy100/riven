use std::path::Path;

use clap::{Arg, Command};

#[derive(Debug)]
pub struct CompilerConfig {
    pub input_file: String,
    pub output_file: String,
    pub debug_lex: bool,
    pub debug_parse: bool,
    pub debug_validate: bool,
    pub run_after_compile: bool,
}

fn has_valid_extension(filename: &str, required_ext: &str) -> bool 
{
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext_str| ext_str == required_ext)
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
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .help("Optional output file")
            .value_name("FILE"))
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
        output_file: matches
            .get_one::<String>("output")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "out/out.lua".to_string()),
        debug_lex: matches.get_flag("debug_lex"),
        debug_parse: matches.get_flag("debug_parse"),
        debug_validate: matches.get_flag("debug_validate"),
        run_after_compile: matches.get_flag("run"),
    }
}