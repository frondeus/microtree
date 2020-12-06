use crate::{
    Context, Error, OptionExt, Parser, PeekableIterator, SmolStr, State, TextRange, TokenKind,
};
use microtree::{Green, Name};
use std::collections::BTreeSet;

impl<Fun, Tok> Parser<Tok> for Fun
where
    Tok: TokenKind,
    Fun: Fn(Builder<Tok>) -> (Option<Green>, State<Tok>),
{
    fn parse(&self, state: State<Tok>, ctx: &Context<Tok>) -> (Option<Green>, State<Tok>) {
        self(state.builder(ctx))
    }
}

pub struct Builder<'a, Tok: TokenKind> {
    pub(crate) state: State<Tok>,
    pub(crate) ctx: &'a Context<'a, Tok>,
    pub(crate) names: BTreeSet<Name>,
}

impl<'a, Tok: TokenKind> Builder<'a, Tok> {
    pub(crate) fn new(state: State<Tok>, ctx: &'a Context<'a, Tok>) -> Self {
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
        self.state.lexer_mut().peek().as_kind()
    }
    pub fn node(self) -> NodeBuilder<'a, Tok> {
        NodeBuilder::new(self)
    }
    pub fn set_ctx(mut self, ctx: &'a Context<'a, Tok>) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn get_ctx(&self) -> &'a Context<'a, Tok> {
        self.ctx
    }

    pub fn none(self) -> (Option<Green>, State<Tok>) {
        (None, self.state)
    }

    pub fn parse(self, parser: impl Parser<Tok>) -> (Option<Green>, State<Tok>) {
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
        trivia: Option<&'a dyn Parser<Tok>>,
        state: State<Tok>,
    ) -> (SmolStr, State<Tok>) {
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

    pub fn error(self, desc: impl ToString) -> (Option<Green>, State<Tok>) {
        let Self {
            mut state, names, ..
        } = self;
        let from = state.lexer_mut().input().cursor();

        let token = state.lexer_mut().next();

        let value = token.map(|t| t.value).unwrap_or_default();

        let range = TextRange::at(from, (value.len() as u32).into());

        let error = Error::new(desc, range);

        state.add_error(error);

        let mut node = state.cache().token("error", value);
        for alias in names {
            node = state.cache().alias(alias, |_| node);
        }

        (Some(node), state)
    }

    pub fn token(self) -> (Option<Green>, State<Tok>) {
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

pub struct NodeBuilder<'a, Tok: TokenKind> {
    state: State<Tok>,
    ctx: &'a Context<'a, Tok>,
    names: BTreeSet<Name>,
    children: Vec<Green>,
}

impl<'a, Tok: TokenKind> NodeBuilder<'a, Tok> {
    pub(crate) fn new(Builder { state, names, ctx }: Builder<'a, Tok>) -> Self {
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
        self.state.lexer_mut().peek().as_kind()
    }

    pub fn set_ctx(mut self, ctx: &'a Context<'a, Tok>) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn parse(mut self, parser: impl Parser<Tok>) -> Self {
        let (res, state) = parser.parse(self.state, self.ctx);
        self.state = state;
        if let Some(res) = res {
            self.children.push(res);
        }

        self
    }

    pub fn parse_mode<Tok2>(self, parser: impl Parser<Tok2>) -> Self
    where
        Tok2: TokenKind,
        Tok::Extra: Into<Tok2::Extra>,
        Tok2::Extra: Into<Tok::Extra>,
    {
        let Self {
            state,
            ctx,
            names,
            mut children,
        } = self;

        let state: State<Tok2> = state.transform();

        let inner_ctx = Context::default();

        let (res, state) = parser.parse(state, &inner_ctx);

        let state: State<Tok> = state.transform();

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

    pub fn finish(self) -> (Option<Green>, State<Tok>) {
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
