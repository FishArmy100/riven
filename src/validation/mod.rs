pub mod ast;

use std::collections::HashMap;

use uuid::Uuid;

use crate::{compiler::CompilerError, lexing::token::Token, parsing::ast::{Declaration, FileNode, TypeName}, utils::TextPos};

#[derive(Debug)]
pub enum TypeError
{
    UnknownType(Token)
}

impl CompilerError for TypeError
{
    fn msg(&self) -> String 
    {
        match self 
        {
            TypeError::UnknownType(token) => format!("Unknown type {}", token.value_string().unwrap()),
        }
    }

    fn pos(&self) -> Option<TextPos> 
    {
        match self 
        {
            TypeError::UnknownType(token) => Some(token.pos),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo
{
    Primary(Uuid),
    Optional(Box<TypeInfo>),
    Array(Box<TypeInfo>),
    Function
    {
        args: Vec<TypeInfo>,
        returned: Option<Box<TypeInfo>>,
    }
}

impl TypeInfo
{
    pub fn from(type_name: &TypeName, structs: &HashMap<String, Uuid>) -> Result<Self, TypeError>
    {
        match type_name
        {
            TypeName::Identifier(token) => {
                let Some(id) = structs.get(token.value_string().unwrap()) else {
                    return Err(TypeError::UnknownType(token.clone()))
                };
                
                Ok(Self::Primary(id.clone()))
            },
            TypeName::Array { open_bracket: _, close_bracket: _, type_name } => {
                let inner = Self::from(type_name, structs)?;
                Ok(Self::Array(Box::new(inner)))
            },
            TypeName::Function { fn_type_tok: _, open_paren: _, parameter_types, close_paren: _, return_type } => {
                let args = parameter_types.iter().map(|p| TypeInfo::from(p, structs)).collect::<Result<_, _>>()?;
                let returned = return_type.as_ref().map(|r| TypeInfo::from(&r.1, structs));

                if let Some(Err(err)) = returned { return Err(err) }

                Ok(Self::Function { args, returned: returned.map(|r| Box::new(r.unwrap())) })
            },
            TypeName::Optional { question_mark: _, type_name } => {
                let inner = Self::from(type_name, structs)?;
                Ok(Self::Optional(Box::new(inner)))
            },
        }
    }
}

pub struct StructMember
{
    pub name: String,
    pub type_info: TypeInfo,
    pub expression: Option<()>,
}

pub struct StructDef
{
    pub id: Uuid,
    pub name: String,
    pub members: Vec<StructMember>
}

fn find_struct_names(node: FileNode) -> HashMap<String, Uuid>
{
    node.declarations.iter().filter_map(|d| match d {
        Declaration::Struct(s) => Some(s.id.value_string().unwrap().clone()),
        _ => None,
    }).map(|n| (n, Uuid::new_v4())).collect()
}