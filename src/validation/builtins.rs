use uuid::Uuid;

use crate::validation::info::struct_info::StructDeclData;

use super::{defs::func_def::{FuncDef, FuncDefParam}, info::{func_info::{FuncDeclData, FuncInfo, FuncInfoParam}, struct_info::StructInfo, types::TypeInfo}, operators::{BinaryOp, BinaryOpType, CastOp, GlobalOperators, IndexOp, UnaryOp, UnaryOpType}};

pub const INT_TYPE_NAME: &str = "Int";
pub const FLOAT_TYPE_NAME: &str = "Float";
pub const BOOL_TYPE_NAME: &str = "Bool";
pub const STRING_TYPE_NAME: &str = "String";
pub const VOID_TYPE_NAME: &str = "Void";

lazy_static::lazy_static! 
{
    pub static ref INT_ID: Uuid = Uuid::new_v4();
    pub static ref FLOAT_ID: Uuid = Uuid::new_v4();
    pub static ref BOOL_ID: Uuid = Uuid::new_v4();
    pub static ref STRING_ID: Uuid = Uuid::new_v4();
    pub static ref VOID_ID: Uuid = Uuid::new_v4();

    pub static ref INT_TYPE: TypeInfo =     TypeInfo::Primary(INT_ID.clone());
    pub static ref FLOAT_TYPE: TypeInfo =   TypeInfo::Primary(FLOAT_ID.clone());
    pub static ref BOOL_TYPE: TypeInfo =    TypeInfo::Primary(BOOL_ID.clone());
    pub static ref STRING_TYPE: TypeInfo =  TypeInfo::Primary(STRING_ID.clone());
    pub static ref VOID_TYPE: TypeInfo =    TypeInfo::Primary(VOID_ID.clone());

    pub static ref PRINT_ID: Uuid = Uuid::new_v4();
    pub static ref PRINTLN_ID: Uuid = Uuid::new_v4();
    pub static ref READ_LINE_ID: Uuid = Uuid::new_v4();
    pub static ref STRING_TO_INT_ID: Uuid = Uuid::new_v4();

    pub static ref UNWRAP_ID: Uuid = Uuid::new_v4();
    pub static ref IS_NONE_ID: Uuid = Uuid::new_v4();
}

pub fn resolve_builtin_member_funcs(type_info: Option<&TypeInfo>, name: &str) -> Option<Uuid>
{
    if let Some(TypeInfo::Optional(_)) = type_info
    {
        if name == "is_none"
        {
            return Some(IS_NONE_ID.clone());
        }

        if name == "unwrap"
        {
            return Some(UNWRAP_ID.clone())
        }
    }

    None
}

pub fn get_builtin_member_func(parent: Option<&TypeInfo>, id: &Uuid) -> Option<FuncInfo>
{
    if let Some(TypeInfo::Optional(inner)) = &parent
    {
        if *id == *IS_NONE_ID
        {
            return Some(FuncInfo { 
                id: id.clone(), 
                name: "is_none".into(), 
                is_pub: true, 
                returned: BOOL_TYPE.clone(), 
                parameters: vec![
                    FuncInfoParam {
                        id: Uuid::new_v4(),
                        name: "self".into(),
                        type_info: parent.unwrap().clone(),
                        init: None,
                    }
                ],
                has_self: true,
                parent: parent.cloned(),
                decl_data: FuncDeclData::Builtin,
            })
        }
        
        if *id == *UNWRAP_ID
        {
            return Some(FuncInfo { 
                id: id.clone(), 
                name: "unwrap".into(), 
                is_pub: true, 
                returned: inner.as_ref().clone(), 
                parameters: vec![
                    FuncInfoParam {
                        id: Uuid::new_v4(),
                        name: "self".into(),
                        type_info: parent.unwrap().clone(),
                        init: None,
                    }
                ], 
                has_self: true,
                parent: parent.cloned(),
                decl_data: FuncDeclData::Builtin,
            })
        }
    }

    None
}

