pub mod ast;
pub mod type_info;
pub mod builtins;
pub mod operators;

use std::collections::HashMap;

use itertools::Itertools;
use type_info::TypeInfo;
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


pub struct TypeLibrary
{
    types: HashMap<Uuid, StructDef>,
    files: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

impl TypeLibrary
{
    pub fn new() -> Self 
    {
        let types: HashMap<_, _> = builtins::get_builtin_types().into_iter().map(|t| (t.id.clone(), t)).collect();
        let mut files = HashMap::<Vec<String>, HashMap<String, Uuid>>::new();
        files.insert(vec![], types.values().map(|t| (t.name.clone(), t.id.clone())).collect());

        Self 
        {
            types,
            files: HashMap::new()
        }
    }

    pub fn push_file(&mut self, node: &FileNode)
    {
        
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