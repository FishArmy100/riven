use std::sync::Arc;

use either::Either;

use crate::lexing::token::{Token, TokenType, ASSIGNMENT_TOKENS};
use super::ast::*;

use super::{expect_ast_item, expect_block_expression, expect_expression, expect_let_condition, expect_type_name, is_expression_and, parse_block_expression, parse_expression, parse_generic_params, parse_if, parse_match, pattern_parsing::expect_pattern, token_reader::TokenReader, ParserError, ParserResult};

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
    else if let Some(stmt) = parse_type_decl(reader)?
    {
        Ok(Some(Statement::TypeDecl(stmt)))
    }
    else if let Some(stmt) = parse_enum_decl(reader)?
    {
        Ok(Some(Statement::EnumDecl(stmt)))
    }
    else if let Some(stmt) = parse_struct_decl(reader)?
    {
        Ok(Some(Statement::StructDecl(stmt)))
    }
    else if let Some(stmt) = parse_fn_decl(reader)?
    {
        Ok(Some(Statement::FnDecl(stmt)))
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
    else if let Some(stmt) = parse_match(reader)?
    {
        Ok(Some(Statement::Match(stmt)))
    }
    else if let Some(stmt) = parse_block_expression(reader)?
    {
        Ok(Some(Statement::Block(stmt)))
    }
    else if let Some(stmt) = parse_expression_stmt(reader)?
    {
        Ok(Some(Statement::Expression(stmt)))
    }
    else if let Some(stmt) = parse_use_stmt(reader)?
    {
        Ok(Some(Statement::Use(stmt)))
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
    let pub_tok = reader.check(TokenType::Pub);

    if let Some(stmt) = parse_fn_decl(reader)?
    {
        return Ok(Some(Declaration::Fn(pub_tok, Arc::new(stmt))));
    }
    
    if let Some(stmt) = parse_struct_decl(reader)?
    {
        return Ok(Some(Declaration::Struct(pub_tok, Arc::new(stmt))));
    }

    if let Some(stmt) = parse_interface_decl(reader)?
    {
        return Ok(Some(Declaration::Interface(pub_tok, Arc::new(stmt))));
    }

    if let Some(stmt) = parse_enum_decl(reader)?
    {
        return Ok(Some(Declaration::Enum(pub_tok, Arc::new(stmt))));
    }

    if let Some(stmt) = parse_type_decl(reader)?
    {
        return Ok(Some(Declaration::Type(pub_tok, Arc::new(stmt))));
    }

    if let Some(stmt) = parse_let(reader)?
    {
        return Ok(Some(Declaration::Let(pub_tok, Arc::new(stmt))));
    }

    if let Some(stmt) = parse_use_stmt(reader)?
    {
        return Ok(Some(Declaration::Use(pub_tok, Arc::new(stmt))));
    }

    if let Some(stmt) = parse_impl_stmt(reader)?
    {
        return Ok(Some(Declaration::Impl(Arc::new(stmt))));
    }

    if pub_tok.is_some()
    {
        return Err(ParserError::ExpectedDeclaration(reader.current()))
    }

    Ok(None)
}

fn parse_while(reader: &mut TokenReader) -> ParserResult<Option<WhileStmt>>
{
    if let Some(while_tok) = reader.check(TokenType::While)
    {
        let condition = expect_let_condition(reader)?;
        let body = expect_block_expression(reader)?;
        Ok(Some(WhileStmt { while_tok, condition, body }))
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

    let pattern = expect_pattern(reader)?;
    let in_tok = reader.expect(TokenType::In)?;
    let expression = expect_expression(reader, parse_expression)?;
    let body = expect_block_expression(reader)?;

    Ok(Some(ForStmt { for_tok, pattern, in_tok, expression, body }))
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

fn parse_impl_stmt(reader: &mut TokenReader) -> ParserResult<Option<ImplStmt>>
{
    let Some(impl_tok) = reader.check(TokenType::Impl) else {
        return Ok(None)
    };

    let generic_params = parse_generic_params(reader)?;
    let type_name = expect_type_name(reader)?;
    let for_clause = if let Some(for_tok) = reader.check(TokenType::For) {
        let type_name = expect_type_name(reader)?;
        Some((for_tok, type_name))
    } else { None };

    let where_clause = parse_where_clause(reader)?;

    let open_brace = reader.expect(TokenType::OpenBrace)?;

    let mut members = vec![];
    while let Some(member) = parse_impl_member(reader)?
    {
        members.push(member);
    }

    let close_brace = reader.expect(TokenType::CloseBrace)?;

    Ok(Some(ImplStmt { 
        impl_tok, 
        generic_params, 
        type_name, 
        for_clause, 
        where_clause, 
        open_brace, 
        members, 
        close_brace
    }))
}

fn parse_impl_member(reader: &mut TokenReader) -> ParserResult<Option<(Option<Token>, Statement)>>
{
    let pub_tok = reader.check(TokenType::Pub);

    let member = if let Some(let_stmt) = parse_let(reader)?
    {
        Some(Statement::Let(let_stmt))
    }
    else if let Some(fn_decl) = parse_fn_decl(reader)?
    {
        Some(Statement::FnDecl(fn_decl))
    }
    else if let Some(type_decl) = parse_type_decl(reader)?
    {
        Some(Statement::TypeDecl(type_decl))
    }
    else 
    {
        None  
    };

    if pub_tok.is_some() && member.is_none()
    {
        Err(ParserError::ExpectedTokens(vec![TokenType::Let, TokenType::Fn, TokenType::Type], reader.current()))
    }
    else if let Some(member) = member
    {
        Ok(Some((pub_tok, member)))    
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_type_decl(reader: &mut TokenReader) -> ParserResult<Option<TypeDecl>>
{
    let Some(type_tok) = reader.check(TokenType::Type) else {
        return  Ok(None);
    };

    let id = reader.expect(TokenType::Identifier)?;
    let generic_params = parse_generic_params(reader)?;
    let equal = reader.expect(TokenType::Equal)?;
    let type_name = expect_type_name(reader)?;
    let semi_colon = reader.expect(TokenType::SemiColon)?;

    Ok(Some(TypeDecl { 
        type_tok, 
        id, 
        generic_params, 
        equal, 
        type_name, 
        semi_colon
    }))
}

fn parse_enum_decl(reader: &mut TokenReader) -> ParserResult<Option<EnumDecl>>
{
    let Some(enum_tok) = reader.check(TokenType::Enum) else { return Ok(None); };
    let id = reader.expect(TokenType::Identifier)?;
    let generic_params = parse_generic_params(reader)?;
    let where_clause = parse_where_clause(reader)?;
    let open_brace = reader.expect(TokenType::OpenBrace)?;

    let mut members = vec![];
    while let Some(member) = parse_enum_member(reader)?
    {
        members.push(member);
        if reader.check(TokenType::Comma).is_none() { break; }
    }

    let close_brace = reader.expect(TokenType::CloseBrace)?;

    Ok(Some(EnumDecl { 
        enum_tok, 
        id, 
        generic_params, 
        where_clause, 
        open_brace, 
        members, 
        close_brace 
    }))
}

fn parse_enum_member(reader: &mut TokenReader) -> ParserResult<Option<EnumMember>>
{
    let Some(id) = reader.check(TokenType::Identifier) else {
        return Ok(None)
    };

    if let Some(open_brace) = reader.check(TokenType::OpenBrace)
    {
        let mut members = vec![];
        while let Some(member) = parse_enum_struct_member(reader)?
        {
            members.push(member);
            if reader.check(TokenType::Comma).is_none() { break; }
        }

        let close_brace = reader.expect(TokenType::CloseBrace)?;
        
        Ok(Some(EnumMember::Struct { id, open_brace, members, close_brace }))
    }
    else if let Some(open_paren) = reader.check(TokenType::OpenParen)
    {
        let type_name = expect_type_name(reader)?;
        let close_paren = reader.expect(TokenType::CloseParen)?;
        Ok(Some(EnumMember::Single { id, open_paren, type_name, close_paren }))
    }
    else 
    {
        Ok(Some(EnumMember::Basic(id)))    
    }
}

fn parse_enum_struct_member(reader: &mut TokenReader) -> ParserResult<Option<EnumStructMember>>
{
    if !reader.current_is(&[TokenType::Mut, TokenType::Identifier]) { return Ok(None) }

    let mut_tok = reader.check(TokenType::Mut);
    let id = reader.expect(TokenType::Identifier)?;

    let colon = reader.expect(TokenType::Colon)?;
    let type_name = expect_type_name(reader)?;

    let initializer = if let Some(equal) = reader.check(TokenType::Equal) {
        let expression = expect_expression(reader, parse_expression)?;
        Some((equal, expression))
    } else { None };

    Ok(Some(EnumStructMember { 
        mut_tok, 
        id,
        colon, 
        type_name, 
        initializer 
    }))
}

fn parse_interface_decl(reader: &mut TokenReader) -> ParserResult<Option<InterfaceDecl>>
{
    let Some(interface_tok) = reader.check(TokenType::Interface) else { return Ok(None); };
    let id = reader.expect(TokenType::Identifier)?;
    let generic_params = parse_generic_params(reader)?;
    let where_clause = parse_where_clause(reader)?;
    let open_brace = reader.expect(TokenType::OpenBrace)?;

    let mut members = vec![];
    while let Some(member) = parse_fn_decl(reader)?
    {
        members.push(Statement::FnDecl(member));
        if reader.check(TokenType::Comma).is_none() { break; }
    }

    let close_brace = reader.expect(TokenType::CloseBrace)?;

    Ok(Some(InterfaceDecl { 
        interface_tok, 
        id, 
        generic_params, 
        where_clause, 
        open_brace, 
        members, 
        close_brace 
    }))
}

fn parse_struct_decl(reader: &mut TokenReader) -> ParserResult<Option<StructDecl>>
{
    let Some(struct_tok) = reader.check(TokenType::Struct) else { return Ok(None); };
    let id = reader.expect(TokenType::Identifier)?;
    let generic_params = parse_generic_params(reader)?;
    let where_clause = parse_where_clause(reader)?;
    let open_brace = reader.expect(TokenType::OpenBrace)?;

    let mut members = vec![];
    while let Some(member) = parse_struct_member(reader)?
    {
        members.push(member);
        if reader.check(TokenType::Comma).is_none() { break; }
    }

    let close_brace = reader.expect(TokenType::CloseBrace)?;

    Ok(Some(StructDecl { 
        struct_tok, 
        id, 
        generic_params, 
        where_clause, 
        open_brace, 
        members, 
        close_brace 
    }))
}

fn parse_struct_member(reader: &mut TokenReader) -> ParserResult<Option<StructMember>>
{
    if !reader.current_is(&[TokenType::Pub, TokenType::Mut, TokenType::Identifier]) {
        return Ok(None)
    }

    let pub_tok = reader.check(TokenType::Pub);
    let mut_tok = reader.check(TokenType::Mut);
    let id = reader.expect(TokenType::Identifier)?;

    let colon = reader.expect(TokenType::Colon)?;
    let type_name = expect_type_name(reader)?;

    let initializer = if let Some(equal) = reader.check(TokenType::Equal) {
        let expression = expect_expression(reader, parse_expression)?;
        Some((equal, expression))
    } else { None };

    Ok(Some(StructMember { 
        pub_tok, 
        mut_tok, 
        id,
        colon, 
        type_name, 
        initializer 
    }))
}

fn parse_fn_decl(reader: &mut TokenReader) -> ParserResult<Option<FnDecl>>
{
    let Some(fn_tok) = reader.check(TokenType::Fn) else { return Ok(None) };
    let id = reader.expect(TokenType::Identifier)?;
    let generic_params = parse_generic_params(reader)?;

    let open_paren = reader.expect(TokenType::OpenParen)?;
    let mut params = vec![];
    while let Some(param) = parse_fn_param(reader)?
    {
        params.push(param);
        if reader.check(TokenType::Comma).is_none() { break; }
    }
    let close_paren = reader.expect(TokenType::CloseParen)?;

    let arrow = reader.expect(TokenType::ThinArrow)?;
    let return_type = expect_type_name(reader)?;
    let where_clause = parse_where_clause(reader)?;
    let body = if let Some(semi_colon) = reader.check(TokenType::SemiColon) 
    {
        Either::Right(semi_colon)
    }
    else 
    {
        Either::Left(expect_block_expression(reader)?)
    };

    Ok(Some(FnDecl { 
        fn_tok, 
        id, 
        generic_params, 
        open_paren, 
        params, 
        close_paren, 
        arrow, 
        return_type, 
        where_clause, 
        body 
    }))
}

fn parse_fn_param(reader: &mut TokenReader) -> ParserResult<Option<FnParam>>
{
    if reader.is_sequence(&[TokenType::SelfVal]) || reader.is_sequence(&[TokenType::Mut, TokenType::SelfVal])
    {
        let mut_tok = reader.check(TokenType::Mut);
        let self_tok = reader.expect(TokenType::SelfVal)?;

        Ok(Some(FnParam::SelfParam { mut_tok, self_tok }))
    }
    else if reader.is_sequence(&[TokenType::Identifier]) || reader.is_sequence(&[TokenType::Mut, TokenType::Identifier])
    {
        let mut_tok = reader.check(TokenType::Mut);
        let id = reader.expect(TokenType::Identifier)?;
        let colon = reader.expect(TokenType::Colon)?;
        let type_name = expect_type_name(reader)?;

        let default_value = if let Some(equal) = reader.check(TokenType::Equal) {
            let expression = expect_expression(reader, parse_expression)?;
            Some((equal, expression))
        } else { None };

        Ok(Some(FnParam::Normal { 
            mut_tok, 
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

fn parse_where_clause(reader: &mut TokenReader) -> ParserResult<Option<WhereClause>>
{
    let Some(where_tok) = reader.check(TokenType::Where) else {
        return Ok(None)
    };

    let mut sub_clauses = vec![parse_where_sub_clause(reader)?];
    while let Some(_comma) = reader.check(TokenType::Comma)
    {
        if !reader.current_is(&[TokenType::Identifier]) { break; }
        sub_clauses.push(parse_where_sub_clause(reader)?);
    }

    Ok(Some(WhereClause { where_tok, sub_clauses }))
}

fn parse_where_sub_clause(reader: &mut TokenReader) -> ParserResult<WhereSubClause>
{
    let id = reader.expect(TokenType::Identifier)?;
    let colon = reader.expect(TokenType::Colon)?;

    let mut types = vec![expect_type_name(reader)?];
    while let Some(_plus) = reader.check(TokenType::Plus)
    {
        types.push(expect_type_name(reader)?);
    }

    Ok(WhereSubClause { id, colon, types })
}

fn parse_let(reader: &mut TokenReader) -> ParserResult<Option<LetStmt>>
{
    if let Some(let_tok) = reader.check(TokenType::Let)
    {
        let pattern = expect_pattern(reader)?;

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
        let else_clause = if let Some(else_tok) = reader.check(TokenType::Else) 
        {
            let block = expect_block_expression(reader)?;
            Some((else_tok, block))
        }
        else 
        {
            None
        };

        let semi_colon = reader.expect(TokenType::SemiColon)?;

        if let Pattern::Identifier { mut_tok, id } = pattern
        {
            Ok(Some(LetStmt {
                let_tok,
                binding: LetBinding::Variable { mut_tok, id },
                type_name,
                equal,
                expression,
                else_clause,
                semi_colon,
            }))
        }
        else 
        {
            Ok(Some(LetStmt {
                let_tok,
                binding: LetBinding::Pattern(pattern),
                type_name,
                equal,
                expression,
                else_clause,
                semi_colon,
            }))
        }
    }
    else 
    {
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

fn parse_use_stmt(reader: &mut TokenReader) -> ParserResult<Option<UseStmt>>
{
    if let Some(use_tok) = reader.check(TokenType::Use)
    {
        let mut ids = vec![];
        let mut has_dot = false;
        while let Some(id) = reader.check(TokenType::Identifier)
        {
            ids.push(id);
            has_dot = reader.check(TokenType::Dot).is_some()
        }

        if ids.len() == 0
        {
            return Err(ParserError::ExpectedToken(TokenType::Identifier, reader.current()));
        }

        let star = if has_dot { Some(reader.expect(TokenType::Multiply)?) } else { None };
        let semi_colon = reader.expect(TokenType::SemiColon)?;

        Ok(Some(UseStmt {
            use_tok,
            ids,
            star,
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