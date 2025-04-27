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
        params: Vec<FuncDefParam>
    }
}

#[derive(Debug, Clone)]
pub struct FuncInfoParam
{
    pub name: String,
    pub type_info: TypeInfo,
    pub init: Option<Arc<Expression>>,
    pub id: Uuid, // is the variable id
}

#[derive(Debug, Clone)]
pub struct FuncInfo
{
    pub id: Uuid,
    pub name: String,
    pub parent: Option<TypeInfo>,
    pub has_self: bool,
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
        let parent_type =  match &decl.type_name 
        {
            Some((_, t)) => Some(TypeInfo::from(t, resolver, &file)?),
            None => None,
        };

        let fn_id = func_resolver.get_func_id(&file.info.path.split_relative(), parent_type, name.clone()).unwrap();

        let parent = decl.type_name.as_ref().map(|(_, t)| TypeInfo::from(t, resolver, &file));
        if let Some(Err(e)) = parent {
            return Err(e)
        }
        let parent = parent.map(|r| r.unwrap());

        let has_self = decl.self_param.is_some();
        if has_self && !parent.is_some()
        {
            return Err(TypeError::CannotUseSelfInContext(decl.self_param.as_ref().unwrap().get_loc(&file.info)))
        }

        let mut has_init = false;

        let mut parameters = decl.params.iter().map(|p| {
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
                id: Uuid::new_v4(),
            })
        }).collect::<Result<Vec<_>, _>>()?;

        if has_self
        {
            parameters.insert(0, FuncInfoParam { 
                name: "self".into(), 
                type_info: parent.as_ref().unwrap().clone(), 
                init: None,
                id: Uuid::new_v4(), 
            });
        }

        let returned = match &decl.return_type {
            Some((_, type_name)) => TypeInfo::from(&type_name, resolver, &file)?,
            None => VOID_TYPE.clone(),
        };

        Ok(FuncInfo { 
            id: fn_id, 
            name, 
            parent,
            has_self: decl.self_param.is_some(),
            parameters,
            returned,
            is_pub: decl.pub_tok.is_some(),
            decl_data: FuncDeclData::Decl { 
                decl, 
                file 
            }
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
