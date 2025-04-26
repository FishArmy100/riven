use std::sync::Arc;

use uuid::Uuid;

use crate::{parsing::ast::FileNode, utils::Shared, validation::{ast::{ExprCheckArgs, TypedExpression}, info::{struct_info::{StructDeclData, StructInfo}, types::TypeInfo, InfoContext}, operators::GlobalOperators, type_error::TypeError}};

use super::var_def::VariableStack;

#[derive(Debug)]
pub struct StructDef 
{
    pub id: Uuid,
    pub name: String,
    pub is_pub: bool,
    pub members: Vec<TypeDefMember>,
}

#[derive(Debug, Clone)]
pub struct TypeDefMember
{
    pub name: String,
    pub type_info: TypeInfo,
    pub init: Option<Arc<TypedExpression>>,
}

impl StructDef
{
    pub fn new(info: &StructInfo, context: &InfoContext, operators: &GlobalOperators) -> Result<Self, Vec<TypeError>>
    {
        let var_stack = Shared::new(VariableStack::new());

        let (_, file, s_members) = match &info.decl_data
        {
            StructDeclData::Decl { decl, file, members: s_members } => (decl, file, s_members),
            StructDeclData::Builtin { members } => {
                return Ok(Self { 
                    id: info.id.clone(), 
                    name: info.name.clone(), 
                    is_pub: info.is_pub, 
                    members: members.clone(),
                })
            },
        };

        let check_args = ExprCheckArgs {
            operators,
            context,
            var_stack: var_stack.clone(),
            file: &file,
            self_type: None,
            fn_ret_type: None,
        };

        let mut errors = vec![];
        let mut members = vec![];

        for (name, member) in s_members
        {
            let init = if let Some(init) = &member.init {
                let checked_init = match TypedExpression::check_expr(init, &check_args, Some(&member.type_info)) {
                    Ok(ok) => ok,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    },
                };

                if checked_init.returned() != &member.type_info
                {
                    errors.push(TypeError::ExpectedType(member.type_info.pretty_print(&check_args.context.structs), init.get_pos().get_loc(&check_args.file.info)));
                    continue;
                }

                Some(Arc::new(checked_init))

            } else { None };

            members.push(TypeDefMember {
                name: name.clone(),
                type_info: member.type_info.clone(),
                init,
            });
        }

        if errors.len() > 0
        {
            return Err(errors);
        }

        Ok(StructDef {
            id: info.id.clone(),
            name: info.name.clone(),
            is_pub: info.is_pub,
            members,
        })
    }
}