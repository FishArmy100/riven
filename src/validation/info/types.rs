use std::collections::HashMap;

use itertools::Itertools;
use uuid::Uuid;

use crate::{parsing::ast::{FileNode, TypeName}, utils::FileInfo, validation::{builtins::VOID_TYPE, type_error::TypeError}};

use super::{struct_info::StructInfo, TypeResolver};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo
{
    Primary(Uuid),
    Optional(Box<TypeInfo>),
    Array(Box<TypeInfo>),
    Function
    {
        args: Vec<TypeInfo>,
        returned: Box<TypeInfo>,
    }
}

impl TypeInfo
{
    pub fn from(type_name: &TypeName, resolver: &TypeResolver, file: &FileNode) -> Result<Self, TypeError>
    {
        match type_name
        {
            TypeName::Identifier(token) => {
                let id = resolver.resolve_result(token, file)?;
                Ok(Self::Primary(id.clone()))
            },
            TypeName::Array { open_bracket: _, close_bracket: _, type_name } => {
                let inner = Self::from(type_name, resolver, file)?;
                Ok(Self::Array(Box::new(inner)))
            },
            TypeName::Function { fn_type_tok: _, open_paren: _, parameter_types, close_paren: _, return_type } => {
                let args = parameter_types.iter().map(|p| TypeInfo::from(p, resolver, file)).collect::<Result<_, _>>()?;
                let returned = return_type.as_ref().map(|r| TypeInfo::from(&r.1, resolver, file));

                if let Some(Err(err)) = returned { return Err(err) }

                Ok(Self::Function { args, returned: returned.map(|r| Box::new(r.unwrap())).unwrap_or(Box::new(VOID_TYPE.clone())) })
            },
            TypeName::Optional { question_mark: _, type_name } => {
                let inner = Self::from(type_name, resolver, file)?;
                Ok(Self::Optional(Box::new(inner)))
            },
        }
    }

    pub fn pretty_print(&self, structs: &HashMap<Uuid, StructInfo>) -> String 
    {
        match self 
        {
            TypeInfo::Primary(uuid) => structs.get(uuid).unwrap().name.clone(),
            TypeInfo::Optional(type_info) => format!("?{}", type_info.pretty_print(structs)),
            TypeInfo::Array(type_info) => format!("[]{}", type_info.pretty_print(structs)),
            TypeInfo::Function { args, returned } => {
                let args = args.iter().map(|a| a.pretty_print(structs)).join(", ");
                let returned = returned.pretty_print(structs);
                format!("Fn({}) -> {}", args, returned)
            },
        }
    }

    pub fn is_fn(&self) -> bool 
    {
        match self 
        {
            TypeInfo::Function { args: _, returned: _ } => true,
            _ => false,
        }
    }
    
    pub fn is_iter(&self) -> bool
    {
        self.get_iter_type().is_some()
    }

    pub fn get_iter_type(&self) -> Option<TypeInfo>
    {
        let TypeInfo::Function { args, returned } = self else {
            return None;
        };

        if args.len() != 0
        {
            return None;
        }

        let TypeInfo::Optional(t) = returned.as_ref() else {
            return None;
        };

        Some(t.as_ref().clone())
    }
}