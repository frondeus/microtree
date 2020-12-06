use microtree::Cache;

use crate::{Context, Error, Lexer, ParseResult, Parser, TokenKind};

pub struct State<Tok: TokenKind> {
    lexer: Lexer<Tok>,
    cache: Cache,
    errors: Vec<Error>,
}

impl<Tok> State<Tok>
where
    Tok: TokenKind,
{
    fn new(lexer: Lexer<Tok>) -> Self {
        Self {
            lexer,
            cache: Default::default(),
            errors: Default::default(),
        }
    }

    pub fn lexer_mut(&mut self) -> &mut Lexer<Tok> {
        &mut self.lexer
    }

    pub fn parse(lexer: Lexer<Tok>, parser: impl Parser<Tok>) -> ParseResult {
        let ctx = Context::default();
        let (root, state) = parser.parse(Self::new(lexer), &ctx);

        ParseResult {
            root,
            errors: state.errors,
        }
    }

    pub fn transform<Tok2>(self) -> State<Tok2>
    where
        Tok2: TokenKind,
        Tok::Extra: Into<Tok2::Extra>,
    {
        let Self {
            lexer,
            cache,
            errors,
        } = self;
        State {
            errors,
            cache,
            lexer: lexer.transform(),
        }
    }

    pub(crate) fn add_error(&mut self, err: Error) {
        self.errors.push(err);
    }

    pub(crate) fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }

    pub(crate) fn builder<'a>(self, ctx: &'a Context<'a, Tok>) -> crate::Builder<'a, Tok> {
        crate::Builder::new(self, ctx)
    }
}
