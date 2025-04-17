use crate::utils::TextPos;

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