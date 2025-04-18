use crate::{lexing::{self, token::{Token, TokenType}}, parsing::{self, ast::FileNode, ParserError}, utils::TextPos};

pub trait CompilerError
{
    fn msg(&self) -> String;
    fn pos(&self) -> Option<TextPos>;

    fn format_error(&self, text: &[char], file: Option<&str>) -> String 
    {
        let loc = self.pos().map(|p| p.get_loc(text).to_string()).unwrap_or("None".into());
        match &file {
            Some(file) => format!("[{}:{}]: {}", file.to_string(), loc, self.msg()),
            None => format!("[{}]: \"{}\"", loc, self.msg())
        }
    }
}

pub fn run_lexer(text: &[char], file: Option<&str>) -> Result<Vec<Token>, Vec<String>>
{
    match lexing::lex_text(text)
    {
        Ok(ok) => Ok(ok),
        Err(err) => {
            Err(err.iter().map(|e| e.format_error(text, file)).collect())
        },
    }
}

pub fn run_parser(text: &[char], file: Option<&str>) -> Result<FileNode, Vec<String>>
{
    let tokens = run_lexer(text, file)?;

    match parsing::parse_file(&tokens)
    {
        Ok(Some(ok)) => Ok(ok),
        Ok(None) => {
            let error = ParserError::ExpectedToken(TokenType::EOF, None).format_error(text, file);
            Err(vec![error])
        }
        Err(err) => {
            Err(err.iter().map(|e| e.format_error(text, file)).collect())
        }
    }
}