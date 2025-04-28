pub mod types;
pub mod struct_info;
pub mod func_info;
use std::collections::HashMap;

use func_info::FuncInfo;
use itertools::Itertools;
use struct_info::StructInfo;
use types::TypeInfo;
use uuid::Uuid;

use crate::{lexing::token::Token, parsing::ast::{Declaration, FileNode, Program}, utils::FileInfo};

use super::{builtins, type_error::TypeError};

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

        let mut errors = vec![];

        for file in &program.files
        {
            if let Err(e) = type_resolver.append_file(file)
            {
                errors.extend(e);
            }
        }
        
        let all_types = type_resolver.type_ids();
        let built_in_funcs = builtins::get_builtin_funcs(&all_types);
        let mut func_resolver = FuncResolver::new(&built_in_funcs.funcs);

        for file in &program.files
        {
            if let Err(e) = func_resolver.append_file(file, &type_resolver)
            {
                errors.extend(e);
            }
        }

        let mut structs = HashMap::new();
        for b in builtins::get_builtin_types()
        {
            structs.insert(b.id.clone(), b);
        }
        
        let mut funcs = HashMap::new();
        for b in &built_in_funcs.funcs
        {
            funcs.insert(b.id.clone(), b.clone());
        }
        
        for file in &program.files
        {
            match StructInfo::from_file(&type_resolver, file.clone())
            {
                Ok(ok) => ok.into_iter().for_each(|s| { structs.insert(s.id.clone(), s); }),
                Err(e) => errors.extend(e),
            }
        }

        for file in &program.files
        {
            match FuncInfo::from_file(&type_resolver, &func_resolver, file.clone(), &structs)
            {
                Ok(ok) => ok.into_iter().for_each(|f| { funcs.insert(f.id.clone(), f); }),
                Err(e) => errors.extend(e),
            }
        }

        for b in builtins::get_builtin_types()
        {
            structs.insert(b.id.clone(), b);
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

#[derive(Debug)]
pub struct TypeResolver
{
    map: HashMap<Vec<String>, HashMap<String, Uuid>>,
}

impl TypeResolver
{
    pub fn new() -> Self 
    {
        let mut map = HashMap::<Vec<String>, HashMap<String, Uuid>>::new();
        let bins: &mut HashMap<_, _> = map.entry(vec![]).or_default();
        for b in builtins::get_builtin_types()
        {
            bins.insert(b.name, b.id);
        }
        
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

    pub fn type_ids(&self) -> Vec<Uuid>
    {
        self.map.values()
            .map(|v| v.values().map(|id| id.clone()))
            .flatten()
            .collect()
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

    pub fn is_ok(&self) -> bool 
    {
        match self 
        {
            Self::Ok(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct FuncResolver
{
    map: HashMap<Vec<String>, HashMap<(Option<TypeInfo>, String), Uuid>>,
}

impl FuncResolver
{
    pub fn new(builtins: &Vec<FuncInfo>) -> Self 
    {
        let mut map = HashMap::<Vec<String>, HashMap<(Option<TypeInfo>, String), Uuid>>::new();
        let bins: &mut HashMap<_, _> = map.entry(vec![]).or_default();
        for b in builtins
        {
            bins.insert((b.parent.clone(), b.name.clone()), b.id);
        }
        
        Self { map }
    }

    pub fn append_file(&mut self, file: &FileNode, type_resolver: &TypeResolver) -> Result<(), Vec<TypeError>>
    {
        let mut errors = vec![];

        let file_types = self.map.entry(file.info.path.split_relative()).or_default();
        for decl in &file.declarations
        {
            if let Declaration::Fn(f) = decl {
                let name = f.id.value_string().unwrap().clone();
                
                let parent_type = match f.type_name.as_ref().map(|(_, r)| r)
                {
                    Some(t) => match TypeInfo::from(&t, type_resolver, file)
                    {
                        Ok(ok) => Some(ok),
                        Err(err) => return Err(vec![err])
                    },
                    None => None
                };


                if file_types.contains_key(&(parent_type.clone(), name.clone()))
                {
                    errors.push(TypeError::DuplicateMemberFunc(f.id.get_loc(&file.info)));
                }
                else 
                {
                    file_types.insert((parent_type, name), Uuid::new_v4());
                }
            }
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(())
    }

    pub fn resolve(&self, type_info: Option<TypeInfo>, token: &Token, file: &FileNode) -> FuncResolverResult
    {
        let name = token.value_string().unwrap();
        let possible = file.using_paths.iter()
            .filter_map(|u| self.map.get(u))
            .filter_map(|file| file.get(&(type_info.clone(), name.clone())))
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

    pub fn resolve_result(&self, parent_type: Option<TypeInfo>, token: &Token, file: &FileNode) -> Result<Uuid, TypeError>
    {
        self.resolve(parent_type, token, file).to_result(&file.info)
    }

    pub fn get_func_id(&self, path: &[String], parent_type: Option<TypeInfo>, name: String) -> Option<Uuid>
    {
        self.map.get(path).map(|d| d.get(&(parent_type, name)).cloned()).flatten()
    }
}