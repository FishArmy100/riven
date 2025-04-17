pub mod expr;
pub mod stmt;
pub use expr::*;
use itertools::Itertools;
pub use stmt::*;

use crate::{lexing::token::Token, utils::TextPos};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TypeName
{
    Identifier
    {
        name: Token,
        args: Option<GenericArgs>,
    },
    Array
    {
        open_bracket: Token,
        close_bracket: Token,
        type_name: Box<TypeName>,
    },
    Function 
    {
        fn_tok: Token,
        open_paren: Token,
        parameter_types: Vec<TypeName>,
        close_paren: Token,
        arrow: Token,
        return_type: Box<TypeName>,
    },
    Access 
    {
        inner: Box<TypeName>,
        dot: Token,
        name: Token,
        args: Option<GenericArgs>,
    }
}

impl TypeName
{
    pub fn is_access(&self) -> bool
    {
        match self 
        {
            Self::Access { inner: _, dot: _, name: _, args: _ } => true,
            _ => false,
        }
    }

    pub fn is_definite(&self) -> bool
    {
        match self 
        {
            TypeName::Identifier { name: _, args } => args.as_ref().is_some_and(|a| a.args.iter().map(|a| a.is_definite()).any(|t| t)),
            TypeName::Array { open_bracket: _, close_bracket: _, type_name: _ } => true,
            TypeName::Function { fn_tok: _, open_paren: _, parameter_types: _, close_paren: _, arrow: _, return_type: _ } => true,
            TypeName::Access { inner, dot: _, name: _, args } => inner.is_definite() || args.as_ref().is_some_and(|a| a.args.iter().map(|a| a.is_definite()).any(|t| t)),
        }
    }

    pub fn pretty_print(&self) -> String 
    {
        match self
        {
            TypeName::Identifier { name, args } =>
            {
                let mut text = name.value.as_ref().unwrap().to_string();
                if let Some(args) = args
                {
                    text += &format!("[{}]", args.args.iter().map(|a| a.pretty_print()).join(", "));
                }

                text
            },
            TypeName::Array { open_bracket: _, close_bracket: _, type_name } => 
            {
                format!("[]{}", type_name.pretty_print())
            },
            TypeName::Function { fn_tok: _, open_paren: _, parameter_types, close_paren: _, arrow: _, return_type } => 
            {
                format!("fn({}) -> {}", parameter_types.iter().map(|t| t.pretty_print()).join(", "), return_type.pretty_print())
            },
            TypeName::Access { inner, dot: _, name, args } => 
            {
                let mut text = format!("{}.{}", inner.pretty_print(), name.value.as_ref().unwrap().to_string());
                if let Some(args) = args 
                {
                    text += &format!("[{}]", args.args.iter().map(|a| a.pretty_print()).join(", "));
                }

                text
            }
        }
    }

    pub fn get_pos(&self) -> TextPos
    {
        match self 
        {
            TypeName::Identifier { name, args } => 
            {
                match args
                {
                    Some(args) => name.pos + args.close_bracket.pos + args.open_bracket.pos,
                    None => name.pos,
                }
            },
            TypeName::Array { open_bracket, close_bracket: _, type_name } => open_bracket.pos + type_name.get_pos(),
            TypeName::Function { fn_tok, open_paren: _, parameter_types: _, close_paren: _, arrow: _, return_type } => fn_tok.pos + return_type.get_pos(),
            TypeName::Access { inner, dot: _, name, args } => 
            {
                match args
                {
                    Some(args) => inner.get_pos() + args.close_bracket.pos,
                    None => inner.get_pos() + name.pos,
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenericParams
{
    pub open_bracket: Token,
    pub params: Vec<Token>,
    pub close_bracket: Token,
}

#[derive(Debug, Clone)]
pub struct GenericArgs 
{
    pub open_bracket: Token,
    pub args: Vec<TypeName>,
    pub close_bracket: Token,
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

#[derive(Debug, Clone)]
pub struct PatternField
{
    pub mut_tok: Option<Token>,
    pub id: Token,
    pub colon: Option<Token>,
    pub inner: Option<Box<Pattern>>
}

#[derive(Debug, Clone)]
pub enum Pattern 
{
    Literal(Token),
    Identifier
    {
        mut_tok: Option<Token>,
        id: Token,
    },
    TypeValue
    {
        type_name: TypeName,
        dot: Token,
        id: Token,
    },
    EnumConstruct
    {
        type_name: TypeName,
        open_paren: Token,
        inner: Box<Pattern>,
        close_paren: Token,
    },
    StructConstruct
    {
        type_name: TypeName,
        open_brace: Token,
        patterns: Vec<PatternField>,
        close_brace: Token,
    },
    ArrayConstruct
    {
        open_bracket: Token,
        patterns: Vec<Pattern>,
        close_bracket: Token,
    }
}

#[derive(Debug, Clone)]
pub enum LetCondition
{
    Expression(Box<Expression>),
    Pattern 
    {
        let_tok: Token,
        pattern: Pattern,
        equal: Token,
        expression: Box<Expression>,
        and: Option<Token>,
        other_cond: Option<Box<LetCondition>>,
    }
}