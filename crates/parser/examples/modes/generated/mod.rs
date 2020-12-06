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
pub struct OpenI(Red);
impl Ast for OpenI {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "${" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl OpenI {
    pub fn build() -> TokenBuilder<OpenI> {
        TokenBuilder::new("${")
    }
}

#[derive(Debug)]
pub struct OpenP(Red);
impl Ast for OpenP {
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
impl OpenP {
    pub fn build() -> TokenBuilder<OpenP> {
        TokenBuilder::new("(")
    }
}

#[derive(Debug)]
pub struct CloseP(Red);
impl Ast for CloseP {
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
impl CloseP {
    pub fn build() -> TokenBuilder<CloseP> {
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
pub struct CloseI(Red);
impl Ast for CloseI {
    fn new(node: Red) -> Option<Self> {
        if !node.is("token") {
            return None;
        }
        let green = node.green();
        let tok = green.as_token()?;
        if tok.value != "}" {
            return None;
        }
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl CloseI {
    pub fn build() -> TokenBuilder<CloseI> {
        TokenBuilder::new("}")
    }
}

#[derive(Debug)]
pub enum Value {
    Nil(Nil),
    Atom(Atom),
    List(List),
    Cons(Cons),
    String(String),
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
impl From<String> for Value {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}
impl Ast for Value {
    fn new(node: Red) -> Option<Self> {
        None.or_else(|| Nil::new(node.clone()).map(Value::Nil))
            .or_else(|| Atom::new(node.clone()).map(Value::Atom))
            .or_else(|| List::new(node.clone()).map(Value::List))
            .or_else(|| Cons::new(node.clone()).map(Value::Cons))
            .or_else(|| String::new(node.clone()).map(Value::String))
    }
    fn red(&self) -> Red {
        match &self {
            Value::Nil(node) => node.red(),
            Value::Atom(node) => node.red(),
            Value::List(node) => node.red(),
            Value::Cons(node) => node.red(),
            Value::String(node) => node.red(),
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
    pub fn as_string(self) -> Option<String> {
        match self {
            Self::String(val) => Some(val),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StrValue {
    Text(Text),
    Interpolated(Interpolated),
}
impl From<Text> for StrValue {
    fn from(val: Text) -> Self {
        Self::Text(val)
    }
}
impl From<Interpolated> for StrValue {
    fn from(val: Interpolated) -> Self {
        Self::Interpolated(val)
    }
}
impl Ast for StrValue {
    fn new(node: Red) -> Option<Self> {
        None.or_else(|| Text::new(node.clone()).map(StrValue::Text))
            .or_else(|| Interpolated::new(node.clone()).map(StrValue::Interpolated))
    }
    fn red(&self) -> Red {
        match &self {
            StrValue::Text(node) => node.red(),
            StrValue::Interpolated(node) => node.red(),
        }
    }
}
impl StrValue {
    pub fn as_text(self) -> Option<Text> {
        match self {
            Self::Text(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_interpolated(self) -> Option<Interpolated> {
        match self {
            Self::Interpolated(val) => Some(val),
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
    pub fn open_p_token(&self) -> Option<OpenP> {
        self.0.children().filter_map(OpenP::new).next()
    }
    pub fn close_p_token(&self) -> Option<CloseP> {
        self.0.children().filter_map(CloseP::new).next()
    }
    pub fn build<T0, T1>() -> NilBuilder<T0, T1>
    where
        T0: AstBuilder<T = OpenP>,
        T1: AstBuilder<T = CloseP>,
    {
        Default::default()
    }
}
pub struct NilBuilder<T0, T1>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = CloseP>,
{
    open_p: Option<T0>,
    close_p: Option<T1>,
}
impl<T0, T1> Default for NilBuilder<T0, T1>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = CloseP>,
{
    fn default() -> Self {
        Self {
            open_p: Default::default(),
            close_p: Default::default(),
        }
    }
}
impl<T0, T1> NilBuilder<T0, T1>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = CloseP>,
{
    pub fn fill(self, open_p: T0, close_p: T1) -> Self {
        Self {
            open_p: Some(open_p),
            close_p: Some(close_p),
        }
    }
}
impl<T0, T1> AstBuilder for NilBuilder<T0, T1>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = CloseP>,
{
    type T = Nil;
    fn build(self, builder: &mut Cache) -> Nil {
        let green = AstBuilder::build_green(self, builder);
        Nil::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let children = None
            .into_iter()
            .chain(self.open_p.map(|it| it.build_green(builder)).into_iter())
            .chain(self.close_p.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("Nil", move |_| children)
    }
}
impl<T0, T1> IntoBuilder<Value> for NilBuilder<T0, T1>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = CloseP>,
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
    pub fn open_p_token(&self) -> Option<OpenP> {
        self.0.children().filter_map(OpenP::new).next()
    }
    pub fn values(&self) -> impl Iterator<Item = Value> + '_ {
        self.0.children().filter_map(Value::new)
    }
    pub fn close_p_token(&self) -> Option<CloseP> {
        self.0.children().filter_map(CloseP::new).next()
    }
    pub fn build<T0, T2>() -> ListBuilder<T0, T2>
    where
        T0: AstBuilder<T = OpenP>,
        T2: AstBuilder<T = CloseP>,
    {
        Default::default()
    }
}
pub struct ListBuilder<T0, T2>
where
    T0: AstBuilder<T = OpenP>,
    T2: AstBuilder<T = CloseP>,
{
    open_p: Option<T0>,
    values: Vec<Box<dyn AstBuilder<T = Value>>>,
    close_p: Option<T2>,
}
impl<T0, T2> Default for ListBuilder<T0, T2>
where
    T0: AstBuilder<T = OpenP>,
    T2: AstBuilder<T = CloseP>,
{
    fn default() -> Self {
        Self {
            open_p: Default::default(),
            values: Default::default(),
            close_p: Default::default(),
        }
    }
}
impl<T0, T2> ListBuilder<T0, T2>
where
    T0: AstBuilder<T = OpenP>,
    T2: AstBuilder<T = CloseP>,
{
    pub fn fill(
        self,
        open_p: T0,
        values: Vec<Box<dyn AstBuilder<T = Value>>>,
        close_p: T2,
    ) -> Self {
        Self {
            open_p: Some(open_p),
            values,
            close_p: Some(close_p),
        }
    }
}
impl<T0, T2> AstBuilder for ListBuilder<T0, T2>
where
    T0: AstBuilder<T = OpenP>,
    T2: AstBuilder<T = CloseP>,
{
    type T = List;
    fn build(self, builder: &mut Cache) -> List {
        let green = AstBuilder::build_green(self, builder);
        List::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let children = None
            .into_iter()
            .chain(self.open_p.map(|it| it.build_green(builder)).into_iter())
            .chain({
                self.values
                    .into_iter()
                    .map(|it| it.build_boxed_green(builder))
                    .collect::<Vec<_>>()
            })
            .chain(self.close_p.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("List", move |_| children)
    }
}
impl<T0, T2> IntoBuilder<Value> for ListBuilder<T0, T2>
where
    T0: AstBuilder<T = OpenP>,
    T2: AstBuilder<T = CloseP>,
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
    pub fn open_p_token(&self) -> Option<OpenP> {
        self.0.children().filter_map(OpenP::new).next()
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
    pub fn close_p_token(&self) -> Option<CloseP> {
        self.0.children().filter_map(CloseP::new).next()
    }
    pub fn build<T0, T1, T2, T3, T4>() -> ConsBuilder<T0, T1, T2, T3, T4>
    where
        T0: AstBuilder<T = OpenP>,
        T1: AstBuilder<T = Value>,
        T2: AstBuilder<T = Dot>,
        T3: AstBuilder<T = Value>,
        T4: AstBuilder<T = CloseP>,
    {
        Default::default()
    }
}
pub struct ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = CloseP>,
{
    open_p: Option<T0>,
    car: Option<T1>,
    dot: Option<T2>,
    cdr: Option<T3>,
    close_p: Option<T4>,
}
impl<T0, T1, T2, T3, T4> Default for ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = CloseP>,
{
    fn default() -> Self {
        Self {
            open_p: Default::default(),
            car: Default::default(),
            dot: Default::default(),
            cdr: Default::default(),
            close_p: Default::default(),
        }
    }
}
impl<T0, T1, T2, T3, T4> ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = CloseP>,
{
    pub fn fill(self, open_p: T0, car: T1, dot: T2, cdr: T3, close_p: T4) -> Self {
        Self {
            open_p: Some(open_p),
            car: Some(car),
            dot: Some(dot),
            cdr: Some(cdr),
            close_p: Some(close_p),
        }
    }
}
impl<T0, T1, T2, T3, T4> AstBuilder for ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = CloseP>,
{
    type T = Cons;
    fn build(self, builder: &mut Cache) -> Cons {
        let green = AstBuilder::build_green(self, builder);
        Cons::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let children = None
            .into_iter()
            .chain(self.open_p.map(|it| it.build_green(builder)).into_iter())
            .chain(self.car.map(|it| it.build_green(builder)).into_iter())
            .chain(self.dot.map(|it| it.build_green(builder)).into_iter())
            .chain(self.cdr.map(|it| it.build_green(builder)).into_iter())
            .chain(self.close_p.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("Cons", move |_| children)
    }
}
impl<T0, T1, T2, T3, T4> IntoBuilder<Value> for ConsBuilder<T0, T1, T2, T3, T4>
where
    T0: AstBuilder<T = OpenP>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = Dot>,
    T3: AstBuilder<T = Value>,
    T4: AstBuilder<T = CloseP>,
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
    pub fn l_dquote_token(&self) -> Option<DQuote> {
        self.0.children().filter_map(DQuote::new).next()
    }
    pub fn values(&self) -> impl Iterator<Item = StrValue> + '_ {
        self.0.children().filter_map(StrValue::new)
    }
    pub fn r_dquote_token(&self) -> Option<DQuote> {
        self.0.children().filter_map(DQuote::new).next()
    }
    pub fn build<T0, T2>() -> StringBuilder<T0, T2>
    where
        T0: AstBuilder<T = DQuote>,
        T2: AstBuilder<T = DQuote>,
    {
        Default::default()
    }
}
pub struct StringBuilder<T0, T2>
where
    T0: AstBuilder<T = DQuote>,
    T2: AstBuilder<T = DQuote>,
{
    l_dquote: Option<T0>,
    values: Vec<Box<dyn AstBuilder<T = StrValue>>>,
    r_dquote: Option<T2>,
}
impl<T0, T2> Default for StringBuilder<T0, T2>
where
    T0: AstBuilder<T = DQuote>,
    T2: AstBuilder<T = DQuote>,
{
    fn default() -> Self {
        Self {
            l_dquote: Default::default(),
            values: Default::default(),
            r_dquote: Default::default(),
        }
    }
}
impl<T0, T2> StringBuilder<T0, T2>
where
    T0: AstBuilder<T = DQuote>,
    T2: AstBuilder<T = DQuote>,
{
    pub fn fill(
        self,
        l_dquote: T0,
        values: Vec<Box<dyn AstBuilder<T = StrValue>>>,
        r_dquote: T2,
    ) -> Self {
        Self {
            l_dquote: Some(l_dquote),
            values,
            r_dquote: Some(r_dquote),
        }
    }
}
impl<T0, T2> AstBuilder for StringBuilder<T0, T2>
where
    T0: AstBuilder<T = DQuote>,
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
            .chain(self.l_dquote.map(|it| it.build_green(builder)).into_iter())
            .chain({
                self.values
                    .into_iter()
                    .map(|it| it.build_boxed_green(builder))
                    .collect::<Vec<_>>()
            })
            .chain(self.r_dquote.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("String", move |_| children)
    }
}
impl<T0, T2> IntoBuilder<Value> for StringBuilder<T0, T2>
where
    T0: AstBuilder<T = DQuote>,
    T2: AstBuilder<T = DQuote>,
{
    fn into_builder(self) -> AliasBuilder<Self, Value> {
        AliasBuilder::new("Value", self)
    }
}

#[derive(Debug)]
pub struct Interpolated(Red);
impl Ast for Interpolated {
    fn new(node: Red) -> Option<Self> {
        if !node.is("Interpolated") {
            return None;
        }
        node.green().as_node()?;
        Some(Self(node))
    }
    fn red(&self) -> Red {
        self.0.clone()
    }
}
impl Interpolated {
    pub fn open_i_token(&self) -> Option<OpenI> {
        self.0.children().filter_map(OpenI::new).next()
    }
    pub fn value(&self) -> Option<Value> {
        self.0.children().filter_map(Value::new).next()
    }
    pub fn close_i_token(&self) -> Option<CloseI> {
        self.0.children().filter_map(CloseI::new).next()
    }
    pub fn build<T0, T1, T2>() -> InterpolatedBuilder<T0, T1, T2>
    where
        T0: AstBuilder<T = OpenI>,
        T1: AstBuilder<T = Value>,
        T2: AstBuilder<T = CloseI>,
    {
        Default::default()
    }
}
pub struct InterpolatedBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = OpenI>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = CloseI>,
{
    open_i: Option<T0>,
    value: Option<T1>,
    close_i: Option<T2>,
}
impl<T0, T1, T2> Default for InterpolatedBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = OpenI>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = CloseI>,
{
    fn default() -> Self {
        Self {
            open_i: Default::default(),
            value: Default::default(),
            close_i: Default::default(),
        }
    }
}
impl<T0, T1, T2> InterpolatedBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = OpenI>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = CloseI>,
{
    pub fn fill(self, open_i: T0, value: T1, close_i: T2) -> Self {
        Self {
            open_i: Some(open_i),
            value: Some(value),
            close_i: Some(close_i),
        }
    }
}
impl<T0, T1, T2> AstBuilder for InterpolatedBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = OpenI>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = CloseI>,
{
    type T = Interpolated;
    fn build(self, builder: &mut Cache) -> Interpolated {
        let green = AstBuilder::build_green(self, builder);
        Interpolated::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let children = None
            .into_iter()
            .chain(self.open_i.map(|it| it.build_green(builder)).into_iter())
            .chain(self.value.map(|it| it.build_green(builder)).into_iter())
            .chain(self.close_i.map(|it| it.build_green(builder)).into_iter())
            .collect();
        builder.node("Interpolated", move |_| children)
    }
}
impl<T0, T1, T2> IntoBuilder<StrValue> for InterpolatedBuilder<T0, T1, T2>
where
    T0: AstBuilder<T = OpenI>,
    T1: AstBuilder<T = Value>,
    T2: AstBuilder<T = CloseI>,
{
    fn into_builder(self) -> AliasBuilder<Self, StrValue> {
        AliasBuilder::new("StrValue", self)
    }
}
