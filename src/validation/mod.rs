pub mod ast;

use std::collections::HashMap;

use itertools::Itertools;
use uuid::Uuid;

use crate::{compiler::CompilerError, lexing::token::{Token, TokenType}, parsing::ast::{Declaration, FileNode, TypeName}, utils::TextPos};

#[derive(Debug, Clone)]
pub enum TypeError
{
    UnknownType(Token),
    DuplicateTypeDef(Token),
}

impl CompilerError for TypeError
{
    fn msg(&self) -> String 
    {
        match self 
        {
            TypeError::UnknownType(token) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::DuplicateTypeDef(token) => format!("Duplicate type {}", token.value_string().unwrap()),
        }
    }

    fn pos(&self) -> Option<TextPos> 
    {
        match self 
        {
            TypeError::UnknownType(token) => Some(token.pos),
            TypeError::DuplicateTypeDef(token) => Some(token.pos),
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

#[derive(Debug)]
pub struct StructMember
{
    pub name: String,
    pub type_info: TypeInfo,
    pub expression: Option<()>,
}

#[derive(Debug)]
pub struct StructDef
{
    pub id: Uuid,
    pub name: String,
    pub members: Vec<StructMember>,
    pub is_pub: bool,
}

impl StructDef
{
    pub fn get_defs(file: &FileNode) -> Result<Vec<StructDef>, Vec<TypeError>>
    {
        let names = find_struct_names(&file)?;
        let decls = file.declarations.iter().filter_map(|d| match d {
            Declaration::Struct(s) => Some(s),
            _ => None,
        }).map(|d| {
            let name = d.id.value_string().unwrap().clone();
            let id = names.get(&name).unwrap().clone();
            let members = d.members.iter().map(|m| -> Result<StructMember, TypeError> {
                let name = m.id.value_string().unwrap().clone();
                let type_info = TypeInfo::from(&m.type_name, &names)?;
                let expression = None;
                Ok(StructMember {
                    name,
                    type_info,
                    expression,
                })
            }).collect_vec();

            let errors = members.iter().filter_map(|r| r.as_ref().err().cloned()).collect_vec();
            if errors.len() > 0
            {
                Err(errors)
            }
            else 
            {
                Ok(StructDef {
                    name,
                    id,
                    members: members.into_iter().filter_map(|m| m.ok()).collect_vec(),
                    is_pub: d.pub_tok.is_some()
                })    
            }
        }).collect_vec();

        let errors = decls.iter().flat_map(|d| d.as_ref().err()).flatten().map(|e| e.clone()).collect_vec();
        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(decls.into_iter().flat_map(|d| d.ok()).collect_vec())
        }
    }
}

fn find_struct_names(node: &FileNode) -> Result<HashMap<String, Uuid>, Vec<TypeError>>
{
    let mut name_map = HashMap::new();
    let mut errors = Vec::new();

    node.declarations.iter().filter_map(|d| match d {
        Declaration::Struct(s) => Some(s),
        _ => None,
    }).for_each(|d| {
        let name = d.id.value_string().unwrap();
        if name_map.contains_key(name)
        {
            errors.push(TypeError::DuplicateTypeDef(d.id.clone()));
        }
        else 
        {
            name_map.insert(name.clone(), Uuid::new_v4());
        }
    });

    if errors.len() > 0
    {
        Err(errors)
    }
    else 
    {
        Ok(name_map)    
    }
}