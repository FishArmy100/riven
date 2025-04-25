pub mod token_reader;
pub mod type_parsing;
pub mod expr_parsing;
pub mod stmt_parsing;
pub mod ast;

use std::sync::Arc;

use itertools::Itertools;
pub use type_parsing::*;
pub use expr_parsing::*;
pub use stmt_parsing::*;

use token_reader::TokenReader;
use crate::{compiler::CompilerError, lexing::token::{Token, TokenType}, utils::{FileInfo, PathInfo, TextLoc}};
use self::ast::*;

#[derive(Debug, Clone)]
pub enum ParserError
{
    ExpectedExpression(TextLoc),
    ExpectedType(TextLoc),
    ExpectedToken(TokenType, TextLoc),
    ExpectedTokens(Vec<TokenType>, TextLoc),
    ExpectedALambdaParameter(TextLoc),
    ExpectedALambdaBody(TextLoc),
    ExpectedStatement(TextLoc),
    ExpectedBlock(TextLoc),
    ExpectedDeclaration(TextLoc),
}

impl CompilerError for ParserError
{
    fn loc(&self) -> TextLoc
    {
        match self 
        {
            ParserError::ExpectedExpression(loc) => loc.clone(),
            ParserError::ExpectedType(loc) => loc.clone(),
            ParserError::ExpectedToken(_, loc) => loc.clone(),
            ParserError::ExpectedTokens(_, loc) => loc.clone(),
            ParserError::ExpectedALambdaParameter(loc) => loc.clone(),
            ParserError::ExpectedStatement(loc) => loc.clone(),
            ParserError::ExpectedBlock(loc) => loc.clone(),
            ParserError::ExpectedDeclaration(loc) => loc.clone(),
            ParserError::ExpectedALambdaBody(loc) => loc.clone(),
        }
    }

    fn msg(&self) -> String 
    {
        match self
        {
            ParserError::ExpectedExpression(_) => "Expected an expression".into(),
            ParserError::ExpectedType(_) => "Expected a type".into(),
            ParserError::ExpectedToken(token_type, _) => format!("Expected token {:?} ", token_type),
            ParserError::ExpectedTokens(token_types, _) => format!("Expected one of token {:?} ", token_types.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>()),
            ParserError::ExpectedALambdaParameter(_) => "Expected a lambda parameter".into(),
            ParserError::ExpectedStatement(_) => "Expected a statement".into(),
            ParserError::ExpectedBlock(_) => "Expected a block expression".into(),
            ParserError::ExpectedDeclaration(_) => "Expected a declaration".into(),
            ParserError::ExpectedALambdaBody(_) => "Expected a lambda body".into(),
        }
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

pub fn parse_file(tokens: &Vec<Token>, file: Arc<FileInfo>) -> Result<Option<FileNode>, Vec<ParserError>>
{
    let mut reader = TokenReader::new(tokens, &file, None);
    let mut usings = vec![];
    let mut declarations = vec![];
    let mut errors = vec![];

    loop 
    {
        match parse_use_stmt(&mut reader)
        {
            Ok(Some(decl)) => usings.push(decl),
            Ok(None) => break,
            Err(err) => {
                errors.push(err);
                reader.synchronize(&[TokenType::EOF, TokenType::Pub, TokenType::Use, TokenType::Const, TokenType::Fn, TokenType::Struct]);
            },
        }
    }

    loop 
    {
        match parse_declaration(&mut reader)
        {
            Ok(Some(decl)) => declarations.push(decl),
            Ok(None) => break,
            Err(err) => {
                errors.push(err);
                reader.synchronize(&[TokenType::EOF, TokenType::Pub, TokenType::Const, TokenType::Fn, TokenType::Struct]);
            },
        }
    }

    let eof = match reader.expect(TokenType::EOF) {
        Ok(ok) => ok,
        Err(err) => { 
            errors.push(err);
            return Err(errors);
        }
    };

    if errors.len() > 0
    {
        return Err(errors)
    }

    let mut using_paths = usings.iter()
        .map(|u| u.ids.iter()
            .map(|id| id.value_string().unwrap().clone())
            .collect_vec())
        .collect_vec();

    // a bit borked, but should work
    using_paths.push(vec![]);
    using_paths.push(file.path.split_relative());

    Ok(Some(FileNode {
        usings, 
        using_paths,
        declarations, 
        eof,
        info: file,
    }))
}

fn expect_ast_item<P, R, E>(reader: &mut TokenReader, predicate: P, error: E) -> ParserResult<R>
    where P : Fn(&mut TokenReader) -> ParserResult<Option<R>>,
          E : Fn(TextLoc) -> ParserError
{
    match predicate(reader)?
    {
        Some(r) => Ok(r),
        None => Err(error(reader.current_loc()))
    }
}