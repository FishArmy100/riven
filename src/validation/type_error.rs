use itertools::Itertools;

use crate::{compiler::CompilerError, lexing::token::Token, utils::TextLoc};

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
        }
    }

    fn loc(&self) -> TextLoc 
    {
        match self 
        {
            TypeError::UnknownType(_, loc) => loc.clone(),
            TypeError::DuplicateTypeDef(_, loc) => loc.clone(),
            TypeError::UnknownUsing(_, loc) => loc.clone(),
            TypeError::ConflictingTypes(_, loc) => loc.clone(),
            TypeError::NoBinaryOp(_, _, _, loc) => loc.clone(),
            TypeError::NoUnaryOp(_, _, loc) => loc.clone(),
            TypeError::CannotConstruct(_, loc) => loc.clone(),
            TypeError::InvalidConstructionArgs(loc) => loc.clone(),
            TypeError::ConflictingFunctions(_, loc) => loc.clone(),
            TypeError::UndefinedFunction(_, loc) => loc.clone(),
            TypeError::ExpectedType(_, loc) => loc.clone(),
            TypeError::FunctionArgumentNeedsInitializer(loc) => loc.clone(),
            TypeError::UnknownIdentifier(_, loc) => loc.clone(),
            TypeError::ExpectedFunction(loc) => loc.clone(),
            TypeError::InvalidCallArgs(_, loc) => loc.clone(),
            TypeError::ExpectedAnIterator(loc) => loc.clone(),
            TypeError::UnknownVariable(_, loc) => loc.clone(),
            TypeError::FunctionMustReturn(loc) => loc.clone(),
            TypeError::CannotInferExpression(loc) => loc.clone(),
            TypeError::CannotDefineParentTypeTwice(loc) => loc.clone(),
            TypeError::CannotUseSelfInContext(loc) => loc.clone(),
        }
    }
}