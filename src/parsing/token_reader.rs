use crate::lexing::token::{Token, TokenType};

use super::{ParserError, ParserResult};

#[derive(Debug, Clone)]
pub struct TokenReader<'a>
{
    tokens: &'a [Token],
    index: usize,
}

impl<'a> TokenReader<'a>
{
    pub fn new(tokens: &'a [Token], start_index: Option<usize>) -> Option<Self>
    {
        if tokens.len() == 0 || start_index.is_some_and(|s| s >= tokens.len()) { return None };

        Some(Self {
            tokens,
            index: start_index.map_or(0, |v| v)
        })
    }

    pub fn index(&self) -> usize
    {
        self.index
    }

    pub fn tokens(&self) -> &[Token]
    {
        &self.tokens
    }

    pub fn at_end(&self) -> bool
    {
        self.index >= self.tokens.len()
    }

    pub fn current(&self) -> Option<Token>
    {
        if !self.at_end()
        {
            Some(self.tokens[self.index].clone())
        }
        else 
        {
            None    
        }
    }

    pub fn current_type(&self) -> Option<TokenType>
    {
        self.current().map(|c| c.token_type)
    }

    pub fn advance(&mut self) -> Option<Token>
    {
        if !self.at_end()
        {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(token)
        }
        else 
        {
            None    
        }
    }

    pub fn peek(&self, count: usize) -> Option<Token>
    {
        if self.index + count < self.tokens.len()
        {
            Some(self.tokens[self.index + count].clone())
        }
        else 
        {
            None    
        }
    }

    pub fn peek_is(&self, count: usize, t: TokenType) -> bool
    {
        self.peek(count).is_some_and(|token| token.token_type == t)
    }

    pub fn peek_sequence_is(&self, count: usize, types: &[TokenType]) -> bool
    {
        (0..types.len()).map(|i| self.peek_is(count + i, types[i])).all(|b| b)
    }

    pub fn previous(&self) -> Option<Token>
    {
        if self.index != 0
        {
            Some(self.tokens[self.index - 1].clone())
        }
        else 
        {
            None    
        }
    }

    pub fn current_is(&self, tokens: &[TokenType]) -> bool
    {
        self.current().is_some_and(|t| tokens.contains(&t.token_type))
    }

    pub fn check(&mut self, t: TokenType) -> Option<Token>
    {
        if self.current_is(&[t])
        {
            self.advance()
        } 
        else 
        {
            None    
        }
    }

    pub fn check_many(&mut self, types: &[TokenType]) -> Option<Token>
    {
        let Some(t) = self.current() else { return None; };

        if types.contains(&t.token_type)
        {
            self.advance()
        }
        else 
        {
            None    
        }
    }

    pub fn check_sequence(&mut self, types: &[TokenType]) -> Option<Vec<Token>>
    {
        let all_match = types.iter().enumerate().map(|(i, t)| self.peek(i).is_some_and(|c| c.token_type == *t)).all(|b| b);
        if all_match
        {
            self.advance_count(types.len())
        }
        else 
        {
            None
        }
    }

    pub fn is_sequence(&mut self, types: &[TokenType]) -> bool
    {
        types.iter().enumerate().map(|(i, t)| self.peek(i).is_some_and(|c| c.token_type == *t)).all(|b| b)
    }

    pub fn advance_count(&mut self, count: usize) -> Option<Vec<Token>>
    {
        if count == 0 { return Some(vec![]); }

        if self.index + count - 1 >= self.tokens.len()
        {
            return None;
        }

        let mut tokens = vec![];
        for _ in 0..count
        {
            tokens.push(self.advance().unwrap());
        }

        Some(tokens)
    }

    pub fn expect(&mut self, t: TokenType) -> ParserResult<Token>
    {
        match self.check(t)
        {
            Some(token) => Ok(token),
            None => Err(ParserError::ExpectedToken(t, self.current()))
        }
    }

    pub fn expect_many(&mut self, tokens: &[TokenType]) -> ParserResult<Token>
    {
        match self.check_many(tokens)
        {
            Some(token) => Ok(token),
            None => Err(ParserError::ExpectedTokens(tokens.to_vec(), self.current()))
        }
    }

    pub fn synchronize(&mut self, tokens: &[TokenType])
    {
        while !self.current_is(tokens) && !self.at_end()
        {
            self.advance();
        }
    }
}