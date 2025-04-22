pub mod ast;
pub mod type_info;
pub mod builtins;
pub mod operators;

use std::{collections::HashMap, sync::Arc};

use either::Either::{self, Left, Right};
use itertools::Itertools;
use operators::{BinaryOpType, UnaryOpType};
use type_info::TypeInfo;
use uuid::Uuid;

use crate::{compiler::CompilerError, lexing::token::{Token, TokenType}, parsing::ast::{Declaration, FileNode, StructDecl, TypeName}, utils::{FileInfo, PathInfo, TextLoc, TextPos}};

#[derive(Debug, Clone)]
pub enum TypeError
{
    UnknownType(Token, TextLoc),
    DuplicateTypeDef(Token, TextLoc),
    UnknownUsing(Vec<Token>, TextLoc),
    ConflictingTypes(Token, TextLoc),
    NoBinaryOp(BinaryOpType, String, String, TextLoc),
    NoUnaryOp(UnaryOpType, String, TextLoc),
}

impl CompilerError for TypeError
{
    fn msg(&self) -> String 
    {
        match self 
        {
            TypeError::UnknownType(token, _) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::DuplicateTypeDef(token, _) => format!("Duplicate type {}", token.value_string().unwrap()),
            TypeError::UnknownUsing(tokens, _) => format!("Using path {} does not exist", tokens.iter().map(|t| t.value_string().unwrap()).join(".")),
            TypeError::ConflictingTypes(token, _) => format!("Conflicting type definitions for {}", token.value_string().unwrap()),
            TypeError::NoBinaryOp(op, left, right, _) => format!("No binary operator {} for types {} and {}", op.to_string(), left, right),
            TypeError::NoUnaryOp(op, t, _) => format!("No unary operator {} for type {}", op.to_string(), t),
        }
    }

    fn loc(&self) -> TextLoc 
    {
        match self 
        {
            TypeError::UnknownType(_, loc) => loc.clone(),
            TypeError::DuplicateTypeDef(_, loc) => loc.clone(),
            TypeError::UnknownUsing(_, loc) => loc.clone(),
            TypeError::ConflictingTypes(_, loc) => loc.clone(),
            TypeError::NoBinaryOp(_, _, _, loc) => loc.clone(),
            TypeError::NoUnaryOp(_, _, loc) => loc.clone(),
        }
    }
}

#[derive(Debug)]
pub struct TypeLibrary
{
    type_defs: HashMap<Uuid, StructDef>,
    type_names: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

impl TypeLibrary
{
    pub fn resolver(&self) -> TypeResolver<'_>
    {
        TypeResolver(&self.type_names)
    }

    pub fn get_type(&self, id: &Uuid) -> &StructDef
    {
        self.type_defs.get(id).unwrap()
    }
}

pub struct TypeLibraryBuilder
{
    file_defs: HashMap<Vec<String>, Either<(HashMap<String, Arc<StructDecl>>, Arc<FileNode>), Vec<StructDef>>>,
    errors: Vec<TypeError>,
}

impl TypeLibraryBuilder
{
    pub fn new() -> Self
    {
        Self 
        {
            file_defs: HashMap::new(),
            errors: vec![],
        }
    }

    pub fn append_file(&mut self, file: Arc<FileNode>) 
    {
        let decls = self.file_defs.entry(file.info.path.split_relative()).or_insert(Either::Left((HashMap::new(), file.clone())));
        for d in file.declarations.iter()
        {
            let Declaration::Struct(s) = d else {
                continue;   
            };

            let name = s.id.value_string().unwrap().clone();
            if decls.as_ref().left().unwrap().0.contains_key(&name)
            {
                self.errors.push(TypeError::DuplicateTypeDef(s.id.clone(), s.id.get_loc(&file.info)));
            }
            else 
            {
                decls.as_mut().left().unwrap().0.insert(name, s.clone());
            }
        }
    }

    pub fn append_builtins(&mut self, path: Vec<String>, defs: Vec<StructDef>)
    {
        self.file_defs.insert(path, Right(defs));
    }

