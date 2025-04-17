use crate::lexing::token::TokenType;
use super::ast::*;

use super::{parse_generic_args, token_reader::TokenReader, ParserError, ParserResult};

pub fn peek_type(reader: &TokenReader) -> Option<(TypeName, usize)>
{
    let Some(mut type_reader) = TokenReader::new(reader.tokens(), Some(reader.index())) else {
        return None;
    };

    match parse_type_name(&mut type_reader)
    {
        Ok(Some(t)) => Some((t, type_reader.index() - reader.index())),
        _ => None
    }
}

pub fn is_type(reader: &TokenReader) -> Option<usize>
{
    let Some(mut type_reader) = TokenReader::new(reader.tokens(), Some(reader.index())) else {
        return None;
    };

    match parse_type_name(&mut type_reader)
    {
        Ok(Some(_)) => Some(type_reader.index() - reader.index()),
        _ => None
    }
}

pub fn is_type_and<F>(reader: &TokenReader, f: F) -> Option<usize>
    where F : Fn(&TypeName) -> bool
{
    let Some(mut type_reader) = TokenReader::new(reader.tokens(), Some(reader.index())) else {
        return None;
    };

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
    let Some(mut inner) = match reader.current().map(|c| c.token_type)
    {
        Some(TokenType::Identifier) | Some(TokenType::SelfType) => 
        {
            let identifier = reader.advance().unwrap();
            let args = parse_generic_args(reader)?;
            Ok(Some(TypeName::Identifier{
                name: identifier,
                args,
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
        Some(TokenType::Fn) =>
        {
            Ok(Some(parse_fn_type(reader)?))
        },
        _ => return Ok(None),
    }? else { return Ok(None) };

    while let Some(dot) = reader.check(TokenType::Dot)
    {
        let name = reader.expect(TokenType::Identifier)?;
        let args = parse_generic_args(reader)?;
        inner = TypeName::Access { inner: Box::new(inner), dot, name, args }
    }

    Ok(Some(inner))
}

fn parse_fn_type(reader: &mut TokenReader) -> ParserResult<TypeName>
{
    let fn_tok = reader.expect(TokenType::Fn)?;
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

    let arrow = reader.expect(TokenType::ThinArrow)?;
    let Some(return_type) = parse_type_name(reader)? else {
        return Err(ParserError::ExpectedType(reader.current()))
    };

    Ok(TypeName::Function { 
        fn_tok, 
        open_paren, 
        parameter_types, 
        close_paren, 
        arrow, 
        return_type: Box::new(return_type)
    })
}