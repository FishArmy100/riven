pub mod expr;
pub mod stmt;
pub use expr::*;
use itertools::Itertools;
pub use stmt::*;

use crate::{lexing::token::Token, utils::TextPos};

#[derive(Debug, Clone)]
pub enum TypeName
{
    Identifier(Token),
    Array
    {
        open_bracket: Token,
        close_bracket: Token,
        type_name: Box<TypeName>,
    },
    Function 
    {
        fn_type_tok: Token,
        open_paren: Token,
        parameter_types: Vec<TypeName>,
        close_paren: Token,
        arrow: Token,
        return_type: Box<TypeName>,
    },
    Optional 
    {
        question_mark: Token,
        type_name: Box<TypeName>,
    }
}

impl TypeName
{
    pub fn pretty_print(&self) -> String 
    {
        match self
        {
            TypeName::Identifier(name) =>
            {
                name.value_string().unwrap().clone()
            },
            TypeName::Array { open_bracket: _, close_bracket: _, type_name } => 
            {
                format!("[]{}", type_name.pretty_print())
            },
            TypeName::Function { fn_type_tok: _, open_paren: _, parameter_types, close_paren: _, arrow: _, return_type } => 
            {
                format!("Fn({}) -> {}", parameter_types.iter().map(|t| t.pretty_print()).join(", "), return_type.pretty_print())
            },
            TypeName::Optional { question_mark: _, type_name } => 
            {
                format!("?{}", format!("[]{}", type_name.pretty_print()))
            }
        }
    }

    pub fn get_pos(&self) -> TextPos
    {
        match self 
        {
            TypeName::Identifier(name) => 
            {
                name.pos
            },
            TypeName::Array { open_bracket, close_bracket: _, type_name } => open_bracket.pos + type_name.get_pos(),
            TypeName::Function { fn_type_tok, open_paren: _, parameter_types: _, close_paren: _, arrow: _, return_type } => fn_type_tok.pos + return_type.get_pos(),
            TypeName::Optional { question_mark, type_name  } => 
            {
                question_mark.pos + type_name.get_pos()
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameters
{
    pub open_paren: Token,
    pub parameters: Vec<Parameter>,
    pub close_paren: Token,
}

#[derive(Debug, Clone)]
pub struct Parameter 
{
    pub var: Option<Token>,
    pub name: Token,
    pub colon: Token,
    pub type_name: TypeName,
    pub equal: Option<Token>,
    pub expression: Option<Expression>,
}