use std::collections::HashMap;

use uuid::Uuid;

use crate::{lexing::{self, token::{Token, TokenType}}, parsing::{self, ast::{Expression, FileNode}, token_reader::TokenReader, ParserError}, utils::{PathInfo, TextPos}};

pub trait CompilerError
{
    fn msg(&self) -> String;
    fn pos(&self) -> Option<(TextPos, Uuid)>;

    fn format_error(&self, files: &HashMap<Uuid, String>) -> String 
    {
        let loc = self.pos().map(|(p, f)| p.get_loc(text).to_string()).unwrap_or("None".into());
        match &file {
            Some(file) => format!("[{}:{}]: {}", file.to_string(), loc, self.msg()),
            None => format!("[{}]: \"{}\"", loc, self.msg())
        }
    }
}

pub fn run_lexer(text: &[char], file: Option<&PathInfo>) -> Result<Vec<Token>, Vec<String>>
{
    match lexing::lex_text(text)
    {
        Ok(ok) => Ok(ok),
        Err(err) => {
            Err(err.iter().map(|e| e.format_error(text, file.map(|p| p.full_path.as_str()))).collect())
        },
    }
}

pub fn run_parser(text: &[char], path: Option<PathInfo>) -> Result<FileNode, Vec<String>>
{
    let tokens = run_lexer(text, path.as_ref())?;

    match parsing::parse_file(&tokens, path.clone())
    {
        Ok(Some(ok)) => Ok(ok),
        Ok(None) => {
            let error = ParserError::ExpectedToken(TokenType::EOF, None).format_error(text, path.as_ref().map(|p| p.full_path.as_str()));
            Err(vec![error])
        }
        Err(err) => {
            Err(err.iter().map(|e| e.format_error(text, path.as_ref().map(|p| p.full_path.as_str()))).collect())
        }
    }
}

pub fn run_expression_parser(text: &[char]) -> Result<Expression, Vec<String>>
{
    let tokens = run_lexer(text, None)?;
    let mut reader = TokenReader::new(&tokens, None);
    match parsing::expect_expression(&mut reader, parsing::parse_expression)
    {
        Ok(ok) => Ok(ok),
        Err(err) => {
            Err(vec![err.format_error(text, None)])
        }
    }
}