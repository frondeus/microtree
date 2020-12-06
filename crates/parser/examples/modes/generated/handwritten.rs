#![allow(dead_code)]
use super::*;
use microtree::{Ast, Red, TokenBuilder};
use smol_str::SmolStr;

#[derive(Debug)]
pub struct Atom(Red);
impl Ast for Atom {
    fn new(node: Red) -> Option<Self> {
        if !node.is("atom") {
            return None;
        }
        node.green().as_token()?;
        Some(Self(node))
    }

    fn red(&self) -> Red {
        self.0.clone()
    }
}

impl Atom {
    pub fn build(value: impl Into<SmolStr>) -> TokenBuilder<Atom> {
        TokenBuilder::custom("atom", value)
    }
}

impl IntoBuilder<Value> for TokenBuilder<Atom> {
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}

#[derive(Debug)]
pub struct Text(Red);
impl Ast for Text {
    fn new(node: Red) -> Option<Self> {
        if !node.is("text") {
            return None;
        }
        node.green().as_token()?;
        Some(Self(node))
    }

    fn red(&self) -> Red {
        self.0.clone()
    }
}

impl Text {
    pub fn build(value: impl Into<SmolStr>) -> TokenBuilder<Text> {
        TokenBuilder::custom("text", value)
    }
}

impl IntoBuilder<StrValue> for TokenBuilder<Text> {
    fn into_builder(self) -> AliasBuilder<Self, StrValue> {
        AliasBuilder::new("StrValue", self)
    }
}
