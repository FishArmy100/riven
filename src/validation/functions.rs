use std::{cell::UnsafeCell, collections::HashMap, sync::Arc};

use either::Either::{self, Left, Right};
use itertools::Itertools;
use uuid::Uuid;

use crate::{lexing::token::Token, parsing::ast::{BlockStmt, Declaration, FileNode, FnDecl}, utils::FileInfo};

use super::{ast::{stmt::TypedStatement, ExprCheckArgs}, builtins::VOID_TYPE, type_info::TypeInfo, var::VarDef, Initializer, TypeError, TypeResolver};

#[derive(Debug, Clone)]
pub enum FuncBody
{
    AST(Arc<BlockStmt>),
    Built 
    {
        variables: Arc<HashMap<Uuid, VarDef>>,
        body: Arc<TypedStatement>,
    }
}

impl FuncBody 
{
    pub fn built(&self) -> (&HashMap<Uuid, VarDef>, &TypedStatement)
    {
        match self 
        {
            Self::Built { variables, body } => (&variables, &body),
            _ => panic!("Trying to access un-built function data")
        }
    }
}

#[derive(Debug)]
pub struct FuncLibrary
{
    func_defs: UnsafeCell<HashMap<Uuid, FuncDef>>,
    func_names: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

impl FuncLibrary
{
    pub fn resolver(&self) -> FuncResolver<'_>
    {
        FuncResolver(&self.func_names)
    }

    pub fn get_func(&self, id: &Uuid) -> &FuncDef
    {
        unsafe 
        {
            (*self.func_defs.get()).get(id).unwrap()
        }
    }

    pub fn build_initializers(&self, args: ExprCheckArgs) -> Result<(), Vec<TypeError>>
    {
        let mut errors = vec![];
        let path = args.file.path.split_relative();

        for id in self.func_names.get(&path).unwrap().values()
        {
            let def =  unsafe {
                (*self.func_defs.get()).get_mut(id).unwrap()
            };

            for param in def.parameters.iter_mut()
            {
                if let Err(e) = param.initializer.build(args, &param.type_info)
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

struct FnFileDefs 
{
    defs: HashMap<String, Arc<FnDecl>>,
    file: Arc<FileNode>,
}

pub struct FuncLibraryBuilder
{
    file_defs: HashMap<Vec<String>, Either<FnFileDefs, Vec<FuncDef>>>,
    errors: Vec<TypeError>,
}

impl FuncLibraryBuilder
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
        let decls = self.file_defs.entry(file.info.path.split_relative()).or_insert(Either::Left(FnFileDefs { 
            defs: HashMap::new(), 
            file: file.clone() 
        }));

        for d in file.declarations.iter()
        {
            let Declaration::Fn(f) = d else {
                continue;   
            };

            let name = f.id.value_string().unwrap().clone();
            if decls.as_ref().left().unwrap().defs.contains_key(&name)
            {
                self.errors.push(TypeError::DuplicateTypeDef(f.id.clone(), f.id.get_loc(&file.info)));
            }
            else 
            {
                decls.as_mut().left().unwrap().defs.insert(name, f.clone());
            }
        }
    }

    pub fn append_builtins(&mut self, path: Vec<String>, defs: Vec<FuncDef>)
    {
        self.file_defs.insert(path, Right(defs));
    }

    pub fn build(self, type_resolver: TypeResolver) -> Result<FuncLibrary, Vec<TypeError>>
    {
        if self.errors.len() > 0 {
            return Err(self.errors);
        }
        
        let func_names = self.file_defs.iter().map(|(path, decls)| {
            let defs = match decls
            {
                Left(defs) => {
                    defs.defs.iter()
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

        let resolver = FuncResolver(&func_names);

        let mut errors = vec![];
        let mut func_defs = HashMap::new();
        
        for (_, decls) in &self.file_defs
        {
            let FnFileDefs { defs, file } = match decls {
                Left(parsed) => parsed,
                Right(builtins) => {
                    for b in builtins
                    {
                        func_defs.insert(b.id.clone(), b.clone());
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
            
            for (_, decl) in defs
            {
                let id = match resolver.resolve(&decl.id, &usings).to_result(&file.info) {
                    Ok(ok) => ok,
                    Err(err) => {
                        errors.push(err);
                        continue;
                    }
                };
                
                match FuncDef::new(id, decl, type_resolver, &usings, &file.info)
                {
                    Ok(ok) => { func_defs.insert(id, ok); },
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
            Ok(FuncLibrary { 
                func_defs: UnsafeCell::new(func_defs), 
                func_names 
            })    
        }
    }
}

#[derive(Debug, Clone)]
pub enum FuncResolverResult
{
    Undefined(Token),
    ConflictingFunctions(Token),
    Ok(Uuid),
}

impl FuncResolverResult
{
    pub fn to_result(&self, file: &FileInfo) -> Result<Uuid, TypeError>
    {
        match self 
        {
            FuncResolverResult::Undefined(token) => Err(TypeError::UndefinedFunction(token.clone(), token.get_loc(file))),
            FuncResolverResult::ConflictingFunctions(token) => Err(TypeError::ConflictingFunctions(token.clone(), token.get_loc(file))),
            FuncResolverResult::Ok(uuid) => Ok(uuid.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FuncResolver<'a>(pub &'a HashMap<Vec<String>, HashMap<String, Uuid>>);

impl<'a> FuncResolver<'a>
{
    pub fn resolve(&self, token: &Token, usings: &[Vec<String>]) -> FuncResolverResult
    {
        let name = token.value_string().unwrap();
        let possible = usings.iter()
            .filter_map(|u| self.0.get(u))
            .filter_map(|file| file.get(name))
            .collect_vec();

        
        if possible.len() == 1
        {
            FuncResolverResult::Ok(possible[0].clone())
        }
        else if possible.len() == 0
        {
            FuncResolverResult::Undefined(token.clone())
        }
        else 
        {
            FuncResolverResult::ConflictingFunctions(token.clone())
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