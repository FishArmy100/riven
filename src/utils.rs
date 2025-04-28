use std::{cell::{Ref, RefCell, RefMut}, fs::{create_dir_all, File, OpenOptions}, io::{Read, Write}, ops::Add, path::Path, rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use uuid::Uuid;

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

    pub fn get_loc(&self, file: &FileInfo) -> TextLoc
    {
        let mut line = 1;
        let mut column = 1;

        let line_count = file.chars.iter().filter(|f| **f == '\n').count() + 1;

        if self.begin >= file.chars.len()
        {
            return TextLoc { line: line_count, column, file: file.path.clone() };
        }

        for i in 0..=self.begin
        {
            if file.chars[i] == '\n'
            {
                line += 1;
                column = 0;
            }
            else 
            {
                column += 1;
            }
        }

        TextLoc { line, column, file: file.path.clone() }
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

#[derive(Debug, Clone)]
pub struct TextLoc
{
    pub line: usize,
    pub column: usize,
    pub file: PathInfo,
}

impl std::fmt::Display for TextLoc
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{}:{}:{}", self.file.full_path.as_ref().unwrap_or(&self.file.relative_path), self.line, self.column)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathInfo
{
    pub full_path: Option<String>,
    pub relative_path: String,
}

impl PathInfo
{
    pub fn split_relative(&self) -> Vec<String>
    {
        let path = Path::new(&self.relative_path);
        let mut parts: Vec<String> = Vec::new();

        for component in path.parent().unwrap_or_else(|| Path::new("")).components() {
            parts.push(component.as_os_str().to_string_lossy().to_string());
        }

        // Add file stem (filename without extension) if it exists
        if let Some(stem) = path.file_stem() {
            parts.push(stem.to_string_lossy().to_string());
        }

        parts
    }
}

#[derive(Debug)]
pub struct FileInfo 
{
    pub id: Uuid,
    pub path: PathInfo,
    pub chars: Vec<char>,
}

impl FileInfo
{
    pub fn new(src: String, path: PathInfo) -> Self 
    {
        FileInfo { 
            id: Uuid::new_v4(), 
            path, 
            chars: src.chars().collect(), 
        }
    }

    pub fn read(path: &str, code_path: &str) -> Result<Self, String>
    {
        let src = read_file(path)?;
        let path = PathInfo {
            full_path: Some(path.to_string()),
            relative_path: code_path.into()
        };

        Ok(Self::new(src, path))
    }

    pub fn from_text(src: &str, path: String) -> Self 
    {
        let path = PathInfo {
            full_path: None,
            relative_path: path.to_string(),
        };

        Self::new(src.to_string(), path)
    }

    pub fn as_arc(self) -> Arc<Self>
    {
        Arc::new(self)
    }
}

#[derive(Debug)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T>
{
    pub fn new(v: T) -> Self
    {
        Self(Rc::new(RefCell::new(v)))
    }

    pub fn get(&self) -> Ref<'_, T>
    {
        self.0.borrow()
    }

    pub fn get_mut(&self) -> RefMut<'_, T>
    {
        self.0.borrow_mut()
    }

    pub fn inner(&self) -> &Rc<RefCell<T>>
    {
        &self.0
    }
}

impl<T> From<Rc<RefCell<T>>> for Shared<T>
{
    fn from(value: Rc<RefCell<T>>) -> Self 
    {
        Self(value)
    }
}

impl<T> From<Shared<T>> for Rc<RefCell<T>>
{
    fn from(value: Shared<T>) -> Self 
    {
        value.0
    }
}

impl<T> Clone for Shared<T>
{
    fn clone(&self) -> Self 
    {
        Self(self.0.clone())
    }
}