use uuid::Uuid;

use super::StructDef;

pub const INT_TYPE: &str = "Int";
pub const FLOAT_TYPE: &str = "Float";
pub const BOOL_TYPE: &str = "Bool";
pub const STRING_TYPE: &str = "String";

lazy_static::lazy_static! 
{
    pub static ref INT_ID: Uuid = Uuid::new_v4();
    pub static ref FLOAT_ID: Uuid = Uuid::new_v4();
    pub static ref BOOL_ID: Uuid = Uuid::new_v4();
    pub static ref STRING_ID: Uuid = Uuid::new_v4();
}

pub fn get_builtin_types() -> Vec<StructDef>
{
    let int_type = StructDef {
        name: INT_TYPE.into(),
        id: INT_ID.clone(),
        members: vec![],
        is_pub: true,
    };

    let float_type = StructDef {
        name: FLOAT_TYPE.into(),
        id: FLOAT_ID.clone(),
        members: vec![],
        is_pub: true,
    };

    let bool_type = StructDef {
        name: BOOL_TYPE.into(),
        id: BOOL_ID.clone(),
        members: vec![],
        is_pub: true,
    };

    let string_type = StructDef {
        name: STRING_TYPE.into(),
        id: STRING_ID.clone(),
        members: vec![],
        is_pub: true,
    };

    vec![int_type, float_type, bool_type, string_type]
}