use std::sync::Arc;

use either::Either;

use crate::lexing::token::{TokenType, ASSIGNMENT_TOKENS};
use super::{ast::*, expect_ast_item, is_type};

use super::{expect_expression, expect_type_name, is_expression_and, parse_expression, token_reader::TokenReader, ParserError, ParserResult};

pub fn expect_statement(reader: &mut TokenReader) -> ParserResult<Statement>
{
    if let Some(statement) = parse_statement(reader)?
    {
        Ok(statement)
    }
    else
    {
        Err(ParserError::ExpectedStatement(reader.current()))    
    }
}

pub fn parse_statement(reader: &mut TokenReader) -> ParserResult<Option<Statement>>
{
    if let Some(stmt) = parse_while(reader)?
    {
        Ok(Some(Statement::While(stmt)))
    }
    else if let Some(stmt) = parse_for(reader)?
    {
        Ok(Some(Statement::For(stmt)))
    }
    else if let Some(stmt) = parse_return(reader)?
    {
        Ok(Some(Statement::Return(stmt)))
    }
    else if let Some(stmt) = parse_continue(reader)?
    {
        Ok(Some(Statement::Continue(stmt)))
    }
    else if let Some(stmt) = parse_break(reader)?
    {
        Ok(Some(Statement::Break(stmt)))
    }
    else if let Some(stmt) = parse_let(reader)?
    {
        Ok(Some(Statement::Let(stmt)))
    }
    else if let Some(stmt) = parse_assignment(reader)?
    {
        Ok(Some(Statement::Assign(stmt)))
    }
    else if let Some(stmt) = parse_if(reader)?
    {
        Ok(Some(Statement::If(stmt)))
    }
    else if let Some(stmt) = parse_block_stmt(reader)?
    {
        Ok(Some(Statement::BlockStmt(stmt)))
    }
    else if let Some(stmt) = parse_expression_stmt(reader)?
    {
        Ok(Some(Statement::Expression(stmt)))
    }
    else 
    {
        Ok(None)
    }
}

pub fn expect_declaration(reader: &mut TokenReader) -> ParserResult<Declaration>
{
    expect_ast_item(reader, |r| parse_declaration(r), |t| ParserError::ExpectedDeclaration(t))
}

pub fn parse_declaration(reader: &mut TokenReader) -> ParserResult<Option<Declaration>>
{
    if let Some(stmt) = parse_fn_decl(reader)?
    {
        Ok(Some(Declaration::Fn(Arc::new(stmt))))
    }
    else if let Some(stmt) = parse_struct_decl(reader)?
    {
        Ok(Some(Declaration::Struct(Arc::new(stmt))))
    }
    else if let Some(stmt) = parse_const(reader)?
    {
        Ok(Some(Declaration::Const(Arc::new(stmt))))
    }
    else 
    {
        Ok(None)   
    }
}

pub fn expect_block_stmt(reader: &mut TokenReader) -> ParserResult<BlockStmt>
{
    match parse_block_stmt(reader)?
    {
        Some(b) => Ok(b),
        None => Err(ParserError::ExpectedBlock(reader.current()))
    }
}

pub fn parse_block_stmt(reader: &mut TokenReader) -> ParserResult<Option<BlockStmt>>
{
    if let Some(open_brace) = reader.check(TokenType::OpenBrace)
    {
        let mut statements = vec![];
        while let Some(statement) = parse_statement(reader)?
        {
            statements.push(statement);
        }

        let close_brace = reader.expect(TokenType::CloseBrace)?;

        let block_expr = BlockStmt {
            open_brace,
            statements,
            close_brace,
        };

        Ok(Some(block_expr))
    }
    else 
    {
        Ok(None)    
    }
}

