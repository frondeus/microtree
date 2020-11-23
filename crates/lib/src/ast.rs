use crate::{Green, GreenBuilder, Name, Red};
use smol_str::SmolStr;
use std::marker::PhantomData;

pub trait Ast: Sized {
    fn new(node: Red) -> Option<Self>;
    fn red(&self) -> Red;
}

pub trait AstBuilder {
    type T;
    fn build(self, builder: &mut GreenBuilder) -> Self::T;
    fn build_green(self, builder: &mut GreenBuilder) -> Green;
    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green;
}

pub struct TokenBuilder<A> {
    pre: Option<SmolStr>,
    name: Name,
    token: SmolStr,
    post: Option<SmolStr>,
    _phantom: PhantomData<A>,
}
impl<A> TokenBuilder<A> {
    pub fn custom(name: Name, token: impl Into<SmolStr>) -> Self {
        Self {
            name,
            token: token.into(),
            pre: Default::default(),
            post: Default::default(),
            _phantom: Default::default(),
        }
    }
    pub fn new(token: impl Into<SmolStr>) -> Self {
        Self::custom("token", token)
    }
    pub fn with_pre(mut self, pre: impl Into<SmolStr>) -> Self {
        self.pre = Some(pre.into());
        self
    }
    pub fn with_post(mut self, post: impl Into<SmolStr>) -> Self {
        self.post = Some(post.into());
        self
    }

    pub fn build_token(self, builder: &mut GreenBuilder) -> Green {
        let pre = self.pre.unwrap_or_default();
        let post = self.post.unwrap_or_default();
        builder.with_trivia(self.name, pre, self.token, post)
    }
}

impl<T> AstBuilder for TokenBuilder<T>
where
    T: Ast,
{
    type T = T;
    fn build_green(self, builder: &mut GreenBuilder) -> Green {
        self.build_token(builder)
    }

    fn build(self, builder: &mut GreenBuilder) -> T {
        T::new(Red::root(self.build_token(builder))).unwrap()
    }

    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green {
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
    fn build(self, builder: &mut GreenBuilder) -> As {
        let green = AstBuilder::build_green(self, builder);
        As::new(Red::root(green)).unwrap()
    }
    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green {
        AstBuilder::build_green(*self, builder)
    }
    fn build_green(self, builder: &mut GreenBuilder) -> Green {
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
