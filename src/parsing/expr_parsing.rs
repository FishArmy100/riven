use either::Either;

use crate::lexing::token::TokenType;
use self::ast::{AccessExpr, CallExpr, Expression, IndexExpr, UnaryExpr};

use super::{stmt_parsing::parse_statement, *};

pub fn is_expression_and<F>(reader: &mut TokenReader, f: F) -> Option<Expression>
    where F : Fn(&TokenReader) -> bool
{
    let mut checker = reader.clone();
    
    match parse_expression(&mut checker)
    {
        Ok(Some(expression)) => if f(&checker) { 
            *reader = checker;
            Some(expression) 
        } 
        else 
        {
            None
        },
        _ => None
    }
}

pub fn expect_expression<F>(reader: &mut TokenReader, f: F) -> ParserResult<Expression>
    where F : Fn(&mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(expression) = f(reader)?
    {
        Ok(expression)
    }
    else
    {
        Err(ParserError::ExpectedExpression(reader.current()))    
    }
}

pub fn expect_block_expression(reader: &mut TokenReader) -> ParserResult<BlockExpr>
{
    match parse_block_expression(reader)?
    {
        Some(b) => Ok(b),
        None => Err(ParserError::ExpectedBlock(reader.current()))
    }
}

pub fn parse_expression(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(or) = parse_logical_or(reader)?
    {
        Ok(Some(or))
    }
    else if let Some(if_expr) = parse_if(reader)?
    {
        Ok(Some(Expression::IfExpr(if_expr)))
    }
    else if let Some(match_expr) = parse_match(reader)?
    {
        Ok(Some(Expression::MatchExpr(match_expr)))
    }
    else 
    {
        Ok(None)    
    }
}

pub fn parse_match(reader: &mut TokenReader) -> ParserResult<Option<MatchExpr>>
{
    if let Some(match_tok) = reader.check(TokenType::Match)
    {
        let expression = expect_expression(reader, parse_expression)?;
        let open_brace = reader.expect(TokenType::OpenBrace)?;
        let branches = parse_match_branches(reader)?;
        let close_brace = reader.expect(TokenType::CloseBrace)?;

        Ok(Some(MatchExpr { 
            match_tok, 
            expression: Box::new(expression), 
            open_brace, 
            branches, 
            close_brace
        }))
    }
    else  
    {
        Ok(None)    
    }
}

fn parse_match_branches(reader: &mut TokenReader) -> ParserResult<Vec<MatchBranch>>
{
    let mut branches = vec![];
    while !reader.current_is(&[TokenType::CloseBrace])
    {
        let branch = parse_match_branch(reader)?;
        branches.push(branch);

        if !reader.current_is(&[TokenType::CloseBrace, TokenType::Comma])
        {
            return Err(ParserError::ExpectedToken(TokenType::CloseBrace, reader.current()));
        }

        let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
    }

    Ok(branches)
}

fn parse_match_branch(reader: &mut TokenReader) -> ParserResult<MatchBranch>
{
    let pattern = expect_pattern(reader)?;
    let arrow = reader.expect(TokenType::ThickArrow)?;
    let expression = expect_expression(reader, parse_expression)?;

    Ok(MatchBranch { pattern, arrow, expression: Box::new(expression) })
}

