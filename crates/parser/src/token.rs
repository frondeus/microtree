use crate::{SmolStr, TextRange};

#[derive(Debug)]
pub struct Token<Tok> {
    pub token: Tok,
    pub value: SmolStr,
    pub range: TextRange,
}
