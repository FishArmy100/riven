use std::sync::Arc;

use uuid::Uuid;

use crate::{parsing::ast::{Declaration, Expression, FileNode, FnDecl}, validation::{builtins::VOID_TYPE, defs::func_def::FuncDefParam, type_error::TypeError}};

use super::{types::TypeInfo, FuncResolver, TypeResolver};

#[derive(Debug, Clone)]
pub enum FuncDeclData
{
    Decl 
    {
        decl: Arc<FnDecl>,
        file: Arc<FileNode>
    },
    Builtin 
    {
        params: Vec<Arc<FuncDefParam>>
    }
}

#[derive(Debug, Clone)]
pub struct FuncInfoParam
{
    pub name: String,
    pub type_info: TypeInfo,
    pub init: Option<Arc<Expression>>
}

#[derive(Debug, Clone)]
pub struct FuncInfo
{
    pub id: Uuid,
    pub name: String,
    pub parameters: Vec<FuncInfoParam>,
    pub returned: TypeInfo,
    pub is_pub: bool,
    pub decl_data: FuncDeclData,
}

impl FuncInfo
{
    pub fn from_file(type_resolver: &TypeResolver, func_resolver: &FuncResolver, file: Arc<FileNode>) -> Result<Vec<Self>, Vec<TypeError>>
    {
        let mut errors = vec![];
        let mut infos = vec![];

        for d in &file.declarations
        {
            if let Declaration::Fn(s) = d 
            {
                match Self::new(s.clone(), type_resolver, func_resolver, file.clone())
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
    
    pub fn new(decl: Arc<FnDecl>, resolver: &TypeResolver, func_resolver: &FuncResolver, file: Arc<FileNode>) -> Result<Self, TypeError>
    {
        let name = decl.id.value_string().unwrap().clone();
        let id = func_resolver.get_func_id(&file.info.path.split_relative(), &name).unwrap();
        let mut has_init = false;

        let parameters = decl.params.iter().map(|p| {
            let type_info = TypeInfo::from(&p.type_name, resolver, &file)?;
            let m_has_init = p.default_value.as_ref().map(|v| v.1.clone());
            if m_has_init.is_some()
            {
                has_init = true;
            }

            if has_init && !m_has_init.is_some()
            {
                return Err(TypeError::FunctionArgumentNeedsInitializer((p.id.pos + p.type_name.get_pos()).get_loc(&file.info)))
            }

            Ok(FuncInfoParam {
                name: p.id.value_string().unwrap().clone(),
                type_info,
                init: m_has_init,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        let returned = match &decl.return_type {
            Some((_, type_name)) => TypeInfo::from(&type_name, resolver, &file)?,
            None => VOID_TYPE.clone(),
        };

        Ok(FuncInfo { 
            id, 
            name, 
            parameters,
            returned,
            is_pub: decl.pub_tok.is_some(),
            decl,
            file
        })
    }

    pub fn get_type_info(&self) -> TypeInfo
    {
        TypeInfo::Function { 
            args: self.parameters.iter()
                .map(|p| p.type_info.clone())
                .collect(), 
            returned: Box::new(self.returned.clone())
        }
    }
}
