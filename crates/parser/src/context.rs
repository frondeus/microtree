use crate::{Parser, TokenKind};

pub struct Context<'a, Tok: TokenKind> {
    pub leading_trivia: Option<&'a dyn Parser<Tok>>,
    pub trailing_trivia: Option<&'a dyn Parser<Tok>>,
}

impl<'a, Tok: TokenKind> Default for Context<'a, Tok> {
    fn default() -> Self {
        Self {
            leading_trivia: None,
            trailing_trivia: None,
        }
    }
}

impl<'a, Tok: TokenKind> Context<'a, Tok> {
    pub fn new(trivia: &'a dyn Parser<Tok>) -> Self {
        Self {
            leading_trivia: Some(trivia),
            trailing_trivia: Some(trivia),
        }
    }

    pub fn leading_trivia(&self) -> Option<&'a dyn Parser<Tok>> {
        self.leading_trivia
    }

    pub fn trailing_trivia(&self) -> Option<&'a dyn Parser<Tok>> {
        self.trailing_trivia
    }
}
