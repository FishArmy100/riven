use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{parsing::ast::{FileNode, StructDecl}, validation::type_error::TypeError};

use super::{types::TypeInfo, TypeResolver};

#[derive(Debug, Clone)]
pub struct StructInfo
{
    pub id: Uuid,
    pub name: String,
    pub members: HashMap<String, StructMember>,
    pub is_pub: bool,
    pub decl: Arc<StructDecl>
}

impl StructInfo
{
    pub fn new(decl: Arc<StructDecl>, type_resolver: &TypeResolver, file: &FileNode) -> Result<Self, TypeError>
    {
        let name = decl.id.value_string().unwrap().clone();

        let id = type_resolver.get_type_id(&file.info.path.split_relative(), &name).unwrap();

        let members = decl.members.iter().map(|m| {
            let name = m.id.value_string().unwrap().clone();
            let type_info = TypeInfo::from(&m.type_name, type_resolver, file)?;
            let has_init = m.initializer.is_some();

            Ok(StructMember {
                name,
                type_info,
                has_init,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(StructInfo { 
            id, 
            name, 
            members: members.into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
            is_pub: decl.pub_tok.is_some(),
            decl,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StructMember
{
    pub name: String,
    pub type_info: TypeInfo,
    pub has_init: bool, // delayed initialization
}
