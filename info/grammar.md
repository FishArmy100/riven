# Cirrus Grammar

### Utilities
```fs
typeName  	-> IDENTIFIER;
            | "[" "]" typeName 
			| "Fn" "(" ( typeName ("," typeName)* )? ")"  "->" typeName;


arguments       -> expression ("," expression)* ","?;
parameters      -> IDENTIFIER ":" typeName ("=" expression)? (IDENTIFIER ":" typeName ("=" expression)?)* ","?;
```

### Expressions
```fs
lambda -> (IDENTIFIER | "|" (IDENTIFIER (":" typeName)? ("," IDENTIFIER (":" typeName)?)* ","? "|") ("->" typeName)?) "=>" (expression | blockStmt);
arrayLiteral -> "[" (expression ("," expression)* ","? )? "]"
primary -> NUMBER
        | STRING
        | IDENTIFIER
        | "(" expression ")"
        | "self"
        | "true"
        | "false"
        | lambda
		| typeName "{" (IDENTIFIER ":" expression ("," IDENTIFIER ":" expression)? "," )? "}";
        | typeName "." IDENTIFIER // NOTE: only with type that has a function or array in it
        | arrayLiteral

call        -> primary ( genericArgs "(" arguments? ")" | "[" expression "]" | "." IDENTIFIER )*;
unary       -> ("!" | "-") unary | call;
factor      -> unary ( ( "/" | "*" ) unary )* ;
term        -> factor ( ( "+" | "-" ) factor )* ;
comparison  -> term ( ( ">" | "<" | ">=" | "<=" ) term )* ;
equality    -> comparison ( ( "==" | "!=" ) comparison )* ;
logicalAnd  -> equality ( "||" equality)* ;
logicalOr   -> logicalAnd ( "&&" logicalAnd)* ;

expression  -> logicalOr;
```

### Statements
```fs
useStmt 	-> "use" IDENTIFIER ("." IDENTIFIER)* ";";
exprStmt	-> expression ";";
letStmt     -> "let" IDENTIFIER (":" typeName)? "=" expression ";";
constStmt   -> "pub"? "const" IDENTIFIER ":" TypeName "=" expression ";";
assignStmt  -> typeName? ("." IDENTIFIER)* "=" expression ";";

blockStmt   -> "{" statement* "}";
ifStmt      -> "if" letCondition blockStmt ("else" (ifStmt | blockStmt))?;

fnDecl      -> "pub"? "fn" (typeName ".")? IDENTIFIER "(" parameters? ")" ("->" typeName)? ("for" typeName)?  blockStmt;

structParam	-> IDENTIFIER ":" typeName ("=" expression)?
structDecl  -> "pub"? "struct" IDENTIFIER genericParams? "{" ( structParam ("," structParam)* )? "}";

breakStmt   -> "break"+ ";";
breakStmt   -> "continue"+ ";";

forStmt     -> "for" pattern "in" expression blockStmt;
whileStmt   -> "while" letCondition blockStmt;

statement	-> letStmt | assignStmt | ifStmt | blockStmt | exprStmt | constStmt | whileStmt | forStmt | returnStmt | breakStmt | continueStmt;
declaration	-> fnDecl | structDecl | constStmt;
file        -> useStmt* | declaration* EOF;
program     -> file*;
```