use crate::{SmolStr, TextLen, TextRange, TextSize};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Input {
    str: Box<str>,
    range: TextRange,
}

impl Input {
    pub fn chomp(&mut self, len: usize) -> SmolStr {
        let range = match self
            .as_ref()
            .char_indices()
            .nth(len - 1)
            .and_then(|(last, c)| TextSize::try_from(last + c.len_utf8()).ok())
        {
            Some(last) => TextRange::new(self.range.start(), self.range.start() + last),
            None => self.range,
        };
        self.set_cursor(range.end());

        self.str_for_range(range)
    }

    pub fn cursor(&self) -> TextSize {
        self.range.start()
    }

    pub fn set_cursor(&mut self, cursor: TextSize) {
        self.range = TextRange::new(cursor, self.range.end());
    }

    pub fn set_range(&mut self, range: TextRange) {
        self.range = range;
    }

    pub fn str_for_range(&self, range: TextRange) -> SmolStr {
        SmolStr::new(&self.str[range])
    }
}

impl From<&'_ str> for Input {
    fn from(input: &str) -> Self {
        let str: Box<str> = Box::from(input);
        Self {
            str,
            range: TextRange::up_to(input.text_len()),
        }
    }
}

impl AsRef<str> for Input {
    fn as_ref(&self) -> &str {
        &self.str[self.range]
    }
}
