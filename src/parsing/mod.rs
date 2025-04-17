pub mod token_reader;
pub mod type_parsing;
pub mod expr_parsing;
pub mod stmt_parsing;
pub mod pattern_parsing;
pub mod ast;

use pattern_parsing::expect_pattern;
use stmt_parsing::parse_declaration;
pub use type_parsing::*;
pub use expr_parsing::*;

use token_reader::TokenReader;
use crate::lexing::token::{Token, TokenType};
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
    ExpectedStatement(Option<Token>),
    ExpectedPattern(Option<Token>),
    ExpectedBlock(Option<Token>),
    ExpectedDeclaration(Option<Token>),
}

impl CompilerStepError for ParserError
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
            ParserError::ExpectedPattern(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedBlock(token) => token.as_ref().map(|t| t.pos),
            ParserError::ExpectedDeclaration(token) => token.as_ref().map(|t| t.pos),
        }
    }

    fn message(&self) -> String 
    {
        match self
        {
            ParserError::ExpectedExpression(_) => "Expected an expression".into(),
            ParserError::ExpectedType(_) => "Expected a type".into(),
            ParserError::ExpectedToken(token_type, _) => format!("Expected token {:?} ", token_type),
            ParserError::ExpectedTokens(token_types, _) => format!("Expected one of token {:?} ", token_types.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>()),
            ParserError::ExpectedALambdaParameter(_) => "Expected a lambda parameter".into(),
            ParserError::ExpectedStatement(_) => "Expected a statement".into(),
            ParserError::ExpectedPattern(_) => "Expected a pattern".into(),
            ParserError::ExpectedBlock(_) => "Expected a block expression".into(),
            ParserError::ExpectedDeclaration(_) => "Expected a declaration".into(),
        }
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

pub fn parse(tokens: &Vec<Token>) -> Result<Option<Program>, Vec<ParserError>>
{
    let Some(mut reader) = TokenReader::new(tokens, None) else { return Ok(None) };
    let mut declarations = vec![];
    let mut errors = vec![];

    loop 
    {
        match parse_declaration(&mut reader)
        {
            Ok(Some(decl)) => declarations.push(decl),
            Ok(None) => break,
            Err(err) => {
                errors.push(err);
                reader.synchronize(&[TokenType::EOF, TokenType::Let, TokenType::Const, TokenType::Fn, TokenType::Struct, TokenType::Impl, TokenType::Enum, TokenType::Interface]);
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

    Ok(Some(Program { declarations, eof }))
}

fn expect_let_condition(reader: &mut TokenReader) -> ParserResult<LetCondition>
{
    match parse_let_condition(reader)?
    {
        Some(ok) => Ok(ok),
        None => Err(ParserError::ExpectedExpression(reader.current()))
    }
}

fn parse_let_condition(reader: &mut TokenReader) -> ParserResult<Option<LetCondition>>
{
    if let Some(let_tok) = reader.check(TokenType::Let)
    {
        let pattern = expect_pattern(reader)?;
        let equal = reader.expect(TokenType::Equal)?;
        let expression = expect_expression(reader, parse_expression)?;

        if let Some(and) = reader.check(TokenType::AndAnd)
        {
            let Some(other_cond) = parse_let_condition(reader)? else {
                return Err(ParserError::ExpectedExpression(reader.current()));
            };

            Ok(Some(LetCondition::Pattern { 
                let_tok, 
                pattern, 
                equal, 
                expression: Box::new(expression), 
                and: Some(and), 
                other_cond: Some(Box::new(other_cond))
            }))
        }
        else 
        {
            Ok(Some(LetCondition::Pattern { 
                let_tok, 
                pattern, 
                equal, 
                expression: Box::new(expression), 
                and: None, 
                other_cond: None,
            }))  
        }
    }
    else 
    {
        let Some(expression) = parse_expression(reader)? else {
            return Ok(None);
        };

        Ok(Some(LetCondition::Expression(Box::new(expression))))
    }
}

fn parse_generic_args(reader: &mut TokenReader) -> ParserResult<Option<GenericArgs>>
{
    let Some(open_bracket) = reader.check(TokenType::OpenBracket) else {
        return Ok(None);
    };

    let mut types = vec![];
    while !reader.current_is(&[TokenType::CloseBracket])
    {
        let Some(type_name) = parse_type_name(reader)? else {
            return Err(ParserError::ExpectedType(reader.current()));
        };

        types.push(type_name);

        if !reader.current_is(&[TokenType::CloseBracket, TokenType::Comma])
        {
            return Err(ParserError::ExpectedToken(TokenType::CloseBracket, reader.current()));
        }

        let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
    }

    let close_bracket = reader.expect(TokenType::CloseBracket)?;
    let args = GenericArgs {
        open_bracket,
        args: types,
        close_bracket
    };

    Ok(Some(args))
}

fn parse_generic_params(reader: &mut TokenReader) -> ParserResult<Option<GenericParams>>
{
    let Some(open_bracket) = reader.check(TokenType::OpenBracket) else {
        return Ok(None)
    };

    let mut params = vec![];
    while let Some(id) = reader.check(TokenType::Identifier)
    {
        params.push(id);
        if !reader.check(TokenType::Comma).is_some() { break; }
    }

    let close_bracket = reader.expect(TokenType::CloseBracket)?;
    Ok(Some(GenericParams { open_bracket, params, close_bracket }))
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

