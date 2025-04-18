use std::{fs::{create_dir_all, File, OpenOptions}, io::{Read, Write}, ops::Add};

pub fn read_file(path: &str) -> Result<String, String> 
{
    let Ok(mut file) = File::open(path) else {
        return Err(format!("Could not open file: `{}`", path));
    };

    let mut contents = String::new();
    if let Err(_) = file.read_to_string(&mut contents)
    {
        return Err(format!("Invalid file format"));
    }

    Ok(contents)
}

pub fn write_file(filename: &str, content: &str) -> Result<(), String> {
    // Ensure the parent directory exists
    if let Some(parent) = std::path::Path::new(filename).parent() {
        if let Err(e) = create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)
    else {
        return Err("Failed to create file".into());
    };

    let Ok(()) = file.write_all(content.as_bytes()) else {
        return Err("Failed to write to file".into());
    };

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextPos
{
    pub begin: usize,
    pub end: usize
}

impl TextPos
{
    pub fn new(begin: usize, end: usize) -> Self 
    {
        Self 
        {
            begin,
            end,
        }
    }

    pub fn uniform(index: usize) -> Self 
    {
        Self 
        { 
            begin: index, 
            end: index 
        }
    }

    pub fn get_loc(&self, text: &[char]) -> TextLoc
    {
        let mut line = 1;
        let mut column = 1;

        let line_count = text.iter().filter(|f| **f == '\n').count() + 1;

        if self.begin >= text.len()
        {
            return TextLoc { line: line_count, column };
        }

        for i in 0..=self.begin
        {
            if text[i] == '\n'
            {
                line += 1;
                column = 0;
            }
            else 
            {
                column += 1;
            }
        }

        TextLoc { line, column }
    }
}

impl Add for TextPos
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output 
    {
        Self 
        {
            begin: self.begin.min(rhs.begin),
            end: self.end.max(rhs.end),
        }
    }
}

impl From<usize> for TextPos
{
    fn from(value: usize) -> Self 
    {
        Self::uniform(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextLoc
{
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for TextLoc
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}:{}", self.line, self.column)
    }
}

pub fn partition_errors<T, E>(results: impl IntoIterator<Item = Result<T, E>>) -> Result<Vec<T>, Vec<E>>
{
    let mut oks = Vec::new();
    let mut errs = Vec::new();

    for result in results {
        match result {
            Ok(val) => oks.push(val),
            Err(err) => errs.push(err),
        }
    }

    if errs.is_empty() 
    {
        Ok(oks)
    } 
    else 
    {
        Err(errs)
    }
}