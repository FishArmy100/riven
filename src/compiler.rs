use std::collections::HashMap;

use uuid::Uuid;

use crate::{lexing::{self, token::{Token, TokenType}}, parsing::{self, ast::{Expression, FileNode}, token_reader::TokenReader, ParserError}, utils::{FileInfo, PathInfo, TextLoc, TextPos}};

pub trait CompilerError
{
    fn msg(&self) -> String;
    fn loc(&self) -> TextLoc;

    fn format_error(&self) -> String 
    {
        format!("[{}]: {}", self.loc(), self.msg())
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

pub fn run_parser(file: &FileInfo) -> Result<FileNode, Vec<String>>
{
    let tokens = run_lexer(file)?;

    match parsing::parse_file(&tokens, file)
    {
        Ok(Some(ok)) => Ok(ok),
        Ok(None) => {
            let error = ParserError::ExpectedToken(TokenType::EOF, TextPos::uniform(0).get_loc(file)).format_error();
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