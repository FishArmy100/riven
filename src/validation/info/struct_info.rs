use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{parsing::ast::{Declaration, Expression, FileNode, StructDecl, StructDeclMember}, validation::{defs::struct_def::TypeDefMember, type_error::TypeError}};

use super::{types::TypeInfo, TypeResolver};

#[derive(Debug, Clone)]
pub enum StructDeclData
{
    Decl 
    {
        decl: Arc<StructDecl>,
        file: Arc<FileNode>,
        members: HashMap<String, StructMember>,
    },
    Builtin 
    {
        members: Vec<TypeDefMember>
    }
}

#[derive(Debug, Clone)]
pub struct StructInfo
{
    pub id: Uuid,
    pub name: String,
    pub is_pub: bool,
    pub decl_data: StructDeclData,
}

pub struct MemberInfo
{
    pub name: String,
    pub type_info: TypeInfo,
    pub has_init: bool,
}

impl StructInfo
{
    pub fn from_file(type_resolver: &TypeResolver, file: Arc<FileNode>) -> Result<Vec<Self>, Vec<TypeError>>
    {
        let mut errors = vec![];
        let mut infos = vec![];

        for d in &file.declarations
        {
            if let Declaration::Struct(s) = d 
            {
                match Self::new(s.clone(), type_resolver, file.clone())
                {
                    Ok(ok) => infos.push(ok),
                    Err(e) => errors.push(e),
                }
            }
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(infos)
    }

    pub fn new(decl: Arc<StructDecl>, type_resolver: &TypeResolver, file: Arc<FileNode>) -> Result<Self, TypeError>
    {
        let name = decl.id.value_string().unwrap().clone();

        let id = type_resolver.get_type_id(&file.info.path.split_relative(), &name).unwrap();

        let members = decl.members.iter().map(|m| {
            let name = m.id.value_string().unwrap().clone();
            let type_info = TypeInfo::from(&m.type_name, type_resolver, &file)?;
            let init = m.initializer.as_ref().map(|(_, i)| i.clone());

            Ok(StructMember {
                name,
                type_info,
                init,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(StructInfo { 
            id, 
            name,
            is_pub: decl.pub_tok.is_some(),
            decl_data: StructDeclData::Decl { 
                decl, 
                file,
                members: members.into_iter()
                    .map(|m| (m.name.clone(), m))
                    .collect() 
            }
        })
    }

    pub fn members(&self) -> HashMap<String, MemberInfo>
    {
        match &self.decl_data
        {
            StructDeclData::Decl { decl: _, file: _, members } => members.iter().map(|(n, m)| (n.clone(), MemberInfo {
                name: n.clone(),
                type_info: m.type_info.clone(),
                has_init: m.init.is_some(),
            })).collect(),
            StructDeclData::Builtin { members } => members.iter().map(|m| (m.name.clone(), MemberInfo {
                name: m.name.clone(),
                type_info: m.type_info.clone(),
                has_init: m.init.is_some(),
            })).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructMember
{
    pub name: String,
    pub type_info: TypeInfo,
    pub init: Option<Arc<Expression>>, // delayed initialization
}
