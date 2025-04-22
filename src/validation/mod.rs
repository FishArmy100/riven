pub mod ast;
pub mod type_info;
pub mod builtins;
pub mod operators;

use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use type_info::TypeInfo;
use uuid::Uuid;

use crate::{compiler::CompilerError, lexing::token::{Token, TokenType}, parsing::ast::{Declaration, FileNode, StructDecl, TypeName}, utils::{PathInfo, TextPos}};

#[derive(Debug, Clone)]
pub enum TypeError
{
    UnknownType(Token),
    DuplicateTypeDef(Token),
    UnknownUsing(Vec<Token>),
}

impl CompilerError for TypeError
{
    fn msg(&self) -> String 
    {
        match self 
        {
            TypeError::UnknownType(token) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::DuplicateTypeDef(token) => format!("Duplicate type {}", token.value_string().unwrap()),
            TypeError::UnknownUsing(tokens) => format!("Using path {} does not exist", tokens.iter().map(|t| t.value_string().unwrap()).join(".")),
        }
    }

    fn loc(&self) -> Option<TextPos> 
    {
        match self 
        {
            TypeError::UnknownType(token) => Some(token.pos),
            TypeError::DuplicateTypeDef(token) => Some(token.pos),
            TypeError::UnknownUsing(tokens) => {
                tokens.iter().map(|v| v.pos).fold(None, |a, b| a.map_or(Some(b), |a| Some(a + b)))
            },
        }
    }
}


pub struct TypeLibrary
{
    types: HashMap<Uuid, StructDef>,
    files: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

pub struct TypeResolver
{
    types: HashMap<String, Uuid>
}

impl TypeResolver
{
    pub fn new(usings: &Vec<String>, decls: &HashMap<Vec<String>, HashMap<String, StructDeclInfo>>)
    {

    }
}

struct StructDeclInfo
{
    decl: Arc<StructDecl>,
    id: Uuid,
    name: String,
    file: PathInfo,
}

struct StructFileInfo
{
    
}

pub struct TypeLibraryBuilder<'a>
{
    type_id_map: HashMap<Vec<String>, HashMap<String, StructDeclInfo>>,
    files: HashMap<Vec<String>, &'a FileNode>,
    errors: Vec<TypeError>
}

impl<'a> TypeLibraryBuilder<'a>
{
    pub fn append_file(mut self, node: &'a FileNode) -> Self 
    {
        let path = node.path.as_ref().map_or(vec![], |p| p.split_relative());
        self.files.insert(path.clone(), node);

        node.declarations.iter().filter_map(|d| match d {
            Declaration::Struct(s) => Some(s.clone()),
            _ => None
        }).for_each(|d| {
            
        });

        self
    }

    pub fn build(self) -> Result<TypeLibrary, Vec<TypeError>>
    {
        let mut types = HashMap::new();
        for (path, file) in &self.files
        {
            let Some(id_map) = self.type_id_map.get(path) else {
                continue;
            };

            file.declarations.iter().filter_map(|d| match d {
                Declaration::Struct(s) => Some(s.clone()),
                _ => None
            }).for_each(|d| {
                
            });
        }

        todo!()
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