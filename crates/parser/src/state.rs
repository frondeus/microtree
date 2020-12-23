use microtree::{Cache, Green};

use crate::{Context, Error, Lexer, ParseResult, Parser, TokenKind};

pub struct State<'source, Tok: TokenKind<'source>> {
    lexer: Lexer<'source, Tok>,
    cache: Cache,
    errors: Vec<Error>,
}

impl<'source, Tok> State<'source, Tok>
where
    Tok: TokenKind<'source>,
{
    fn new(lexer: Lexer<'source, Tok>) -> Self {
        Self {
            lexer,
            cache: Default::default(),
            errors: Default::default(),
        }
    }

    pub fn lexer_mut(&mut self) -> &mut Lexer<'source, Tok> {
        &mut self.lexer
    }

    pub fn parse(lexer: Lexer<'source, Tok>, parser: impl Parser<'source, Tok>) -> ParseResult {
        let ctx = Context::default();
        let (root, state) = parser.parse(Self::new(lexer), &ctx);

        ParseResult {
            root,
            errors: state.errors,
        }
    }

    pub fn morph<Tok2>(self) -> State<'source, Tok2>
    where
        Tok2: TokenKind<'source>,
        Tok::Extras: Into<Tok2::Extras>,
    {
        let Self {
            lexer,
            cache,
            errors,
        } = self;
        State {
            errors,
            cache,
            lexer: lexer.morph(),
        }
    }

    pub(crate) fn add_error(&mut self, err: Error) {
        self.errors.push(err);
    }

    pub(crate) fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }

    pub(crate) fn builder<'ctx>(self, ctx: &'ctx Context<'source, 'ctx, Tok>) -> crate::Builder<'source, 'ctx, Tok> {
        crate::Builder::new(self, ctx)
    }
}