pub fn get_builtin_types() -> Vec<StructInfo>
{
    let int_type = StructInfo {
        name: INT_TYPE_NAME.into(),
        id: INT_ID.clone(),
        is_pub: true,
        decl_data: StructDeclData::Builtin { members: vec![] }
    };

    let float_type = StructInfo {
        name: FLOAT_TYPE_NAME.into(),
        id: FLOAT_ID.clone(),
        is_pub: true,
        decl_data: StructDeclData::Builtin { members: vec![] }
    };

    let bool_type = StructInfo {
        name: BOOL_TYPE_NAME.into(),
        id: BOOL_ID.clone(),
        is_pub: true,
        decl_data: StructDeclData::Builtin { members: vec![] }
    };

    let string_type = StructInfo {
        name: STRING_TYPE_NAME.into(),
        id: STRING_ID.clone(),
        is_pub: true,
        decl_data: StructDeclData::Builtin { members: vec![] }
    };

    let void_type = StructInfo {
        name: VOID_TYPE_NAME.into(),
        id: VOID_ID.clone(),
        is_pub: true,
        decl_data: StructDeclData::Builtin { members: vec![] }
    };

    vec![int_type, float_type, bool_type, string_type, void_type]
}

pub fn get_builtin_funcs(structs: &[Uuid]) -> Vec<FuncInfo>
{
    let println_func = FuncInfo {
        id: PRINTLN_ID.clone(),
        name: "println".into(),
        parent: None,
        has_self: false,
        parameters: vec![FuncInfoParam {
            name: "msg".into(),
            type_info: STRING_TYPE.clone(),
            init: None,
            id: Uuid::new_v4(),
        }],
        returned: VOID_TYPE.clone(),
        is_pub: true,
        decl_data: FuncDeclData::Builtin,
    };

    let print_func = FuncInfo {
        id: PRINT_ID.clone(),
        name: "print".into(),
        parent: None,
        has_self: false,
        parameters: vec![FuncInfoParam {
            name: "msg".into(),
            type_info: STRING_TYPE.clone(),
            init: None,
            id: Uuid::new_v4(),
        }],
        returned: VOID_TYPE.clone(),
        is_pub: true,
        decl_data: FuncDeclData::Builtin,
    };

    let read_line_func = FuncInfo {
        id: READ_LINE_ID.clone(),
        name: "read_line".into(),
        parent: None,
        has_self: false,
        parameters: vec![],
        returned: STRING_TYPE.clone(),
        is_pub: true,
        decl_data: FuncDeclData::Builtin,
    };

    let string_to_int_func = FuncInfo {
        id: STRING_TO_INT_ID.clone(),
        name: "to_int".into(),
        parent: Some(STRING_TYPE.clone()),
        has_self: true,
        parameters: vec![FuncInfoParam {
            name: "self".into(),
            type_info: STRING_TYPE.clone(),
            init: None,
            id: Uuid::new_v4(),
        }],
        returned: TypeInfo::Optional(Box::new(INT_TYPE.clone())),
        is_pub: true,
        decl_data: FuncDeclData::Builtin,
    };

    vec![println_func, print_func, read_line_func, string_to_int_func]
}

pub fn get_operators() -> GlobalOperators
{
    let mut operators = GlobalOperators::new();
    make_bool_type_ops(&mut operators);
    make_int_type_ops(&mut operators);
    make_float_type_ops(&mut operators);
    make_string_type_ops(&mut operators);
    add_cast_ops(&mut operators);
    add_index_ops(&mut operators);

    operators
}

fn make_int_type_ops(ops: &mut GlobalOperators)
{
    let int_type = TypeInfo::Primary(INT_ID.clone());
    add_numeric_ops(ops, int_type.clone());
    add_equality_ops(ops, int_type.clone());
}

fn make_float_type_ops(ops: &mut GlobalOperators)
{
    let float_type = TypeInfo::Primary(FLOAT_ID.clone());
    add_numeric_ops(ops, float_type.clone());
    add_equality_ops(ops, float_type.clone());
}

fn make_string_type_ops(ops: &mut GlobalOperators)
{
    let string_type = TypeInfo::Primary(STRING_ID.clone());
    add_equality_ops(ops, string_type.clone());
    
    add_string_concat_op(ops, TypeInfo::Primary(INT_ID.clone()));
    add_string_concat_op(ops, TypeInfo::Primary(BOOL_ID.clone()));
    add_string_concat_op(ops, TypeInfo::Primary(FLOAT_ID.clone()));
    add_string_concat_op(ops, TypeInfo::Primary(STRING_ID.clone()));
}

