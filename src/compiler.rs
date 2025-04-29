use std::sync::Arc;

use crate::{lexing::{self, token::{Token, TokenType}}, parsing::{self, ast::{BlockStmt, Expression, FileNode, Program}, token_reader::TokenReader, ParserError}, utils::{FileInfo, TextLoc, TextPos}, validation::CheckedProgram};

pub trait CompilerError
{
    fn msg(&self) -> String;
    fn loc(&self) -> Option<TextLoc>;

    fn format_error(&self) -> String 
    {
        match self.loc()
        {
            Some(loc) => format!("[{}]: {}", loc, self.msg()),
            None => format!("{}", self.msg())
        }
    }
}

pub fn run_lexer(file: &FileInfo) -> Result<Vec<Token>, Vec<String>>
{
    match lexing::lex_text(file)
    {
        Ok(ok) => Ok(ok),
        Err(err) => {
            Err(err.iter().map(|e| e.format_error()).collect())
        },
    }
}

pub fn run_parser(file: Arc<FileInfo>) -> Result<FileNode, Vec<String>>
{
    let tokens = run_lexer(&file)?;

    match parsing::parse_file(&tokens, file.clone())
    {
        Ok(Some(ok)) => Ok(ok),
        Ok(None) => {
            let error = ParserError::ExpectedToken(TokenType::EOF, TextPos::uniform(0).get_loc(&file)).format_error();
            Err(vec![error])
        }
        Err(err) => {
            Err(err.iter().map(|e| e.format_error()).collect())
        }
    }
}

pub fn run_expression_parser(file: &FileInfo) -> Result<Expression, Vec<String>>
{
    let tokens = run_lexer(file)?;
    let mut reader = TokenReader::new(&tokens, file, None);
    match parsing::expect_expression(&mut reader, parsing::parse_expression)
    {
        Ok(ok) => Ok(ok),
        Err(err) => {
            Err(vec![err.format_error()])
        }
    }
}

pub fn run_block_parser(file: &FileInfo) -> Result<BlockStmt, Vec<String>>
{
    let tokens = run_lexer(file)?;
    let mut reader = TokenReader::new(&tokens, file, None);
    match parsing::expect_block_stmt(&mut reader)
    {
        Ok(ok) => Ok(ok),
        Err(err) => {
            Err(vec![err.format_error()])
        }
    }
}

pub fn run_validator(program: &[Arc<FileInfo>]) -> Result<CheckedProgram, Vec<String>>
{
    let mut errors = vec![];
    let mut files = vec![];
    for f in program.iter()
    {
        match run_parser(f.clone())
        {
            Ok(ok) => files.push(Arc::new(ok)),
            Err(e) => errors.extend(e),
        }
    }

    if errors.len() > 0
    {
        return Err(errors);
    }

    let program = Program {
        files
    };

    let checked = CheckedProgram::new(&program);
    match checked
    {
        Ok(ok) => Ok(ok),
        Err(e) => Err(e.iter().map(|e| e.format_error()).collect())
    }
}