pub fn parse_if(reader: &mut TokenReader) -> ParserResult<Option<IfExpr>>
{
    if let Some(if_tok) = reader.check(TokenType::If)
    {
        let condition = expect_let_condition(reader)?;
        let block = expect_block_expression(reader)?;

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
                    body: Either::Right(expect_block_expression(reader)?)
                }
            };

            Ok(Some(IfExpr { 
                if_tok, 
                condition, 
                block, 
                else_branch: Some(else_branch)
            }))
        }
        else 
        {
            Ok(Some(IfExpr { 
                if_tok, 
                condition, 
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

fn parse_logical_or(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    parse_binary_expr(reader, &[TokenType::AndAnd], parse_logical_and)
}

fn parse_logical_and(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    parse_binary_expr(reader, &[TokenType::PipePipe], parse_equality)
}

fn parse_equality(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    parse_binary_expr(reader, &[TokenType::EqualEqual, TokenType::BangEqual], parse_comparison)
}

fn parse_comparison(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    parse_binary_expr(reader, &[TokenType::GreaterEqual, TokenType::GreaterThan, TokenType::LessEqual, TokenType::LessThan], parse_term)
}

fn parse_term(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    parse_binary_expr(reader, &[TokenType::Plus, TokenType::Minus], parse_factor)
}

fn parse_factor(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    parse_binary_expr(reader, &[TokenType::Multiply, TokenType::Divide, TokenType::Modulus], parse_unary)
}

fn parse_unary(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(operator) = reader.check_many(&[TokenType::Bang, TokenType::Minus])
    {
        let expression = expect_expression(reader, parse_unary)?;
        Ok(Some(Expression::Unary(UnaryExpr {
            expression: Box::new(expression),
            operator
        })))
    }
    else 
    {
        parse_call(reader)    
    }
}

fn parse_call(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    let Some(callee) = parse_cast(reader)? else {
        return Ok(None)
    };

    parse_call_args(reader, callee)
}

fn parse_call_args(reader: &mut TokenReader, callee: Expression) -> ParserResult<Option<Expression>>
{
    if let Some(open_paren) = reader.check(TokenType::OpenParen)
    {
        let mut args = vec![];
        while !reader.current_is(&[TokenType::CloseParen])
        {
            let Some(expression) = parse_expression(reader)? else {
                return Err(ParserError::ExpectedExpression(reader.current()));
            };
    
            args.push(expression);
    
            if !reader.current_is(&[TokenType::CloseParen, TokenType::Comma])
            {
                return Err(ParserError::ExpectedToken(TokenType::CloseParen, reader.current()));
            }
    
            let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
        }

        let close_paren = reader.expect(TokenType::CloseParen)?;

        parse_call_args(reader, Expression::Call(CallExpr { 
            expression: Box::new(callee), 
            open_paren, 
            args, 
            close_paren 
        }))
    }
    else if let Some(open_bracket) = reader.check(TokenType::OpenBracket)
    {
        let arg = expect_expression(reader, parse_expression)?;
        let close_bracket = reader.expect(TokenType::CloseBracket)?;

        parse_call_args(reader, Expression::Index(IndexExpr {
            expression: Box::new(callee),
            open_bracket,
            indexer: Box::new(arg),
            close_bracket,
        }))
    }
    else if let Some(dot) = reader.check(TokenType::Dot)
    {
        let identifier = reader.expect(TokenType::Identifier)?;
        parse_call_args(reader, Expression::Access(AccessExpr {
            expression: Box::new(callee),
            dot,
            identifier,
        }))
    }
    else 
    {
        Ok(Some(callee))    
    }
}


fn parse_primary(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    // make sure to check for type expressions that may conflict with below examples
    if let Some(block) = parse_block_expression(reader)?
    {
        Ok(Some(Expression::BlockExpr(block)))
    }
    else if let Some(construction) = parse_construction_expression(reader)?
    {
        Ok(Some(construction))
    }
    else if let Some(grouping) = parse_grouping(reader)?
    {
        Ok(Some(grouping))
    }
    else if let Some(lambda) = parse_lambda(reader)?
    {
        Ok(Some(lambda))
    }
    else if let Some(array) = parse_array_literal(reader)?
    {
        Ok(Some(array))
    }
    else if let Some(con) = parse_enum_construction(reader)?
    {
        Ok(Some(con))
    }
    else if let Some(literal) = reader.check_many(&[
        TokenType::IntegerLiteral,
        TokenType::StringLiteral,
        TokenType::FloatLiteral,
        TokenType::Identifier,
        TokenType::SelfVal,
        TokenType::True,
        TokenType::False,
    ])
    {
        Ok(Some(Expression::Literal(literal)))
    }
    else 
    {
        Ok(None)
    }
}

pub fn parse_block_expression(reader: &mut TokenReader) -> ParserResult<Option<BlockExpr>>
{
    if let Some(open_brace) = reader.check(TokenType::OpenBrace)
    {
        let mut statements = vec![];
        while let Some(statement) = parse_statement(reader)?
        {
            statements.push(statement);
        }

        let expression = parse_expression(reader)?.map(|e| Box::new(e));
        let close_brace = reader.expect(TokenType::CloseBrace)?;

        let block_expr = BlockExpr {
            open_brace,
            statements,
            expression,
            close_brace,
        };

        Ok(Some(block_expr))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_enum_construction(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if is_type_and(reader, TypeName::is_definite).is_some()
    {
        let type_name = parse_type_name(reader)?.unwrap();
        let open_paren = reader.expect(TokenType::OpenParen)?;
        let expression = expect_expression(reader, parse_expression)?;
        let close_paren = reader.expect(TokenType::CloseParen)?;

        let con = EnumConstructionExpr {
            type_name,
            open_paren,
            expression: Box::new(expression),
            close_paren,
        };

        Ok(Some(Expression::EnumConstruction(con)))
    }
    else 
    {
        Ok(None)
    }
}

fn parse_construction_arg(reader: &mut TokenReader) -> ParserResult<ConstructionArg>
{
    let name = reader.expect(TokenType::Identifier)?;
    let colon = reader.expect(TokenType::Colon)?;
    let Some(initializer) = parse_expression(reader)? else {
        return Err(ParserError::ExpectedExpression(reader.current()));
    };

    Ok(ConstructionArg { 
        name, 
        colon, 
        value: Box::new(initializer) 
    })
}

fn parse_construction_expression(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(offset) = is_type(reader)
    {
        if reader.peek_sequence_is(offset, &[TokenType::OpenBrace, TokenType::Identifier, TokenType::Colon]) || 
           reader.peek_sequence_is(offset, &[TokenType::OpenBrace, TokenType::CloseBrace])
        {
            let type_name = parse_type_name(reader)?.unwrap();
            let open_brace = reader.expect(TokenType::OpenBrace)?;
    
            let mut args = vec![];
            while !reader.current_is(&[TokenType::CloseBrace])
            {
                args.push(parse_construction_arg(reader)?);
        
                if !reader.current_is(&[TokenType::CloseBrace, TokenType::Comma])
                {
                    return Err(ParserError::ExpectedToken(TokenType::CloseBrace, reader.current()));
                }
        
                let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
            }
    
            let close_brace = reader.expect(TokenType::CloseBrace)?;

            let expr = ConstructionExpr {
                type_name,
                open_brace,
                args,
                close_brace,
            };

            return Ok(Some(Expression::Construction(expr)));
        }
    }

    Ok(None)
}

fn parse_cast(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(expression) = parse_primary(reader)?
    {
        if let Some(as_tok) = reader.check(TokenType::As)
        {
            let type_name = expect_type_name(reader)?;
            Ok(Some(Expression::Cast(CastExpr { expression: Box::new(expression), as_tok, type_name })))
        }
        else
        {
            Ok(Some(expression))
        }
    }
    else 
    {
        Ok(None)
    }
}

fn parse_array_literal(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(open_bracket) = reader.check(TokenType::OpenBracket)
    {
        let mut expressions = vec![];
        while !reader.current_is(&[TokenType::CloseBracket])
        {
            let Some(expression) = parse_expression(reader)? else {
                return Err(ParserError::ExpectedExpression(reader.current()));
            };
    
            expressions.push(expression);
    
            if !reader.current_is(&[TokenType::CloseBracket, TokenType::Comma])
            {
                return Err(ParserError::ExpectedToken(TokenType::CloseBracket, reader.current()));
            }
    
            let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
        }

        let close_bracket = reader.expect(TokenType::CloseBracket)?;
        let array_literal = ArrayLiteral {
            open_bracket,
            expressions,
            close_bracket
        };

        Ok(Some(Expression::ArrayLiteral(array_literal)))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_grouping(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(open_paren) = reader.check(TokenType::OpenParen)
    {
        let Some(expression) = parse_expression(reader)? else {
            return Err(ParserError::ExpectedExpression(reader.current()))
        };
        let expression = Box::new(expression);

        let close_paren = reader.expect(TokenType::CloseParen)?;
        let grouping = GroupingExpr { open_paren, expression, close_paren };
        Ok(Some(Expression::Grouping(grouping)))
    }
    else 
    {
        Ok(None)
    }
}

fn parse_lambda_param(reader: &mut TokenReader) -> ParserResult<Option<LambdaParam>>
{
    let Some(name) = reader.check(TokenType::Identifier) else {
        return Ok(None)
    };

    if let Some(colon) = reader.check(TokenType::Colon)
    {
        let Some(type_name) = parse_type_name(reader)? else {
            return Err(ParserError::ExpectedType(reader.current()))
        };
        Ok(Some(LambdaParam { 
            name, 
            colon: Some(colon), 
            type_name: Some(type_name) 
        }))
    }
    else 
    {
        Ok(Some(LambdaParam { 
            name, 
            colon: None, 
            type_name: None 
        }))
    }
}

fn parse_lambda_params(reader: &mut TokenReader) -> ParserResult<LambdaParams>
{
    let open_pipe = reader.expect(TokenType::Pipe)?;
    let mut parameters = vec![];
    while reader.check(TokenType::Pipe).is_none()
    {
        let Some(param) = parse_lambda_param(reader)? else {
            return Err(ParserError::ExpectedALambdaParameter(reader.current()));
        };

        parameters.push(param);

        if !reader.current_is(&[TokenType::Pipe, TokenType::Comma])
        {
            return Err(ParserError::ExpectedToken(TokenType::Pipe, reader.current()));
        }

        let _ = reader.check(TokenType::Comma); // makes sure to skip the comma
    }

    let close_pipe = reader.previous().unwrap();

    let arrow = reader.check(TokenType::ThinArrow);
    let return_type = if arrow.is_some() 
    { 
        let Some(type_name) = parse_type_name(reader)? else {
            return Err(ParserError::ExpectedType(reader.current()))
        };

        Some(type_name)
    }
    else 
    {
        None
    };

    Ok(LambdaParams::Complex { open_pipe, parameters, close_pipe, arrow, return_type })
}

fn parse_lambda(reader: &mut TokenReader) -> ParserResult<Option<Expression>>
{
    if let Some(tokens) = reader.check_sequence(&[TokenType::Identifier, TokenType::ThickArrow])
    {
        let name = tokens[0].clone();
        let arrow = tokens[1].clone();
        let Some(expression) = parse_expression(reader)? else {
            return Err(ParserError::ExpectedExpression(reader.current()));
        };

        let params = LambdaParams::Simple(name);
        return Ok(Some(Expression::Lambda(LambdaExpr {
            params,
            arrow,
            expression: Box::new(expression)
        })));
    }

    if reader.current_is(&[TokenType::Pipe])
    {
        let params = parse_lambda_params(reader)?;

        let arrow = reader.expect(TokenType::ThickArrow)?;
        let Some(expression) = parse_expression(reader)? else {
            return Err(ParserError::ExpectedExpression(reader.current()));
        };

        Ok(Some(Expression::Lambda(LambdaExpr {
            params,
            arrow,
            expression: Box::new(expression)
        })))
    }
    else 
    {
        Ok(None)    
    }
}

fn parse_binary_expr<F>(reader: &mut TokenReader, tokens: &[TokenType], previous: F) -> ParserResult<Option<Expression>>
    where F : Fn(&mut TokenReader) -> ParserResult<Option<Expression>> + Copy
{
    let Some(mut left) = previous(reader)? else {
        return Ok(None)
    };

    while let Some(operator) = reader.check_many(tokens)
    {
        let right = expect_expression(reader, previous)?;
        left = Expression::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    Ok(Some(left))
}
