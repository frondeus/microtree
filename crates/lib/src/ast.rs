use crate::{Cache, Green, Name, Red};
use smol_str::SmolStr;
use std::marker::PhantomData;

pub trait Ast: Sized {
    fn new(node: Red) -> Option<Self>;
    fn red(&self) -> Red;
}

pub trait AstBuilder {
    type T;
    fn build(self, builder: &mut Cache) -> Self::T;
    fn build_green(self, builder: &mut Cache) -> Green;
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green;
}

pub struct TokenBuilder<A> {
    leading: Option<SmolStr>,
    name: Name,
    token: SmolStr,
    trailing: Option<SmolStr>,
    _phantom: PhantomData<A>,
}
impl<A> TokenBuilder<A> {
    pub fn custom(name: Name, token: impl Into<SmolStr>) -> Self {
        Self {
            name,
            token: token.into(),
            leading: Default::default(),
            trailing: Default::default(),
            _phantom: Default::default(),
        }
    }
    pub fn new(token: impl Into<SmolStr>) -> Self {
        Self::custom("token", token)
    }
    pub fn with_leading(mut self, leading: impl Into<SmolStr>) -> Self {
        self.leading = Some(leading.into());
        self
    }
    pub fn with_trailing(mut self, trailing: impl Into<SmolStr>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    pub fn build_token(self, builder: &mut Cache) -> Green {
        let leading = self.leading.unwrap_or_default();
        let trailing = self.trailing.unwrap_or_default();
        builder.with_trivia(self.name, leading, self.token, trailing)
    }
}

impl<T> AstBuilder for TokenBuilder<T>
where
    T: Ast,
{
    type T = T;
    fn build_green(self, builder: &mut Cache) -> Green {
        self.build_token(builder)
    }

    fn build(self, builder: &mut Cache) -> T {
        T::new(Red::root(self.build_token(builder))).unwrap()
    }

    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        self.build_token(builder)
    }
}

pub struct AliasBuilder<B, As>
where
    B: AstBuilder,
    As: Ast,
{
    alias: Name,
    builder: B,
    _phantom: PhantomData<As>,
}

impl<B, As> AliasBuilder<B, As>
where
    B: AstBuilder,
    As: Ast,
{
    pub fn new(alias: Name, builder: B) -> Self {
        Self {
            alias,
            builder,
            _phantom: Default::default(),
        }
    }
}

impl<B, As> AstBuilder for AliasBuilder<B, As>
where
    B: AstBuilder,
    As: Ast,
{
    type T = As;
    fn build(self, builder: &mut Cache) -> As {
        let green = AstBuilder::build_green(self, builder);
        As::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut Cache) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut Cache) -> Green {
        let green = AstBuilder::build_green(self.builder, builder);
        builder.alias(self.alias, move |_| green)
    }
}

pub trait IntoBuilder<As: Ast>: AstBuilder + Sized {
    fn into_builder(self) -> AliasBuilder<Self, As>;
    fn into_dyn(self) -> Box<AliasBuilder<Self, As>> {
        Box::new(self.into_builder())
    }
}
