use itertools::Itertools;
use uuid::Uuid;

use crate::{parsing::ast::TypeName, utils::FileInfo};

use super::{builtins::VOID_TYPE, TypeError, TypeLibrary, TypeResolver};

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
    pub fn from(type_name: &TypeName, resolver: TypeResolver, usings: &[Vec<String>], file: &FileInfo) -> Result<Self, TypeError>
    {
        match type_name
        {
            TypeName::Identifier(token) => {
                let id = resolver.resolve(token, usings).to_result(file)?;
                Ok(Self::Primary(id.clone()))
            },
            TypeName::Array { open_bracket: _, close_bracket: _, type_name } => {
                let inner = Self::from(type_name, resolver, usings, file)?;
                Ok(Self::Array(Box::new(inner)))
            },
            TypeName::Function { fn_type_tok: _, open_paren: _, parameter_types, close_paren: _, return_type } => {
                let args = parameter_types.iter().map(|p| TypeInfo::from(p, resolver, usings, file)).collect::<Result<_, _>>()?;
                let returned = return_type.as_ref().map(|r| TypeInfo::from(&r.1, resolver, usings, file));

                if let Some(Err(err)) = returned { return Err(err) }

                Ok(Self::Function { args, returned: returned.map(|r| Box::new(r.unwrap())).unwrap_or(Box::new(VOID_TYPE.clone())) })
            },
            TypeName::Optional { question_mark: _, type_name } => {
                let inner = Self::from(type_name, resolver, usings, file)?;
                Ok(Self::Optional(Box::new(inner)))
            },
        }
    }

    pub fn pretty_print(&self, library: &TypeLibrary) -> String 
    {
        match self 
        {
            TypeInfo::Primary(uuid) => library.get_type(uuid).name.clone(),
            TypeInfo::Optional(type_info) => format!("?{}", type_info.pretty_print(library)),
            TypeInfo::Array(type_info) => format!("[]{}", type_info.pretty_print(library)),
            TypeInfo::Function { args, returned } => {
                let args = args.iter().map(|a| a.pretty_print(library)).join(", ");
                let returned = returned.pretty_print(library);
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
}