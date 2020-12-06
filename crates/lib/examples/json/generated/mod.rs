#![allow(clippy::redundant_clone, clippy::wrong_self_convention)]
#![allow(dead_code)]
use microtree::{AliasBuilder, Ast, AstBuilder, Cache, Green, IntoBuilder, Red, TokenBuilder};

mod handwritten;
pub use handwritten::*;

#[derive(Debug)]
pub struct DQuote(Red);
impl Ast for DQuote {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "\"" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl DQuote {
    pub fn build() -> TokenBuilder<DQuote> {
        TokenBuilder::new("\"")
    }
}

#[derive(Debug)]
pub struct Comma(Red);
impl Ast for Comma {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "," {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl Comma {
    pub fn build() -> TokenBuilder<Comma> {
        TokenBuilder::new(",")
    }
}

#[derive(Debug)]
pub struct LBracket(Red);
impl Ast for LBracket {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "[" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl LBracket {
    pub fn build() -> TokenBuilder<LBracket> {
        TokenBuilder::new("[")
    }
}

#[derive(Debug)]
pub struct RBracket(Red);
impl Ast for RBracket {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "]" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl RBracket {
    pub fn build() -> TokenBuilder<RBracket> {
        TokenBuilder::new("]")
    }
}

#[derive(Debug)]
pub enum Value {
    Array(Array),
    Number(Number),
    String(String),
}
impl From<Array> for Value {
    fn from(val: Array) -> Self {
        Self::Array(val)
    }
}
impl From<Number> for Value {
    fn from(val: Number) -> Self {
        Self::Number(val)
    }
}
impl From<String> for Value {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}
impl Ast for Value {
    fn new(node: Red) -> Option<Self> {
        None.or_else(|| Array::new(node.clone()).map(Value::Array))
            .or_else(|| Number::new(node.clone()).map(Value::Number))
            .or_else(|| String::new(node.clone()).map(Value::String))
    }
    fn red(&self) -> Red {
        match &self {
            Value::Array(node) => node.red(),
            Value::Number(node) => node.red(),
            Value::String(node) => node.red(),
        }
    }
}
impl Value {
    pub fn as_array(self) -> Option<Array> {
        match self {
            Self::Array(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_number(self) -> Option<Number> {
        match self {
            Self::Number(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_string(self) -> Option<String> {
        match self {
            Self::String(val) => Some(val),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Array(Red);
impl Ast for Array {
    fn new(node: Red) -> Option<Self> {
        if !node.is("Array") {
            return None;
        }
        node.green().as_node()?;
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl Array {
    pub fn lbracket_token(&self) -> Option<LBracket> {
        self.0.children().filter_map(LBracket::new).next()
    }
    pub fn values(&self) -> impl Iterator<Item = Value> + '_ {
        self.0.children().filter_map(Value::new)
    }
    pub fn rbracket_token(&self) -> Option<RBracket> {
        self.0.children().filter_map(RBracket::new).next()
    }
    pub fn build<T0, T1, T2>() -> ArrayBuilder<T0, T1, T2>
    where
        T0: AstBuilder<T = LBracket>,
        T1: AstBuilder<T = Comma>,
        T2: AstBuilder<T = RBracket>,
    {
        Default::default()
    }
}
pub struct ArrayBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = LBracket>,
    T1: AstBuilder<T = Comma>,
    T2: AstBuilder<T = RBracket>,
{
    lbracket: Option<T0>,
    values: Vec<Box<dyn AstBuilder<T = Value>>>,
    comma: Option<T1>,
    rbracket: Option<T2>,
}
impl<T0, T1, T2> Default for ArrayBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = LBracket>,
    T1: AstBuilder<T = Comma>,
    T2: AstBuilder<T = RBracket>,
{
    fn default() -> Self {
        Self {
            lbracket: Default::default(),
            values: Default::default(),
            comma: Default::default(),
            rbracket: Default::default(),
        }
    }
}
impl<T0, T1, T2> ArrayBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = LBracket>,
    T1: AstBuilder<T = Comma>,
    T2: AstBuilder<T = RBracket>,
{
    pub fn fill(
        self,
        lbracket: T0,
        values: Vec<Box<dyn AstBuilder<T = Value>>>,
        comma: T1,
        rbracket: T2,
    ) -> Self {
        Self {
            lbracket: Some(lbracket),
            values,
            comma: Some(comma),
            rbracket: Some(rbracket),
        }
    }
}
impl<T0, T1, T2> AstBuilder for ArrayBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = LBracket>,
    T1: AstBuilder<T = Comma>,
    T2: AstBuilder<T = RBracket>,
{
    type T = Array;
    fn build(self, builder: &mut Cache) -> Array {
        let green = AstBuilder::build_green(self, builder);
        Array::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let children = None
            .into_iter()
            .chain(self.lbracket.map(|it| it.build_green(builder)).into_iter())
            .chain({
                let delit = self.comma.map(|it| it.build_green(builder));
                self.values
                    .into_iter()
                    .flat_map(|it| {
                        Some(it.build_boxed_green(builder))
                            .into_iter()
                            .chain(delit.clone().into_iter())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .chain(self.rbracket.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("Array", move |_| children)
    }
}
impl<T0, T1, T2> IntoBuilder<Value> for ArrayBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = LBracket>,
    T1: AstBuilder<T = Comma>,
    T2: AstBuilder<T = RBracket>,
{
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}

#[derive(Debug)]
pub struct String(Red);
impl Ast for String {
    fn new(node: Red) -> Option<Self> {
        if !node.is("String") {
            return None;
        }
        node.green().as_node()?;
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl String {
    pub fn left_dquote_token(&self) -> Option<DQuote> {
        self.0.children().filter_map(DQuote::new).next()
    }
    pub fn value_token(&self) -> Option<StringVal> {
        self.0.children().filter_map(StringVal::new).next()
    }
    pub fn right_dqoute_token(&self) -> Option<DQuote> {
        self.0.children().filter_map(DQuote::new).next()
    }
    pub fn build<T0, T1, T2>() -> StringBuilder<T0, T1, T2>
    where
        T0: AstBuilder<T = DQuote>,
        T1: AstBuilder<T = StringVal>,
        T2: AstBuilder<T = DQuote>,
    {
        Default::default()
    }
}
pub struct StringBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = DQuote>,
    T1: AstBuilder<T = StringVal>,
    T2: AstBuilder<T = DQuote>,
{
    left_dquote: Option<T0>,
    value: Option<T1>,
    right_dqoute: Option<T2>,
}
impl<T0, T1, T2> Default for StringBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = DQuote>,
    T1: AstBuilder<T = StringVal>,
    T2: AstBuilder<T = DQuote>,
{
    fn default() -> Self {
        Self {
            left_dquote: Default::default(),
            value: Default::default(),
            right_dqoute: Default::default(),
        }
    }
}
impl<T0, T1, T2> StringBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = DQuote>,
    T1: AstBuilder<T = StringVal>,
    T2: AstBuilder<T = DQuote>,
{
    pub fn fill(self, left_dquote: T0, value: T1, right_dqoute: T2) -> Self {
        Self {
            left_dquote: Some(left_dquote),
            value: Some(value),
            right_dqoute: Some(right_dqoute),
        }
    }
}
impl<T0, T1, T2> AstBuilder for StringBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = DQuote>,
    T1: AstBuilder<T = StringVal>,
    T2: AstBuilder<T = DQuote>,
{
    type T = String;
    fn build(self, builder: &mut Cache) -> String {
        let green = AstBuilder::build_green(self, builder);
        String::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let children = None
            .into_iter()
            .chain(
                self.left_dquote
                    .map(|it| it.build_green(builder))
                    .into_iter(),
            )
            .chain(self.value.map(|it| it.build_green(builder)).into_iter())
            .chain(
                self.right_dqoute
                    .map(|it| it.build_green(builder))
                    .into_iter(),
            )
            .collect();
        builder.node("String", move |_| children)
    }
}
impl<T0, T1, T2> IntoBuilder<Value> for StringBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = DQuote>,
    T1: AstBuilder<T = StringVal>,
    T2: AstBuilder<T = DQuote>,
{
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}
