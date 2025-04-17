pub struct CharReader
{
    chars: Vec<char>,
    index: usize,
}

impl CharReader
{
    pub fn chars(&self) -> &[char]
    {
        &self.chars
    }

    pub fn index(&self) -> usize
    {
        self.index
    }

    pub fn new(text: &str) -> Option<Self>
    {
        if text.len() == 0 { return None; }

        Some(Self 
        {
            chars: text.chars().collect(),
            index: 0,
        })
    }

    pub fn peek(&self, count: usize) -> Option<char>
    {
        if self.index + count < self.chars.len()
        {
            Some(self.chars[self.index + count])
        }
        else 
        {
            None    
        }
    }

    pub fn current(&self) -> Option<char> 
    {
        if self.index < self.chars.len()
        {
            Some(self.chars[self.index])
        }
        else 
        {
            None    
        }
    }

    pub fn current_is(&self, chars: &[char]) -> bool
    {
        self.current().is_some_and(|c| chars.contains(&c))
    }

    pub fn advance(&mut self) -> Option<char> 
    {
        if !self.at_end()
        {
            let c = self.current().unwrap();
            self.index += 1;
            Some(c)
        }
        else 
        {
            None    
        }
    }

    pub fn at_end(&self) -> bool 
    {
        self.index >= self.chars.len()
    }

    pub fn check(&mut self, cs: &[char]) -> Option<char> 
    {
        let Some(c) = self.current() else { return None };

        if cs.contains(&c)
        {
            self.advance()
        }
        else 
        {
            None    
        }
    }

    pub fn check_many(&mut self, cs: &str) -> Option<String>
    {
        let all_match = cs.chars().enumerate().map(|(i, c)| self.peek(i) == Some(c)).all(|b| b);
        if all_match
        {
            for _ in 0..cs.len()
            {
                self.advance();
            }
            
            Some(cs.to_owned())
        }
        else 
        {
            None    
        }
    }

    pub fn read_spaces(&mut self) -> bool
    {
        let mut read = false;
        while self.current().is_some_and(|c| c.is_whitespace())
        {
            read = true;
            self.advance();
        }

        read
    }

    pub fn advance_count(&mut self, c: char) -> usize
    {
        let mut count = 0;
        while let Some(_) = self.check(&[c])
        {
            count += 1;
        }

        count
    }
}