    pub fn build(self) -> Result<TypeLibrary, Vec<TypeError>>
    {
        if self.errors.len() > 0 {
            return Err(self.errors);
        }
        
        let type_names = self.file_defs.iter().map(|(path, decls)| {
            let defs = match decls
            {
                Left((decls, _)) => {
                    decls.iter()
                        .map(|(name, _)| (name.clone(), Uuid::new_v4()))
                        .collect::<HashMap<_, _>>()
                },
                Right(defs) => {
                    defs.iter()
                        .map(|d| (d.name.clone(), d.id.clone()))
                        .collect::<HashMap<_, _>>()
                }
            };
            (path.clone(), defs)
        }).collect();

        let resolver = TypeResolver(&type_names);

        let mut errors = vec![];
        let mut type_defs = HashMap::new();
        
        for (_, decls) in &self.file_defs
        {
            let (decls, file) = match decls {
                Left(parsed) => parsed,
                Right(builtins) => {
                    for b in builtins
                    {
                        type_defs.insert(b.id.clone(), b.clone());
                    }
                    continue;
                },
            };

            let mut usings = file.usings.iter()
                .map(|u| u.ids.iter().map(
                    |id| id.value_string().unwrap().clone())
                    .collect_vec())
                .collect_vec();

            let self_path = file.info.path.split_relative();
            if self_path.len() > 0
            {
                usings.push(self_path);
            }
            usings.push(vec![]);
            
            for (_, decl) in decls
            {
                let id = match resolver.resolve(&decl.id, &usings).to_result(&file.info) {
                    Ok(ok) => ok,
                    Err(err) => {
                        errors.push(err);
                        continue;
                    }
                };
                
                match StructDef::new(id, decl, resolver, &usings, &file.info)
                {
                    Ok(ok) => { type_defs.insert(id, ok); },
                    Err(err) => errors.push(err),
                };
            }
        }

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(TypeLibrary { 
                type_defs, 
                type_names 
            })    
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructDef
{
    pub id: Uuid,
    pub name: String,
    pub members: HashMap<String, StructMember>,
    pub is_pub: bool,
}

impl StructDef
{
    pub fn new(id: Uuid, decl: &StructDecl, resolver: TypeResolver, usings: &[Vec<String>], file: &FileInfo) -> Result<Self, TypeError>
    {
        let name = decl.id.value_string().unwrap().clone();
        let members = decl.members.iter().map(|m| {
            let name = m.id.value_string().unwrap().clone();
            let type_info = TypeInfo::from(&m.type_name, resolver, usings, file)?;
            Ok(StructMember {
                name,
                type_info
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(StructDef { 
            id, 
            name, 
            members: members.into_iter()
                .map(|m| (m.name.clone(), m))
                .collect(),
            is_pub: decl.pub_tok.is_some(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StructMember
{
    pub name: String,
    pub type_info: TypeInfo,
}

#[derive(Debug, Clone)]
pub enum ResolverResult
{
    Undefined(Token),
    ConflictingTypes(Token),
    Ok(Uuid),
}

impl ResolverResult
{
    pub fn to_result(&self, file: &FileInfo) -> Result<Uuid, TypeError>
    {
        match self 
        {
            ResolverResult::Undefined(token) => Err(TypeError::UnknownType(token.clone(), token.get_loc(file))),
            ResolverResult::ConflictingTypes(token) => Err(TypeError::UnknownType(token.clone(), token.get_loc(file))),
            ResolverResult::Ok(uuid) => Ok(uuid.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeResolver<'a>(pub &'a HashMap<Vec<String>, HashMap<String, Uuid>>);

impl<'a> TypeResolver<'a>
{
    pub fn resolve(&self, token: &Token, usings: &[Vec<String>]) -> ResolverResult
    {
        let name = token.value_string().unwrap();
        let possible = usings.iter()
            .filter_map(|u| self.0.get(u))
            .filter_map(|file| file.get(name))
            .collect_vec();

        
        if possible.len() == 1
        {
            ResolverResult::Ok(possible[0].clone())
        }
        else if possible.len() == 0
        {
            ResolverResult::Undefined(token.clone())
        }
        else 
        {
            ResolverResult::ConflictingTypes(token.clone())
        }
    }

    pub fn resolve_name(&self, name: &str, usings: &[Vec<String>]) -> Option<Uuid>
    {
        let possible = usings.iter()
            .filter_map(|u| self.0.get(u))
            .filter_map(|file| file.get(name))
            .collect_vec();

        if possible.len() == 1
        {
            Some(possible[0].clone())
        }
        else 
        {
            None    
        }
    }
}