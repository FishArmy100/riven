use std::sync::Arc;

use uuid::Uuid;

use crate::{parsing::ast::{FileNode, FnDecl}, validation::{builtins::VOID_TYPE, type_error::TypeError}};

use super::{types::TypeInfo, FuncResolver, TypeResolver};


#[derive(Debug, Clone)]
pub struct FuncInfoParam
{
    pub name: String,
    pub type_info: TypeInfo,
    pub has_init: bool
}

#[derive(Debug, Clone)]
pub struct FuncInfo
{
    pub id: Uuid,
    pub name: String,
    pub parameters: Vec<FuncInfoParam>,
    pub returned: TypeInfo,
    pub is_pub: bool,
    pub decl: Arc<FnDecl>,
}

impl FuncInfo
{
    pub fn new(decl: Arc<FnDecl>, resolver: &TypeResolver, func_resolver: &FuncResolver, file: &FileNode) -> Result<Self, TypeError>
    {
        let name = decl.id.value_string().unwrap().clone();
        let id = func_resolver.get_func_id(&file.info.path.split_relative(), &name).unwrap();
        let mut has_init = false;

        let parameters = decl.params.iter().map(|p| {
            let type_info = TypeInfo::from(&p.type_name, resolver, file)?;
            let m_has_init = p.default_value.is_some();
            if m_has_init
            {
                has_init = true;
            }

            if has_init && !m_has_init
            {
                return Err(TypeError::FunctionArgumentNeedsInitializer((p.id.pos + p.type_name.get_pos()).get_loc(&file.info)))
            }

            Ok(FuncInfoParam {
                name: p.id.value_string().unwrap().clone(),
                type_info,
                has_init: m_has_init,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        let returned = match &decl.return_type {
            Some((_, type_name)) => TypeInfo::from(&type_name, resolver, file)?,
            None => VOID_TYPE.clone(),
        };

        Ok(FuncInfo { 
            id, 
            name, 
            parameters,
            returned,
            is_pub: decl.pub_tok.is_some(),
            decl
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
