#![allow(dead_code)]
use super::*;
use microtree::{Ast, AstBuilder, Red, TokenBuilder};
use smol_str::SmolStr;

#[derive(Debug)]
pub struct Number(Red);
impl Ast for Number {
    fn new(node: Red) -> Option<Self> {
        if !node.is("number") {
            return None;
        }
        node.green().as_token()?;
        Some(Self(node))
    }

    fn red(&self) -> Red {
        self.0.clone()
    }
}

impl Number {
    pub fn build(value: u32) -> TokenBuilder<Number> {
        TokenBuilder::custom("number", value.to_string())
    }
    pub fn value(&self) -> Option<u32> {
        Some(self.0.green().as_token()?.value.parse().ok()?)
    }
}

impl IntoBuilder<Value> for TokenBuilder<Number> {
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}

#[derive(Debug)]
pub struct StringVal(Red);
impl Ast for StringVal {
    fn new(node: Red) -> Option<Self> {
        if !node.is("string") {
            return None;
        }
        node.green().as_token()?;
        Some(Self(node))
    }

    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl StringVal {
    pub fn build(value: impl Into<SmolStr>) -> StringValBuilder {
        StringValBuilder::new(value)
    }
}
pub struct StringValBuilder {
    val: SmolStr,
}

impl StringValBuilder {
    fn new(val: impl Into<SmolStr>) -> Self {
        Self { val: val.into() }
    }
}

impl AstBuilder for StringValBuilder {
    type T = StringVal;
    fn build(self, builder: &mut microtree::Cache) -> StringVal {
        StringVal::new(Red::root(self.build_green(builder))).unwrap()
    }

    fn build_green(self, builder: &mut microtree::Cache) -> microtree::Green {
        builder.token("string", self.val.to_string())
    }

    fn build_boxed_green(self: Box<Self>, builder: &mut microtree::Cache) -> microtree::Green {
        self.build_green(builder)
    }
}