fn make_bool_type_ops(ops: &mut GlobalOperators)
{
    let bool_type = TypeInfo::Primary(BOOL_ID.clone());
    add_equality_ops(ops, bool_type.clone());
    ops.add_binary_op(BinaryOp::make_uniform(BOOL_TYPE.clone(), BinaryOpType::And));
    ops.add_binary_op(BinaryOp::make_uniform(BOOL_TYPE.clone(), BinaryOpType::Or));

    ops.add_unary_op(UnaryOp {
        input: bool_type.clone(),
        result: bool_type.clone(),
        op: UnaryOpType::Invert,
    });
}

fn add_numeric_ops(ops: &mut GlobalOperators, ty: TypeInfo)
{
    ops.add_unary_op(UnaryOp {
        input: ty.clone(),
        result: ty.clone(),
        op: UnaryOpType::Negate,
    });

    ops.add_binary_op(BinaryOp::make_uniform(ty.clone(), BinaryOpType::Plus));
    ops.add_binary_op(BinaryOp::make_uniform(ty.clone(), BinaryOpType::Minus));
    ops.add_binary_op(BinaryOp::make_uniform(ty.clone(), BinaryOpType::Multiply));
    ops.add_binary_op(BinaryOp::make_uniform(ty.clone(), BinaryOpType::Divide));
    ops.add_binary_op(BinaryOp::make_uniform(ty.clone(), BinaryOpType::Modulus));
}

fn add_equality_ops(ops: &mut GlobalOperators, ty: TypeInfo)
{
    let bool_type = TypeInfo::Primary(BOOL_ID.clone());
    ops.add_binary_op(BinaryOp::make_isosceles(ty.clone(), bool_type.clone(), BinaryOpType::Equal));
    ops.add_binary_op(BinaryOp::make_isosceles(ty.clone(), bool_type.clone(), BinaryOpType::NotEqual));
    ops.add_binary_op(BinaryOp::make_isosceles(ty.clone(), bool_type.clone(), BinaryOpType::LessThan));
    ops.add_binary_op(BinaryOp::make_isosceles(ty.clone(), bool_type.clone(), BinaryOpType::GreaterThan));
    ops.add_binary_op(BinaryOp::make_isosceles(ty.clone(), bool_type.clone(), BinaryOpType::LessThanEqual));
    ops.add_binary_op(BinaryOp::make_isosceles(ty.clone(), bool_type.clone(), BinaryOpType::GreaterThanEqual));
}

fn add_cast_ops(ops: &mut GlobalOperators)
{
    // Int as Int, ...
    ops.add_cast_op(CastOp {
        checker: Box::new(|e: &TypeInfo, c: &TypeInfo| -> Option<TypeInfo> {
            if e == c 
            {
                Some(c.clone())
            }
            else 
            {
                None    
            }
        })
    });

    // Int as Int?, ...
    ops.add_cast_op(CastOp {
        checker: Box::new(|e: &TypeInfo, c: &TypeInfo| -> Option<TypeInfo> {
            let TypeInfo::Optional(i) = c else {
                return None;
            };

            if e == &**i 
            {
                Some(c.clone())
            }
            else 
            {
                None    
            }
        })
    });
}

fn add_index_ops(ops: &mut GlobalOperators)
{
    ops.add_index_op(IndexOp {
        checker: Box::new(|i: &TypeInfo, a: &TypeInfo| -> Option<TypeInfo> {
            if a != &*INT_TYPE
            {
                return None;
            }

            let TypeInfo::Array(inner) = i else {
                return None;
            };

            Some(inner.as_ref().clone())
        })
    });

    ops.add_index_op(IndexOp {
        checker: Box::new(|i: &TypeInfo, a: &TypeInfo| -> Option<TypeInfo> {
            if i == &*STRING_TYPE && a == &*INT_TYPE
            {
                return Some(STRING_TYPE.clone());
            }
            
            None
        })
    });
}

fn add_string_concat_op(ops: &mut GlobalOperators, other: TypeInfo)
{
    let string_type = TypeInfo::Primary(STRING_ID.clone());
    let st = string_type.clone();
    let ty = other.clone();
    let checker = Box::new(move |a: &TypeInfo, b: &TypeInfo| {
        *a == st && *b == ty ||
        *a == ty && *b == st
    });

    ops.add_binary_op(BinaryOp { 
        checker, 
        result: string_type, 
        op: BinaryOpType::Plus 
    });
}