pub fn parse_if(reader: &mut TokenReader) -> ParserResult<Option<IfStmt>>
{
    if let Some(if_tok) = reader.check(TokenType::If)
    {
        let open_paren = reader.expect(TokenType::OpenParen)?;
        let condition = expect_expression(reader, parse_expression)?;
        let close_paren = reader.expect(TokenType::CloseParen)?;
        let block = expect_block_stmt(reader)?;

        if let Some(else_tok) = reader.check(TokenType::Else)
        {
            let else_branch = match parse_if(reader)?
            {
                Some(if_expr) => ElseBranch { 
                    else_tok, 
                    body: Either::Left(Box::new(if_expr)) 
                },
                None => ElseBranch { 
                    else_tok, 
                    body: Either::Right(expect_block_stmt(reader)?)
                }
            };

            Ok(Some(IfStmt { 
                if_tok, 
                open_paren,
                condition,
                close_paren, 
                block, 
                else_branch: Some(else_branch)
            }))
        }
        else 
        {
            Ok(Some(IfStmt { 
                if_tok, 
                open_paren,
                condition, 
                close_paren,
                block, 
                else_branch: None
            }))
        }
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_while(reader: &mut TokenReader) -> ParserResult<Option<WhileStmt>>
{
    if let Some(while_tok) = reader.check(TokenType::While)
    {
        let open_paren = reader.expect(TokenType::OpenParen)?;
        let condition = expect_expression(reader, parse_expression)?;
        let close_paren = reader.expect(TokenType::CloseParen)?;
        let body = expect_block_stmt(reader)?;
        Ok(Some(WhileStmt { 
            while_tok, 
            open_paren,
            condition, 
            close_paren,
            body: Box::new(body) 
        }))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_for(reader: &mut TokenReader) -> ParserResult<Option<ForStmt>>
{
    let Some(for_tok) = reader.check(TokenType::For) else {
        return Ok(None)
    };

    let open_paren = reader.expect(TokenType::OpenParen)?;
    let id_tok = reader.expect(TokenType::Identifier)?;
    let in_tok = reader.expect(TokenType::In)?;
    let expression = expect_expression(reader, parse_expression)?;
    let close_paren = reader.expect(TokenType::CloseParen)?;
    let body = expect_block_stmt(reader)?;

    Ok(Some(ForStmt { 
        for_tok, 
        open_paren,
        id_tok, 
        in_tok, 
        expression,
        close_paren, 
        body: Box::new(body) 
    }))
}

fn parse_break(reader: &mut TokenReader) -> ParserResult<Option<BreakStmt>>
{
    if let Some(break_tok) = reader.check(TokenType::Break)
    {
        let semi_colon = reader.expect(TokenType::SemiColon)?;
        Ok(Some(BreakStmt { break_tok, semi_colon }))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_continue(reader: &mut TokenReader) -> ParserResult<Option<ContinueStmt>>
{
    if let Some(continue_tok) = reader.check(TokenType::Continue)
    {
        let semi_colon = reader.expect(TokenType::SemiColon)?;
        Ok(Some(ContinueStmt { continue_tok, semi_colon }))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_return(reader: &mut TokenReader) -> ParserResult<Option<ReturnStmt>>
{
    if let Some(return_tok) = reader.check(TokenType::Return)
    {
        let expression = parse_expression(reader)?;
        let semi_colon = reader.expect(TokenType::SemiColon)?;
        Ok(Some(ReturnStmt { return_tok, expression, semi_colon }))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_struct_decl(reader: &mut TokenReader) -> ParserResult<Option<StructDecl>>
{
    let pub_tok = if reader.is_sequence(&[TokenType::Pub, TokenType::Struct]) { reader.advance() } else { None };

    let Some(struct_tok) = reader.check(TokenType::Struct) else { 
        if pub_tok.is_some()
        {
            return Err(ParserError::ExpectedDeclaration(reader.current()))
        }

        return Ok(None); 
    };

    let id = reader.expect(TokenType::Identifier)?;
    let open_brace = reader.expect(TokenType::OpenBrace)?;

    let mut members = vec![];
    while let Some(member) = parse_struct_member(reader)?
    {
        members.push(member);
        if reader.check(TokenType::Comma).is_none() { break; }
    }

    reader.check(TokenType::Comma);
    let close_brace = reader.expect(TokenType::CloseBrace)?;

    Ok(Some(StructDecl { 
        pub_tok,
        struct_tok, 
        id, 
        open_brace, 
        members, 
        close_brace 
    }))
}

fn parse_struct_member(reader: &mut TokenReader) -> ParserResult<Option<StructDeclMember>>
{
    if !reader.is_sequence(&[TokenType::Identifier, TokenType::Colon]) 
    {
        return Ok(None);
    }

    let id = reader.expect(TokenType::Identifier)?;

    let colon = reader.expect(TokenType::Colon)?;
    let type_name = expect_type_name(reader)?;

    let initializer = if let Some(equal) = reader.check(TokenType::Equal) {
        let expression = expect_expression(reader, parse_expression)?;
        Some((equal, expression))
    } else { None };

    Ok(Some(StructDeclMember {
        id,
        colon, 
        type_name, 
        initializer 
    }))
}

fn parse_fn_decl(reader: &mut TokenReader) -> ParserResult<Option<FnDecl>>
{
    let pub_tok = if reader.is_sequence(&[TokenType::Pub, TokenType::Struct]) { reader.advance() } else { None };

    let Some(fn_tok) = reader.check(TokenType::Fn) else { 

        if pub_tok.is_some()
        {
            return Err(ParserError::ExpectedDeclaration(reader.current()))
        }

        return Ok(None) 
    };

    let type_name = if let Some(offset) = is_type(reader) {
        if reader.peek_is(offset, TokenType::Dot)
        {
            let type_name = expect_type_name(reader)?;
            let dot = reader.expect(TokenType::Dot)?;
            Some((dot, type_name))
        }
        else { None }
    } else { None };

    let id = reader.expect(TokenType::Identifier)?;

    let open_paren = reader.expect(TokenType::OpenParen)?;
    let mut params = vec![];

    let self_param = reader.check(TokenType::SelfVal);
    if self_param.is_some() && !reader.current_is(&[TokenType::Comma, TokenType::CloseParen])
    {
        return Err(ParserError::ExpectedToken(TokenType::Comma, reader.current()))
    }

    reader.check(TokenType::Comma); // advance past the comma

    while let Some(param) = parse_fn_param(reader)?
    {
        params.push(param);
        if reader.check(TokenType::Comma).is_none() { break; }
    }
    let close_paren = reader.expect(TokenType::CloseParen)?;

    let return_type = if let Some(arrow) = reader.check(TokenType::ThinArrow) {
        let type_name = expect_type_name(reader)?;
        Some((arrow, type_name))
    } else { None };

    let body = expect_block_stmt(reader)?;

    Ok(Some(FnDecl { 
        pub_tok,
        fn_tok, 
        type_name,
        id, 
        open_paren, 
        self_param,
        params, 
        close_paren, 
        return_type,
        body 
    }))
}

fn parse_fn_param(reader: &mut TokenReader) -> ParserResult<Option<FnParam>>
{
    if let Some(id) = reader.check(TokenType::Identifier)
    {
        let colon = reader.expect(TokenType::Colon)?;
        let type_name = expect_type_name(reader)?;

        let default_value = if let Some(equal) = reader.check(TokenType::Equal) {
            let expression = expect_expression(reader, parse_expression)?;
            Some((equal, expression))
        } else { None };

        Ok(Some(FnParam {
            id, 
            colon, 
            type_name, 
            default_value
        }))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_let(reader: &mut TokenReader) -> ParserResult<Option<LetStmt>>
{
    if let Some(let_tok) = reader.check(TokenType::Let)
    {
        let id = reader.expect(TokenType::Identifier)?;

        let type_name = if let Some(colon) = reader.check(TokenType::Colon)
        {
            let type_name = expect_type_name(reader)?;
            Some((colon, type_name))
        }
        else 
        {
            None
        };

        let equal = reader.expect_many(ASSIGNMENT_TOKENS)?;
        let expression = expect_expression(reader, parse_expression)?;

        let semi_colon = reader.expect(TokenType::SemiColon)?;

        Ok(Some(LetStmt {
            let_tok,
            id,
            type_name,
            equal,
            expression,
            semi_colon,
        }))
    }
    else 
    {
        Ok(None)
    }
}

fn parse_const(reader: &mut TokenReader) -> ParserResult<Option<ConstStmt>>
{
    let pub_tok = if reader.is_sequence(&[TokenType::Pub, TokenType::Struct]) { reader.advance() } else { None };

    if let Some(const_tok) = reader.check(TokenType::Const)
    {
        let id = reader.expect(TokenType::Identifier)?;
        let colon = reader.expect(TokenType::Colon)?;
        let type_name = expect_type_name(reader)?;

        let equal = reader.expect_many(ASSIGNMENT_TOKENS)?;
        let expression = expect_expression(reader, parse_expression)?;

        let semi_colon = reader.expect(TokenType::SemiColon)?;

        Ok(Some(ConstStmt {
            pub_tok,
            const_tok,
            id,
            colon,
            type_name,
            equal,
            expression,
            semi_colon,
        }))
    }
    else 
    {
        if pub_tok.is_some()
        {
            return Err(ParserError::ExpectedDeclaration(reader.current()))
        }

        Ok(None)
    }
}

fn parse_expression_stmt(reader: &mut TokenReader) -> ParserResult<Option<ExpressionStmt>>
{
    if let Some(expression) = is_expression_and(reader, |r| r.current_is(&[TokenType::SemiColon]))
    {
        let semi_colon = reader.expect(TokenType::SemiColon)?;
        Ok(Some(ExpressionStmt {
            expression,
            semi_colon
        }))
    }
    else 
    {
        Ok(None)    
    }
}

pub fn parse_use_stmt(reader: &mut TokenReader) -> ParserResult<Option<UseStmt>>
{
    if let Some(use_tok) = reader.check(TokenType::Use)
    {
        let mut ids = vec![];
        while let Some(id) = reader.check(TokenType::Identifier)
        {
            ids.push(id);
            if reader.check(TokenType::Dot).is_none() { break; }
        }

        if ids.len() == 0
        {
            return Err(ParserError::ExpectedToken(TokenType::Identifier, reader.current()));
        }

        let semi_colon = reader.expect(TokenType::SemiColon)?;

        Ok(Some(UseStmt {
            use_tok,
            ids,
            semi_colon
        }))
    }
    else
    {
        Ok(None)    
    }
}


fn parse_assignment(reader: &mut TokenReader) -> ParserResult<Option<AssignStmt>>
{
    if let Some(value) = is_expression_and(reader, |r| r.current_is(ASSIGNMENT_TOKENS))
    {
        let equal = reader.advance().unwrap();
        let expression = expect_expression(reader, parse_expression)?;
        let semi_colon = reader.expect(TokenType::SemiColon)?;

        Ok(Some(AssignStmt {
            value,
            equal,
            expression,
            semi_colon
        }))
    }
    else 
    {
        Ok(None)
    }
}