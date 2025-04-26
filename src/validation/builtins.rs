use uuid::Uuid;

use crate::validation::info::struct_info::StructDeclData;

use super::{info::{struct_info::StructInfo, types::TypeInfo}, operators::{BinaryOp, BinaryOpType, GlobalOperators, UnaryOp, UnaryOpType}};

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
}

pub fn get_builtins() -> Vec<StructInfo>
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

pub fn get_operators() -> GlobalOperators
{
    let mut operators = GlobalOperators::new();
    make_bool_type_ops(&mut operators);
    make_int_type_ops(&mut operators);
    make_float_type_ops(&mut operators);
    make_string_type_ops(&mut operators);

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