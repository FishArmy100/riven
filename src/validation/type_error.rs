use itertools::Itertools;

use crate::{compiler::CompilerError, lexing::token::Token, parsing::ast::TypeName, utils::TextLoc};

use super::operators::{BinaryOpType, UnaryOpType};

#[derive(Debug, Clone)]
pub enum TypeError
{
    UnknownType(Token, TextLoc),
    DuplicateTypeDef(Token, TextLoc),
    UnknownUsing(Vec<String>, TextLoc),
    ConflictingTypes(Token, TextLoc),
    NoBinaryOp(BinaryOpType, String, String, TextLoc),
    NoUnaryOp(UnaryOpType, String, TextLoc),
    CannotConstruct(String, TextLoc),
    InvalidConstructionArgs(TextLoc),
    ConflictingFunctions(Token, TextLoc),
    UndefinedFunction(Token, TextLoc),
    ExpectedType(String, TextLoc),
    FunctionArgumentNeedsInitializer(TextLoc),
    UnknownIdentifier(String, TextLoc),
    ExpectedFunction(TextLoc),
    InvalidCallArgs(Vec<String>, TextLoc),
    ExpectedAnIterator(TextLoc),
    UnknownVariable(String, TextLoc),
    FunctionMustReturn(TextLoc),
    CannotInferExpression(TextLoc),
    CannotDefineParentTypeTwice(TextLoc),
    CannotUseSelfInContext(TextLoc),
    NoCastOperator
    {
        loc: TextLoc,
        expr_type: String,
        cast_type: String,
    },
    NoIndexOperator
    {
        loc: TextLoc,
        indexed_type: String,
        arg_type: String,
    },
    InvalidLambdaExpressionFormat(TextLoc),
    ExpressionNotAssignable(TextLoc),
    DuplicateMainFn(TextLoc),
    NoMainFn,
    InvalidMainArgs(TextLoc),
    NoMember
    {
        type_name: String,
        member: String,
        loc: TextLoc,
    }
}

impl CompilerError for TypeError
{
    fn msg(&self) -> String 
    {
        match self 
        {
            TypeError::UnknownType(token, _) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::DuplicateTypeDef(token, _) => format!("Duplicate type {}", token.value_string().unwrap()),
            TypeError::UnknownUsing(path, _) => format!("Using path {} does not exist", path.iter().join(".")),
            TypeError::ConflictingTypes(token, _) => format!("Conflicting type definitions for {}", token.value_string().unwrap()),
            TypeError::NoBinaryOp(op, left, right, _) => format!("No binary operator {} for types {} and {}", op.to_string(), left, right),
            TypeError::NoUnaryOp(op, t, _) => format!("No unary operator {} for type {}", op.to_string(), t),
            TypeError::CannotConstruct(t, _) => format!("Cannot construct type {}", t),
            TypeError::InvalidConstructionArgs(_) => format!("Invalid construction args"),
            TypeError::ConflictingFunctions(token, _) => format!("Conflicting function definitions for {}", token.value_string().unwrap()),
            TypeError::UndefinedFunction(token, _) => format!("Unknown type {}", token.value_string().unwrap()),
            TypeError::ExpectedType(t, _) => format!("Expected type {}", t),
            TypeError::FunctionArgumentNeedsInitializer(_) => format!("Must have a function initializer"),
            TypeError::UnknownIdentifier(id, _) => format!("Unknown identifier {}", id),
            TypeError::ExpectedFunction(_) => format!("Expected a function"),
            TypeError::InvalidCallArgs(items, _) => format!("Expected call args: {}", items.iter().join(", ")),
            TypeError::ExpectedAnIterator(_) => format!("Expected an iterator"),
            TypeError::UnknownVariable(name, _) => format!("Unknown variable {}", name),
            TypeError::FunctionMustReturn(_) => format!("Function must return"),
            TypeError::CannotInferExpression(_) => format!("Cannot infer expression type"),
            TypeError::CannotDefineParentTypeTwice(_) => format!("Cannot define parent type multiple times"),
            TypeError::CannotUseSelfInContext(_) => format!("Cannot use `self`, in this context"),
            TypeError::NoCastOperator { loc: _, expr_type, cast_type } => format!("No cast operator from type {} to type {}", expr_type, cast_type),
            TypeError::InvalidLambdaExpressionFormat(_) => format!("Invalid lambda expression format; please use the format `|...| -> ... => {{}}`, other variants will be added in upcoming versions"),
            TypeError::NoIndexOperator { loc: _, indexed_type, arg_type } => format!("No index operator {}[{}]", indexed_type, arg_type),
            TypeError::ExpressionNotAssignable(_) => format!("Expression type is not assignable"),
            TypeError::DuplicateMainFn(_) => format!("Duplicate main function found"),
            TypeError::NoMainFn => format!("No main function found"),
            TypeError::InvalidMainArgs(_) => format!("Invalid main args"),
            TypeError::NoMember { type_name, member, loc: _ } => format!("No member {} on type {}", member, type_name),
        }
    }

    fn loc(&self) -> Option<TextLoc> 
    {
        match self 
        {
            TypeError::UnknownType(_, loc) => Some(loc.clone()),
            TypeError::DuplicateTypeDef(_, loc) => Some(loc.clone()),
            TypeError::UnknownUsing(_, loc) => Some(loc.clone()),
            TypeError::ConflictingTypes(_, loc) => Some(loc.clone()),
            TypeError::NoBinaryOp(_, _, _, loc) => Some(loc.clone()),
            TypeError::NoUnaryOp(_, _, loc) => Some(loc.clone()),
            TypeError::CannotConstruct(_, loc) => Some(loc.clone()),
            TypeError::InvalidConstructionArgs(loc) => Some(loc.clone()),
            TypeError::ConflictingFunctions(_, loc) => Some(loc.clone()),
            TypeError::UndefinedFunction(_, loc) => Some(loc.clone()),
            TypeError::ExpectedType(_, loc) => Some(loc.clone()),
            TypeError::FunctionArgumentNeedsInitializer(loc) => Some(loc.clone()),
            TypeError::UnknownIdentifier(_, loc) => Some(loc.clone()),
            TypeError::ExpectedFunction(loc) => Some(loc.clone()),
            TypeError::InvalidCallArgs(_, loc) => Some(loc.clone()),
            TypeError::ExpectedAnIterator(loc) => Some(loc.clone()),
            TypeError::UnknownVariable(_, loc) => Some(loc.clone()),
            TypeError::FunctionMustReturn(loc) => Some(loc.clone()),
            TypeError::CannotInferExpression(loc) => Some(loc.clone()),
            TypeError::CannotDefineParentTypeTwice(loc) => Some(loc.clone()),
            TypeError::CannotUseSelfInContext(loc) => Some(loc.clone()),
            TypeError::NoCastOperator { loc, expr_type: _, cast_type: _ } => Some(loc.clone()),
            TypeError::InvalidLambdaExpressionFormat(loc) => Some(loc.clone()),
            TypeError::NoIndexOperator { loc, indexed_type: _, arg_type: _ } => Some(loc.clone()),
            TypeError::ExpressionNotAssignable(loc) => Some(loc.clone()),
            TypeError::DuplicateMainFn(text_loc) => Some(text_loc.clone()),
            TypeError::NoMainFn => None,
            TypeError::InvalidMainArgs(text_loc) => Some(text_loc.clone()),
            TypeError::NoMember { type_name: _, member: _, loc } => Some(loc.clone()),
        }
    }
}