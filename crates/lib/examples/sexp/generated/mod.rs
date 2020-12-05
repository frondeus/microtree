#![allow(clippy::redundant_clone, clippy::wrong_self_convention)]
#![allow(dead_code)]
use microtree::{
    AliasBuilder, Ast, AstBuilder, Green, GreenBuilder, IntoBuilder, Red, TokenBuilder,
};

mod handwritten;
pub use handwritten::*;

#[derive(Debug)]
pub struct LParen(Red);
impl Ast for LParen {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "(" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl LParen {
    pub fn build() -> TokenBuilder<LParen> {
        TokenBuilder::new("(")
    }
}

#[derive(Debug)]
pub struct RParen(Red);
impl Ast for RParen {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != ")" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl RParen {
    pub fn build() -> TokenBuilder<RParen> {
        TokenBuilder::new(")")
    }
}

#[derive(Debug)]
pub struct Dot(Red);
impl Ast for Dot {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "." {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl Dot {
    pub fn build() -> TokenBuilder<Dot> {
        TokenBuilder::new(".")
    }
}

#[derive(Debug)]
pub enum Value {
    Nil(Nil),
    Atom(Atom),
    List(List),
    Cons(Cons),
}
impl From<Nil> for Value {
    fn from(val: Nil) -> Self {
        Self::Nil(val)
    }
}
impl From<Atom> for Value {
    fn from(val: Atom) -> Self {
        Self::Atom(val)
    }
}
impl From<List> for Value {
    fn from(val: List) -> Self {
        Self::List(val)
    }
}
impl From<Cons> for Value {
    fn from(val: Cons) -> Self {
        Self::Cons(val)
    }
}
impl Ast for Value {
    fn new(node: Red) -> Option<Self> {
        None.or_else(|| Nil::new(node.clone()).map(Value::Nil))
            .or_else(|| Atom::new(node.clone()).map(Value::Atom))
            .or_else(|| List::new(node.clone()).map(Value::List))
            .or_else(|| Cons::new(node.clone()).map(Value::Cons))
    }
    fn red(&self) -> Red {
        match &self {
            Value::Nil(node) => node.red(),
            Value::Atom(node) => node.red(),
            Value::List(node) => node.red(),
            Value::Cons(node) => node.red(),
        }
    }
}
impl Value {
    pub fn as_nil(self) -> Option<Nil> {
        match self {
            Self::Nil(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_atom(self) -> Option<Atom> {
        match self {
            Self::Atom(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_list(self) -> Option<List> {
        match self {
            Self::List(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_cons(self) -> Option<Cons> {
        match self {
            Self::Cons(val) => Some(val),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Nil(Red);
impl Ast for Nil {
    fn new(node: Red) -> Option<Self> {
        if !node.is("Nil") {
            return None;
        }
        node.green().as_node()?;
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl Nil {
    pub fn lparen_token(&self) -> Option<LParen> {
        self.0.children().filter_map(LParen::new).next()
    }
    pub fn rparen_token(&self) -> Option<RParen> {
        self.0.children().filter_map(RParen::new).next()
    }
    pub fn build<T0, T1>() -> NilBuilder<T0, T1>
    where
        T0: AstBuilder<T = LParen>,
        T1: AstBuilder<T = RParen>,
    {
        Default::default()
    }
}
pub struct NilBuilder<T0, T1>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = RParen>,
{
    lparen: Option<T0>,
    rparen: Option<T1>,
}
impl<T0, T1> Default for NilBuilder<T0, T1>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = RParen>,
{
    fn default() -> Self {
        Self {
            lparen: Default::default(),
            rparen: Default::default(),
        }
    }
}
impl<T0, T1> NilBuilder<T0, T1>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = RParen>,
{
    pub fn fill(self, lparen: T0, rparen: T1) -> Self {
        Self {
            lparen: Some(lparen),
            rparen: Some(rparen),
        }
    }
}
impl<T0, T1> AstBuilder for NilBuilder<T0, T1>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = RParen>,
{
    type T = Nil;
    fn build(self, builder: &mut GreenBuilder) -> Nil {
        let green = AstBuilder::build_green(self, builder);
        Nil::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut GreenBuilder) -> Green {
        let children = None
            .into_iter()
            .chain(self.lparen.map(|it| it.build_green(builder)).into_iter())
            .chain(self.rparen.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("Nil", move |_| children)
    }
}
impl<T0, T1> IntoBuilder<Value> for NilBuilder<T0, T1>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = RParen>,
{
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}

#[derive(Debug)]
pub struct List(Red);
impl Ast for List {
    fn new(node: Red) -> Option<Self> {
        if !node.is("List") {
            return None;
        }
        node.green().as_node()?;
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl List {
    pub fn lparen_token(&self) -> Option<LParen> {
        self.0.children().filter_map(LParen::new).next()
    }
    pub fn values(&self) -> impl Iterator<Item = Value> + '_ {
        self.0.children().filter_map(Value::new)
    }
    pub fn rparen_token(&self) -> Option<RParen> {
        self.0.children().filter_map(RParen::new).next()
    }
    pub fn build<T0, T2>() -> ListBuilder<T0, T2>
    where
        T0: AstBuilder<T = LParen>,
        T2: AstBuilder<T = RParen>,
    {
        Default::default()
    }
}
pub struct ListBuilder<T0, T2>
where
    T0: AstBuilder<T = LParen>,
    T2: AstBuilder<T = RParen>,
{
    lparen: Option<T0>,
    values: Vec<Box<dyn AstBuilder<T = Value>>>,
    rparen: Option<T2>,
}
impl<T0, T2> Default for ListBuilder<T0, T2>
where
    T0: AstBuilder<T = LParen>,
    T2: AstBuilder<T = RParen>,
{
    fn default() -> Self {
        Self {
            lparen: Default::default(),
            values: Default::default(),
            rparen: Default::default(),
        }
    }
}
impl<T0, T2> ListBuilder<T0, T2>
where
    T0: AstBuilder<T = LParen>,
    T2: AstBuilder<T = RParen>,
{
    pub fn fill(self, lparen: T0, values: Vec<Box<dyn AstBuilder<T = Value>>>, rparen: T2) -> Self {
        Self {
            lparen: Some(lparen),
            values,
            rparen: Some(rparen),
        }
    }
}
impl<T0, T2> AstBuilder for ListBuilder<T0, T2>
where
    T0: AstBuilder<T = LParen>,
    T2: AstBuilder<T = RParen>,
{
    type T = List;
    fn build(self, builder: &mut GreenBuilder) -> List {
        let green = AstBuilder::build_green(self, builder);
        List::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut GreenBuilder) -> Green {
        let children = None
            .into_iter()
            .chain(self.lparen.map(|it| it.build_green(builder)).into_iter())
            .chain({
                self.values
                    .into_iter()
                    .map(|it| it.build_boxed_green(builder))
                    .collect::<Vec<_>>()
            })
            .chain(self.rparen.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("List", move |_| children)
    }
}
impl<T0, T2> IntoBuilder<Value> for ListBuilder<T0, T2>
where
    T0: AstBuilder<T = LParen>,
    T2: AstBuilder<T = RParen>,
{
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}

#[derive(Debug)]
pub struct Cons(Red);
impl Ast for Cons {
    fn new(node: Red) -> Option<Self> {
        if !node.is("Cons") {
            return None;
        }
        node.green().as_node()?;
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl Cons {
    pub fn lparen_token(&self) -> Option<LParen> {
        self.0.children().filter_map(LParen::new).next()
    }
    pub fn car(&self) -> Option<Value> {
        self.0.children().filter_map(Value::new).next()
    }
    pub fn dot_token(&self) -> Option<Dot> {
        self.0.children().filter_map(Dot::new).next()
    }
    pub fn cdr(&self) -> Option<Value> {
        self.0.children().filter_map(Value::new).next()
    }
    pub fn rparen_token(&self) -> Option<RParen> {
        self.0.children().filter_map(RParen::new).next()
    }
    pub fn build<T0, T1, T2, T3, T4>() -> ConsBuilder<T0, T1, T2, T3, T4>
    where
        T0: AstBuilder<T = LParen>,
        T1: AstBuilder<T = Value>,
        T2: AstBuilder<T = Dot>,
        T3: AstBuilder<T = Value>,
        T4: AstBuilder<T = RParen>,
    {
        Default::default()
    }
}
pub struct ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = RParen>,
{
    lparen: Option<T0>,
    car: Option<T1>,
    dot: Option<T2>,
    cdr: Option<T3>,
    rparen: Option<T4>,
}
impl<T0, T1, T2, T3, T4> Default for ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = RParen>,
{
    fn default() -> Self {
        Self {
            lparen: Default::default(),
            car: Default::default(),
            dot: Default::default(),
            cdr: Default::default(),
            rparen: Default::default(),
        }
    }
}
impl<T0, T1, T2, T3, T4> ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = RParen>,
{
    pub fn fill(self, lparen: T0, car: T1, dot: T2, cdr: T3, rparen: T4) -> Self {
        Self {
            lparen: Some(lparen),
            car: Some(car),
            dot: Some(dot),
            cdr: Some(cdr),
            rparen: Some(rparen),
        }
    }
}
impl<T0, T1, T2, T3, T4> AstBuilder for ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = RParen>,
{
    type T = Cons;
    fn build(self, builder: &mut GreenBuilder) -> Cons {
        let green = AstBuilder::build_green(self, builder);
        Cons::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut GreenBuilder) -> Green {
        let children = None
            .into_iter()
            .chain(self.lparen.map(|it| it.build_green(builder)).into_iter())
            .chain(self.car.map(|it| it.build_green(builder)).into_iter())
            .chain(self.dot.map(|it| it.build_green(builder)).into_iter())
            .chain(self.cdr.map(|it| it.build_green(builder)).into_iter())
            .chain(self.rparen.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("Cons", move |_| children)
    }
}
impl<T0, T1, T2, T3, T4> IntoBuilder<Value> for ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = LParen>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = RParen>,
{
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}
