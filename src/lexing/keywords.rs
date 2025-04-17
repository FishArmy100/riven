use std::collections::HashMap;

use super::token::TokenType;

pub const KW_AS:        &str = "as";
pub const KW_BREAK:     &str = "break";
pub const KW_CONST:     &str = "const";
pub const KW_CONTINUE:  &str = "continue";
pub const KW_ELSE:      &str = "else";
pub const KW_FALSE:     &str = "false";
pub const KW_FN:        &str = "fn";
pub const KW_FN_TYPE:   &str = "Fn";
pub const KW_FOR:       &str = "for";
pub const KW_IF:        &str = "if";
pub const KW_IN:        &str = "in";
pub const KW_LET:       &str = "let";
pub const KW_RETURN:    &str = "return";
pub const KW_PUB:       &str = "pub";
pub const KW_SELF:      &str = "self";
pub const KW_STRUCT:    &str = "struct";
pub const KW_TRUE:      &str = "true";
pub const KW_USE:       &str = "use";
pub const KW_WHILE:     &str = "while";

lazy_static::lazy_static! 
{
    pub static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut map = HashMap::new();
        map.insert(KW_AS.into(), TokenType::As);
        map.insert(KW_BREAK.into(), TokenType::Break);
        map.insert(KW_CONST.into(), TokenType::Const);
        map.insert(KW_ELSE.into(), TokenType::Else);
        map.insert(KW_FALSE.into(), TokenType::False);
        map.insert(KW_FN.into(), TokenType::Fn);
        map.insert(KW_FN_TYPE.into(), TokenType::FnType);
        map.insert(KW_FOR.into(), TokenType::For);
        map.insert(KW_IF.into(), TokenType::If);
        map.insert(KW_IN.into(), TokenType::In);
        map.insert(KW_LET.into(), TokenType::Let);
        map.insert(KW_RETURN.into(), TokenType::Return);
        map.insert(KW_PUB.into(), TokenType::Pub);
        map.insert(KW_STRUCT.into(), TokenType::Struct);
        map.insert(KW_TRUE.into(), TokenType::True);
        map.insert(KW_USE.into(), TokenType::Use);
        map.insert(KW_WHILE.into(), TokenType::While);
        map
    };
}