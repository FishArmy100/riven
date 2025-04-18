use crate::lexing::token::TokenType;
use super::ast::*;

use super::{token_reader::TokenReader, ParserError, ParserResult};

pub fn peek_type(reader: &TokenReader) -> Option<(TypeName, usize)>
{
    let mut type_reader = TokenReader::new(reader.tokens(), Some(reader.index()));

    match parse_type_name(&mut type_reader)
    {
        Ok(Some(t)) => Some((t, type_reader.index() - reader.index())),
        _ => None
    }
}

pub fn is_type(reader: &TokenReader) -> Option<usize>
{
    let mut type_reader = TokenReader::new(reader.tokens(), Some(reader.index()));

    match parse_type_name(&mut type_reader)
    {
        Ok(Some(_)) => Some(type_reader.index() - reader.index()),
        _ => None
    }
}

pub fn is_type_and<F>(reader: &TokenReader, f: F) -> Option<usize>
    where F : Fn(&TypeName) -> bool
{
    let mut type_reader = TokenReader::new(reader.tokens(), Some(reader.index()));

    match parse_type_name(&mut type_reader)
    {
        Ok(Some(t)) => if f(&t) { Some(type_reader.index() - reader.index()) } else { None },
        _ => None
    }
}

pub fn expect_type_name(reader: &mut TokenReader) -> ParserResult<TypeName>
{
    if let Some(type_name) = parse_type_name(reader)?
    {
        Ok(type_name)
    }
    else
    {
        Err(ParserError::ExpectedType(reader.current()))    
    }
}

pub fn parse_type_name(reader: &mut TokenReader) -> ParserResult<Option<TypeName>>
{
    let Some(inner) = match reader.current().map(|c| c.token_type)
    {
        Some(TokenType::Identifier) => 
        {
            let identifier = reader.advance().unwrap();
            Ok(Some(TypeName::Identifier(identifier)))
        },
        Some(TokenType::Question) => {
            let question_mark = reader.advance().unwrap();
            let type_name = expect_type_name(reader)?;

            Ok(Some(TypeName::Optional { 
                question_mark, 
                type_name: Box::new(type_name)
            }))
        }
        Some(TokenType::OpenBracket) => 
        {
            let open_bracket = reader.advance().unwrap();
            let close_bracket = reader.expect(TokenType::CloseBracket)?;
            let type_name = match parse_type_name(reader)? {
                Some(type_name) => Box::new(type_name),
                None => return Err(ParserError::ExpectedType(reader.current()))
            };

            Ok(Some(TypeName::Array { open_bracket, close_bracket, type_name }))
        }
        Some(TokenType::FnType) =>
        {
            Ok(Some(parse_fn_type(reader)?))
        },
        _ => return Ok(None),
    }? else { return Ok(None) };

    Ok(Some(inner))
}

fn parse_fn_type(reader: &mut TokenReader) -> ParserResult<TypeName>
{
    let fn_type_tok = reader.expect(TokenType::FnType)?;
    let open_paren = reader.expect(TokenType::OpenParen)?;

    let mut parameter_types = vec![];
    while !reader.current_is(&[TokenType::CloseParen])
    {
        let Some(type_name) = parse_type_name(reader)? else {
            return Err(ParserError::ExpectedType(reader.current()));
        };

        parameter_types.push(type_name);

        if !reader.current_is(&[TokenType::CloseParen, TokenType::Comma])
        {
            return Err(ParserError::ExpectedToken(TokenType::CloseParen, reader.current()));
        }

        let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
    }

    let close_paren = reader.expect(TokenType::CloseParen)?;


    let return_type = if let Some(arrow) = reader.check(TokenType::ThinArrow) {
        let type_name = expect_type_name(reader)?;
        Some((arrow, Box::new(type_name)))
    } else { None };

    Ok(TypeName::Function { 
        fn_type_tok, 
        open_paren, 
        parameter_types, 
        close_paren, 
        return_type,
    })
}