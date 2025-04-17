# Cirrus Grammar

### Utilities
```fs
typeName  	-> (IDENTIFIER | Self) genericArgs?;
            | "[" "]" typeName 
			| "fn" "(" ( typeName ("," typeName)* )? ")"  "->" typeName
            | typeName "." IDENTIFIER genericArgs?;


arguments       -> expression ("," expression)* ","?;
genericParams   -> "[" IDENTIFIER ( "," IDENTIFIER )* "," "]";
genericArgs     -> "[" typeName ("," typeName)* ","? "]";
parameters      -> "var"? IDENTIFIER ":" typeName ("=" expression)? (IDENTIFIER ":" typeName ("=" expression)?)* ","?;

patternField    -> "mut"? IDENTIFIER (":" pattern)?;
patternFields	-> patternField ("," patternField)* ","?;
pattern			-> NUMBER 
                | STRING 
                | ("mut"? IDENTIFIER) 
                | typeName ("(" pattern ")")?  // destructured enum
                | typeName ( "{" patternFields? "}"  // destructured struct
                | typeName "." IDENTIFIER
                | "[" ( pattern ("," pattern)* )? "]" ); // destructured array

letCondition    -> expression | "let" pattern "=" expression ("&&" letCondition )?;
```

### Expressions
```fs
lambda -> (IDENTIFIER | "|" (IDENTIFIER (":" typeName)? ("," IDENTIFIER (":" typeName)?)* ","? "|") ("->" typeName)?) "=>" expression;
arrayLiteral -> "[" (expression ("," expression)* ","? )? "]"
primary -> NUMBER
        | STRING
        | IDENTIFIER
        | "(" expression ")"
        | "self"
        | "true"
        | "false"
        | blockExpr
        | lambda
		| typeName "{" (IDENTIFIER ":" expression ("," IDENTIFIER ":" expression)? "," )? "}";
        | typeName "(" expression ")" // NOTE: only with type that has a function or array in it
        | arrayLiteral

call        -> primary ( genericArgs "(" arguments? ")" | "[" expression "]" | "." IDENTIFIER )*;
unary       -> ("!" | "-") unary | call;
factor      -> unary ( ( "/" | "*" ) unary )* ;
term        -> factor ( ( "+" | "-" ) factor )* ;
comparison  -> term ( ( ">" | "<" | ">=" | "<=" ) term )* ;
equality    -> comparison ( ( "==" | "!=" ) comparison )* ;
logicalAnd  -> equality ( "||" equality)* ;
logicalOr   -> logicalAnd ( "&&" logicalAnd)* ;

blockExpr   -> "{" statement* expression? "}";

ifExpr      -> "if" letCondition blockExpr ("else" (ifExpr | blockExpr))?;
matchExpr   -> "match" expression "{" pattern "=>" expression ("," pattern "=>" expression)* ","? "}";

expression  -> logicalOr | ifExpr | matchExpr;
```

### Statements
```fs
useStmt 	-> "use" IDENTIFIER ("." IDENTIFIER)* ("." "*")? ";";
exprStmt	-> expression ";";
letStmt     -> "let" pattern (":" typeName)? "=" expression ("else" blockExpr)? ";";
assignStmt  -> typeName? ("." IDENTIFIER)* "=" expression ";";

whereSubClause  -> IDENTIFIER ":" typeName ( "+" typeName)*;
whereClause -> "where" whereSubClause ("," whereSubClause) ",";
fnDecl      -> "fn" IDENTIFIER genericParams? "(" parameters? ")" ("->" typeName)? whereClause? "{" statement* expression? "}";

structParam	-> "pub"? "mut"? IDENTIFIER ":" typeName ("=" expression)?
structDecl  -> "struct" IDENTIFIER genericParams? "{" ( structParam ("," structParam)* )? "}";
interfaceDecl -> "interface" IDENTIFIER genericParams? "{" ( "fn" IDENTIFIER genericParams? "(" parameters? ")" ( "->" typeName)? whereClause? ";")* "}";
enumDecl	-> "enum" IDENTIFIER genericParams? whereClause? "{" IDENTIFIER ( "(" typeName ")" | "{" parameters? "}") "}";
typeDecl	-> "type" IDENTIFIER genericParams? "=" typeName ";";

implStmt    -> "impl" genericParams? typeName ("for" typeName)? "{" ("pub"? (fnDecl | typeDecl | letStmt | constStmt))* "}";

breakStmt   -> "break"+ ";";
breakStmt   -> "continue"+ ";";

forStmt     -> "for" pattern "in" expression blockExpr;
whileStmt   -> "while" letCondition blockExpr;

statement	-> letStmt | assignStmt | ifExpr | matchExpr | blockExpr | exprStmt | useStmt;
declaration	-> "pub" (fnDecl | structDecl | interfaceDecl | enumDecl | typeDecl | letStmt | constStmt | useStmt) | implStmt;
program -> declStmt* EOF;
```