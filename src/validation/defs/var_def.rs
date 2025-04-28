use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::{parsing::ast::LetStmt, validation::{ast::{ExprCheckArgs, TypedExpression}, info::types::TypeInfo, type_error::TypeError}};

#[derive(Debug, Clone)]
pub struct VarDef
{
    pub id: Uuid,
    pub name: String,
    pub type_info: TypeInfo,
    pub initializer: Option<Arc<TypedExpression>>,
}

impl VarDef
{
    pub fn from_let(let_stmt: &LetStmt, args: &ExprCheckArgs) -> Result<Self, TypeError>
    {
        let name = let_stmt.id.value_string().unwrap().clone();
        let id = Uuid::new_v4();
        let mut type_info = if let Some((_, t)) = &let_stmt.type_name 
        {
            let type_info = TypeInfo::from(t, &args.context.type_resolver, args.file)?;
            Some(type_info)
        } else { None };
        
        let initializer = TypedExpression::check_expr(&let_stmt.expression, args, type_info.as_ref())?;
        if type_info.is_none()
        {
            type_info = Some(initializer.returned().clone())
        }

        Ok(Self {
            id,
            name,
            type_info: type_info.unwrap(),
            initializer: Some(Arc::new(initializer)),
        })
    }

    pub fn new(name: String, type_info: TypeInfo, id: Uuid) -> Self 
    {
        Self {
            id,
            name,
            type_info,
            initializer: None,
        }
    }
}

#[derive(Debug)]
pub struct VariableStack
{
    variables: HashMap<Uuid, VarDef>,
    stack: Vec<HashMap<String, Uuid>>,
}

impl VariableStack
{
    pub fn new() -> Self
    {
        Self 
        {
            variables: HashMap::new(),
            stack: vec![],
        }
    }

    pub fn get_vars(&self) -> HashMap<Uuid, VarDef>
    {
        self.variables.clone()
    }

    pub fn resolve_var(&self, name: &str) -> Option<Uuid>
    {
        self.stack.iter()
            .rev()
            .find_map(|v| v.get(name))
            .cloned()
    }

    pub fn get_var(&self, id: &Uuid) -> &VarDef
    {
        self.variables.get(id).unwrap()
    }

    pub fn push_frame(&mut self)
    {
        self.stack.push(HashMap::new());
    }

    pub fn pop_frame(&mut self)
    {
        self.stack.pop();
    }

    pub fn add_var_def(&mut self, def: VarDef)
    {
        self.stack.last_mut().unwrap().insert(def.name.clone(), def.id.clone());
        self.variables.insert(def.id.clone(), def);
    }

    pub fn add_var(&mut self, name: String, type_info: TypeInfo, id: Uuid)
    {
        let def = VarDef::new(name, type_info, id);
        self.add_var_def(def);
    }
}