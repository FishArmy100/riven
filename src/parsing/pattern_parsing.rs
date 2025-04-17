use crate::{lexing::token::TokenType, parsing::peek_type};
use super::ast::{Pattern, PatternField, TypeName};

use super::{token_reader::TokenReader, ParserError, ParserResult};

pub fn expect_pattern(reader: &mut TokenReader) -> ParserResult<Pattern>
{
    if let Some(pattern) = parse_pattern(reader)?
    {
        Ok(pattern)
    }
    else
    {
        Err(ParserError::ExpectedPattern(reader.current()))    
    }
}

pub fn parse_pattern(reader: &mut TokenReader) -> ParserResult<Option<Pattern>>
{
    if let Some((type_name, offset)) = peek_type(reader)
    {
        if reader.peek_is(offset, TokenType::OpenBrace)
        {
            let _ = reader.advance_count(offset); // move the reader to the current token
            let open_brace = reader.expect(TokenType::OpenBrace)?;
            let patterns = parse_pattern_fields(reader)?;
            let close_brace = reader.expect(TokenType::CloseBrace)?;

            return Ok(Some(Pattern::StructConstruct { type_name, open_brace, patterns, close_brace }));
        }
        
        if reader.peek_is(offset, TokenType::OpenParen)
        {
            let _ = reader.advance_count(offset); // move the reader to the current token
            let open_paren = reader.expect(TokenType::OpenParen)?;
            let inner = expect_pattern(reader)?;
            let close_paren = reader.expect(TokenType::CloseParen)?;

            return Ok(Some(Pattern::EnumConstruct { type_name, open_paren, inner: Box::new(inner), close_paren }));
        }
        
        if let TypeName::Access { inner, dot, name, args: None } = type_name
        {
            let _ = reader.advance_count(offset);
            return  Ok(Some(Pattern::TypeValue { type_name: *inner, dot, id: name }));
        }
    }

    if let Some(open_bracket) = reader.check(TokenType::OpenBracket)
    {
        let patterns = parse_array_patterns(reader)?;
        let close_bracket = reader.expect(TokenType::CloseBracket)?;

        return Ok(Some(Pattern::ArrayConstruct { open_bracket, patterns, close_bracket }));
    }

    if let Some(literal) = reader.check_many(&[TokenType::IntegerLiteral, TokenType::FloatLiteral, TokenType::StringLiteral]) {
        return Ok(Some(Pattern::Literal(literal)));
    }

    if let Some(mut_tok) = reader.check(TokenType::Mut) {
        let id = reader.expect(TokenType::Identifier)?;
        return Ok(Some(Pattern::Identifier { mut_tok: Some(mut_tok), id }))
    }

    if let Some(id) = reader.check(TokenType::Identifier) {
        return Ok(Some(Pattern::Identifier { mut_tok: None, id }));
    }

    Ok(None)
}

fn parse_array_patterns(reader: &mut TokenReader) -> ParserResult<Vec<Pattern>>
{
    let mut patterns = vec![];
    while !reader.current_is(&[TokenType::CloseBracket])
    {
        patterns.push(expect_pattern(reader)?);

        if !reader.current_is(&[TokenType::CloseBracket, TokenType::Comma])
        {
            return Err(ParserError::ExpectedToken(TokenType::CloseBracket, reader.current()));
        }

        let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
    }

    Ok(patterns)
}

fn parse_pattern_fields(reader: &mut TokenReader) -> ParserResult<Vec<PatternField>>
{
    let mut fields = vec![];
    while !reader.current_is(&[TokenType::CloseBrace])
    {
        fields.push(parse_pattern_field(reader)?);

        if !reader.current_is(&[TokenType::CloseBrace, TokenType::Comma])
        {
            return Err(ParserError::ExpectedToken(TokenType::CloseBrace, reader.current()));
        }

        let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
    }

    Ok(fields)
}

fn parse_pattern_field(reader: &mut TokenReader) -> ParserResult<PatternField>
{
    let mut_tok = reader.check(TokenType::Mut);
    let id = reader.expect(TokenType::Identifier)?;

    if let Some(colon) = reader.check(TokenType::Colon)
    {
        let pattern = expect_pattern(reader)?;
        Ok(PatternField { 
            mut_tok, 
            id, 
            colon: Some(colon), 
            inner: Some(Box::new(pattern)) 
        })
    }
    else 
    {
        Ok(PatternField {
            mut_tok,
            id,
            colon: None,
            inner: None
        })    
    }
}