use crate::{SmolStr, TextSize};

#[derive(Debug)]
pub struct Token<Tok> {
    pub kind: Tok,
    pub value: SmolStr,
    pub offset: TextSize,
}

impl<Tok> Token<Tok> {
    pub fn end(&self) -> TextSize {
        TextSize::of(self.value.as_str()) + self.offset
    }
}
