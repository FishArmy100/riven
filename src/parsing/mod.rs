pub mod token_reader;
pub mod type_parsing;
pub mod expr_parsing;
pub mod stmt_parsing;
pub mod ast;

use stmt_parsing::{parse_declaration, parse_use_stmt};
pub use type_parsing::*;
pub use expr_parsing::*;

use token_reader::TokenReader;
use crate::{compiler::CompilerError, lexing::token::{Token, TokenType}};
use self::ast::*;
use crate::utils::TextPos;

#[derive(Debug, Clone)]
pub enum ParserError
{
    ExpectedExpression(Option<Token>),
    ExpectedType(Option<Token>),
    ExpectedToken(TokenType, Option<Token>),
    ExpectedTokens(Vec<TokenType>, Option<Token>),
    ExpectedALambdaParameter(Option<Token>),
    ExpectedALambdaBody(Option<Token>),
    ExpectedStatement(Option<Token>),
    ExpectedBlock(Option<Token>),
    ExpectedDeclaration(Option<Token>),
}

impl CompilerError for ParserError
{
    fn pos(&self) -> Option<TextPos> 
    {
        match self 
        {
            ParserError::ExpectedExpression(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedType(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedToken(_, token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedTokens(_, token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedALambdaParameter(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedStatement(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedBlock(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedDeclaration(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedALambdaBody(token) => token.as_ref().map(|t| t.pos),
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

pub fn parse_file(tokens: &Vec<Token>) -> Result<Option<FileNode>, Vec<ParserError>>
{
    let Some(mut reader) = TokenReader::new(tokens, None) else { return Ok(None) };
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
                reader.synchronize(&[TokenType::EOF, TokenType::Use, TokenType::Const, TokenType::Fn, TokenType::Struct]);
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
                reader.synchronize(&[TokenType::EOF, TokenType::Const, TokenType::Fn, TokenType::Struct]);
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

    Ok(Some(FileNode { 
        usings, 
        declarations, 
        eof 
    }))
}

fn expect_ast_item<P, R, E>(reader: &mut TokenReader, predicate: P, error: E) -> ParserResult<R>
    where P : Fn(&mut TokenReader) -> ParserResult<Option<R>>,
          E : Fn(Option<Token>) -> ParserError
{
    match predicate(reader)?
    {
        Some(r) => Ok(r),
        None => Err(error(reader.current()))
    }
}