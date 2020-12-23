use crate::{Context, Error, Parser, PeekableIterator, SmolStr, State, TextRange, TokenKind};
use microtree::{Green, Name};
use std::collections::BTreeSet;

impl<'source, Fun, Tok> Parser<'source, Tok> for Fun
where
    Tok: TokenKind<'source>,
    Fun: Fn(Builder<'source, '_, Tok>) -> (Option<Green>, State<'source, Tok>),
{
    fn parse(&self, state: State<'source, Tok>, ctx: &Context<'source, '_, Tok>) -> (Option<Green>, State<'source, Tok>) {
        self(state.builder(ctx))
    }
}

pub struct Builder<'source, 'ctx, Tok: TokenKind<'source>> {
    pub(crate) state: State<'source, Tok>, pub(crate) ctx: &'ctx
    Context<'source, 'ctx, Tok>,
    pub(crate) names: BTreeSet<Name>,
}

impl<'source, 'ctx, Tok: TokenKind<'source>> Builder<'source, 'ctx, Tok> {
    pub(crate) fn new(state: State<'source, Tok>, ctx: &'ctx Context<'source, 'ctx, Tok>) -> Self {
        Self {
            state,
            ctx,
            names: Default::default(),
        }
    }
    pub fn name(mut self, name: Name) -> Self {
        self.names.insert(name);
        self
    }
    pub fn peek_token(&mut self) -> Option<Tok> {
        self.state.lexer_mut().peek().map(|t| t.token)
    }
    pub fn node(self) -> NodeBuilder<'source, 'ctx, Tok> {
        NodeBuilder::new(self)
    }
    pub fn set_ctx(mut self, ctx: &'ctx Context<'source, 'ctx, Tok>) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn get_ctx(&self) -> &'ctx Context<'source, 'ctx, Tok> {
        self.ctx
    }

    pub fn none(self) -> (Option<Green>, State<'source, Tok>) {
        (None, self.state)
    }

    pub fn parse(self, parser: impl Parser<'source, Tok>) -> (Option<Green>, State<'source, Tok>) {
        let Self { state, names, ctx } = self;
        let (green, mut state) = parser.parse(state, ctx);

        let mut aliases = names.into_iter();

        let mut node = green;

        let node = loop {
            let alias = aliases.next();
            node = match (node, alias) {
                (None, _) => break None,
                (node, None) => break node,
                (Some(n), Some(alias)) => Some(state.cache().alias(alias, move |_| n)),
            }
        };

        (node, state)
    }

    pub fn handle_trivia(
        trivia: Option<&'ctx dyn Parser<'source, Tok>>,
        state: State<'source, Tok>,
    ) -> (SmolStr, State<'source, Tok>) {
        match trivia {
            None => (Default::default(), state),
            Some(trivia) => {
                let (trivia, state) = {
                    let trivia_ctx = Context::default();
                    trivia.parse(state, &trivia_ctx)
                };
                (
                    trivia.map(|t| t.to_string().into()).unwrap_or_default(),
                    state,
                )
            }
        }
    }

    pub fn error(self, desc: impl ToString) -> (Option<Green>, State<'source, Tok>) {
        let Self {
            mut state, names, ..
        } = self;

        let token = state.lexer_mut().next();
        let (range, value) = match token {
            Some(token) => {
                (token.range,
                token.value)
            },
            None => {
                let range = state.lexer_mut().span();
                (range, Default::default())
            }
        };

        let error = Error::new(desc, range);

        state.add_error(error);

        let mut node = state.cache().token("error", value);
        for alias in names {
            node = state.cache().alias(alias, |_| node);
        }

        (Some(node), state)
    }

    pub fn token(self) -> (Option<Green>, State<'source, Tok>) {
        let Self { state, names, ctx } = self;
        let mut names = names.into_iter();

        let (leading, mut state) = Self::handle_trivia(ctx.leading_trivia, state);

        let value = state.lexer_mut().next().map(|t| t.value);

        let (trailing, mut state) = Self::handle_trivia(ctx.trailing_trivia, state);

        let mut node = match value {
            None => state.cache().with_trivia("eof", leading, "", trailing),
            Some(value) => {
                let name = names.next().unwrap_or_default();

                state.cache().with_trivia(name, leading, value, trailing)
            }
        };

        let aliases = names;
        for alias in aliases {
            node = state.cache().alias(alias, |_| node);
        }

        (Some(node), state)
    }
}

pub struct NodeBuilder<'source, 'ctx, Tok: TokenKind<'source>> {
    state: State<'source, Tok>,
    ctx: &'ctx Context<'source, 'ctx, Tok>,
    names: BTreeSet<Name>,
    children: Vec<Green>,
}

impl<'source, 'ctx, Tok: TokenKind<'source>> NodeBuilder<'source, 'ctx, Tok> {
    pub(crate) fn new(Builder { state, names, ctx }: Builder<'source, 'ctx, Tok>) -> Self {
        Self {
            state,
            names,
            ctx,
            children: Default::default(),
        }
    }

    pub fn name(mut self, name: Name) -> Self {
        self.names.insert(name);
        self
    }

    pub fn peek_token(&mut self) -> Option<Tok> {
        self.state.lexer_mut().peek().map(|t| t.token)
    }

    pub fn set_ctx(mut self, ctx: &'ctx Context<'source, 'ctx, Tok>) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn parse(mut self, parser: impl Parser<'source, Tok>) -> Self {
        let (res, state) = parser.parse(self.state, self.ctx);
        self.state = state;
        if let Some(res) = res {
            self.children.push(res);
        }

        self
    }

    pub fn parse_mode<Tok2>(self, parser: impl Parser<'source, Tok2>) -> Self
    where
        Tok2: TokenKind<'source>,
        Tok::Extras: Into<Tok2::Extras>,
        Tok2::Extras: Into<Tok::Extras>,
    {
        let Self {
            state,
            ctx,
            names,
            mut children,
        } = self;

        let state: State<Tok2> = state.morph();

        let inner_ctx = Context::default();

        let (res, state) = parser.parse(state, &inner_ctx);

        let state: State<Tok> = state.morph();

        if let Some(res) = res {
            children.push(res);
        }

        Self {
            state,
            ctx,
            names,
            children,
        }
    }

    pub fn finish(self) -> (Option<Green>, State<'source, Tok>) {
        let Self {
            mut state,
            names,
            children,
            ..
        } = self;

        let mut names = names.into_iter();
        let name = names.next().unwrap_or_default();
        let aliases = names;

        let mut node = state.cache().node(name, move |_| children);
        for alias in aliases {
            node = state.cache().alias(alias, |_| node);
        }

        (Some(node), state)
    }
}
