use std::collections::HashMap;

use uuid::Uuid;

use crate::parsing::ast::TypeName;

use super::TypeError;

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