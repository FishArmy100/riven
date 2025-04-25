pub mod types;
pub mod struct_info;
pub mod func_info;

use std::collections::HashMap;

use func_info::FuncInfo;
use itertools::Itertools;
use struct_info::StructInfo;
use uuid::Uuid;

use crate::{lexing::token::Token, parsing::ast::{Declaration, FileNode, Program}, utils::FileInfo};

use super::{builtins::{self, BOOL_ID, BOOL_TYPE_NAME, FLOAT_ID, FLOAT_TYPE_NAME, INT_ID, INT_TYPE_NAME, STRING_ID, STRING_TYPE_NAME, VOID_ID, VOID_TYPE_NAME}, operators::GlobalOperators, type_error::TypeError};

pub struct InfoContext
{
    pub type_resolver: TypeResolver,
    pub structs: HashMap<Uuid, StructInfo>,
    pub func_resolver: FuncResolver,
    pub funcs: HashMap<Uuid, FuncInfo>,
}

impl InfoContext
{
    pub fn new(program: &Program) -> Result<Self, Vec<TypeError>>
    {
        let mut type_resolver = TypeResolver::new();
        let mut func_resolver = FuncResolver::new();

        let mut errors = vec![];

        for file in &program.files
        {
            if let Err(e) = type_resolver.append_file(file)
            {
                errors.extend(e);
            }
            if let Err(e) = func_resolver.append_file(file)
            {
                errors.extend(e);
            }
        }

        let mut structs = HashMap::new();
        let mut funcs = HashMap::new();
        
        for file in &program.files
        {
            match StructInfo::from_file(&type_resolver, file.clone())
            {
                Ok(ok) => ok.into_iter().for_each(|s| { structs.insert(s.id.clone(), s); }),
                Err(e) => errors.extend(e),
            }

            match FuncInfo::from_file(&type_resolver, &func_resolver, file.clone())
            {
                Ok(ok) => ok.into_iter().for_each(|f| { funcs.insert(f.id.clone(), f); }),
                Err(e) => errors.extend(e),
            }
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(InfoContext { 
            type_resolver, 
            structs, 
            func_resolver, 
            funcs,
        })
    }
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

pub struct TypeResolver
{
    map: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

impl TypeResolver
{
    pub fn new() -> Self 
    {
        let mut map = HashMap::<Vec<String>, HashMap<String, Uuid>>::new();
        let builtins: &mut HashMap<_, _> = map.entry(vec![]).or_default();
        builtins.insert(INT_TYPE_NAME.to_string(),      *INT_ID);
        builtins.insert(FLOAT_TYPE_NAME.to_string(),    *FLOAT_ID);
        builtins.insert(BOOL_TYPE_NAME.to_string(),     *BOOL_ID);
        builtins.insert(STRING_TYPE_NAME.to_string(),   *STRING_ID);
        builtins.insert(VOID_TYPE_NAME.to_string(),     *VOID_ID);
        
        TypeResolver { map }
    }

    pub fn append_file(&mut self, file: &FileNode) -> Result<(), Vec<TypeError>>
    {
        let mut errors = vec![];

        let file_types = self.map.entry(file.info.path.split_relative()).or_default();
        for decl in &file.declarations
        {
            if let Declaration::Struct(s) = decl {
                let name = s.id.value_string().unwrap().clone();
                if file_types.contains_key(&name)
                {
                    errors.push(TypeError::DuplicateTypeDef(s.id.clone(), s.id.get_loc(&file.info)));
                }
                else 
                {
                    file_types.insert(name, Uuid::new_v4());
                }
            }
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(())
    }

    pub fn resolve(&self, token: &Token, file: &FileNode) -> TypeResolverResult
    {
        let name = token.value_string().unwrap();
        let possible = file.using_paths.iter()
            .filter_map(|u| self.map.get(u))
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

    pub fn resolve_result(&self, token: &Token, file: &FileNode) -> Result<Uuid, TypeError>
    {
        self.resolve(token, file).to_result(&file.info)
    }

    pub fn get_type_id(&self, path: &[String], name: &String) -> Option<Uuid>
    {
        self.map.get(path).map(|d| d.get(name).cloned()).flatten()
    }
}

#[derive(Debug, Clone)]
pub enum FuncResolverResult
{
    Undefined(Token),
    ConflictingFuncs(Token),
    Ok(Uuid),
}

impl FuncResolverResult
{
    pub fn to_result(&self, file: &FileInfo) -> Result<Uuid, TypeError>
    {
        match self 
        {
            FuncResolverResult::Undefined(token) => Err(TypeError::UnknownType(token.clone(), token.get_loc(file))),
            FuncResolverResult::ConflictingFuncs(token) => Err(TypeError::ConflictingFunctions(token.clone(), token.get_loc(file))),
            FuncResolverResult::Ok(uuid) => Ok(uuid.clone()),
        }
    }
}

pub struct FuncResolver
{
    map: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

impl FuncResolver
{
    pub fn new() -> Self 
    {
        Self { map: HashMap::new() }
    }

    pub fn append_file(&mut self, file: &FileNode) -> Result<(), Vec<TypeError>>
    {
        let mut errors = vec![];

        let file_types = self.map.entry(file.info.path.split_relative()).or_default();
        for decl in &file.declarations
        {
            if let Declaration::Fn(f) = decl {
                let name = f.id.value_string().unwrap().clone();
                if file_types.contains_key(&name)
                {
                    errors.push(TypeError::DuplicateTypeDef(f.id.clone(), f.id.get_loc(&file.info)));
                }
                else 
                {
                    file_types.insert(name, Uuid::new_v4());
                }
            }
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(())
    }

    pub fn resolve(&self, token: &Token, file: &FileNode) -> FuncResolverResult
    {
        let name = token.value_string().unwrap();
        let possible = file.using_paths.iter()
            .filter_map(|u| self.map.get(u))
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
            FuncResolverResult::ConflictingFuncs(token.clone())
        }
    }

    pub fn resolve_result(&self, token: &Token, file: &FileNode) -> Result<Uuid, TypeError>
    {
        self.resolve(token, file).to_result(&file.info)
    }

    pub fn get_func_id(&self, path: &[String], name: &String) -> Option<Uuid>
    {
        self.map.get(path).map(|d| d.get(name).cloned()).flatten()
    }
}