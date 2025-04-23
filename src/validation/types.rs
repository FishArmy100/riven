use std::{cell::UnsafeCell, collections::HashMap, sync::Arc};

use either::Either::{self, Left, Right};
use itertools::Itertools;
use uuid::Uuid;

use crate::{lexing::token::Token, parsing::ast::{Declaration, Expression, FileNode, StructDecl}, utils::FileInfo};

use super::{ast::{ExprCheckArgs, TypedExpression}, type_info::TypeInfo, Initializer, TypeError};

#[derive(Debug)]
pub struct TypeLibrary
{
    type_defs: UnsafeCell<HashMap<Uuid, StructDef>>, // This is unsafe technically, but idk how to do it otherwise :shrug:
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
        unsafe 
        {
            (*self.type_defs.get()).get(id).unwrap()
        }
    }

    pub fn build_initializers(&self, args: ExprCheckArgs) -> Result<(), Vec<TypeError>>
    {
        let mut errors = vec![];
        let path = args.file.path.split_relative();

        for id in self.type_names.get(&path).unwrap().values()
        {
            let def =  unsafe {
                (*self.type_defs.get()).get_mut(id).unwrap()
            };
            
            for member in def.members.values_mut()
            {
                if let Err(e) = member.initializer.build(args, &member.type_info)
                {
                    errors.push(e);
                }
            }
        }

        if errors.len() > 0
        {
            Err(errors)
        }
        else 
        {
            Ok(())    
        }
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
                type_defs: UnsafeCell::new(type_defs), 
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
    pub fn new(id: Uuid, decl: &StructDecl, type_resolver: TypeResolver, usings: &[Vec<String>], file: &FileInfo) -> Result<Self, TypeError>
    {
        let name = decl.id.value_string().unwrap().clone();
        let members = decl.members.iter().map(|m| {
            let name = m.id.value_string().unwrap().clone();
            let type_info = TypeInfo::from(&m.type_name, type_resolver, usings, file)?;
            let initializer = m.initializer.as_ref()
                .map(|(_, init)| Arc::new(init.clone()))
                .map_or(Initializer::None, |init| Initializer::AST(init));

            Ok(StructMember {
                name,
                type_info,
                initializer,
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
    pub initializer: Initializer, // delayed initialization
}

#[derive(Debug, Clone)]
pub enum TypeResolverResult
{
    Undefined(Token),
    ConflictingTypes(Token),
    Ok(Uuid),
}

impl TypeResolverResult
{
    pub fn to_result(&self, file: &FileInfo) -> Result<Uuid, TypeError>
    {
        match self 
        {
            TypeResolverResult::Undefined(token) => Err(TypeError::UnknownType(token.clone(), token.get_loc(file))),
            TypeResolverResult::ConflictingTypes(token) => Err(TypeError::ConflictingTypes(token.clone(), token.get_loc(file))),
            TypeResolverResult::Ok(uuid) => Ok(uuid.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypeResolver<'a>(pub &'a HashMap<Vec<String>, HashMap<String, Uuid>>);

impl<'a> TypeResolver<'a>
{
    pub fn resolve(&self, token: &Token, usings: &[Vec<String>]) -> TypeResolverResult
    {
        let name = token.value_string().unwrap();
        let possible = usings.iter()
            .filter_map(|u| self.0.get(u))
            .filter_map(|file| file.get(name))
            .collect_vec();

        
        if possible.len() == 1
        {
            TypeResolverResult::Ok(possible[0].clone())
        }
        else if possible.len() == 0
        {
            TypeResolverResult::Undefined(token.clone())
        }
        else 
        {
            TypeResolverResult::ConflictingTypes(token.clone())
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