use std::marker::PhantomData;
use std::fmt::Display;

use crate::{Builder, Parser, TokenKind};

pub fn any_token<'s, Tok: TokenKind<'s>>() -> impl Parser<'s, Tok> {
  |builder: Builder<'s, '_, Tok>| builder.name("token").token()
}

pub fn error<'s, Tok: TokenKind<'s>>(desc: impl ToString + Clone) -> impl Parser<'s, Tok> {
    move |builder: Builder<'s, '_, Tok>| builder.error(desc.clone())
}

pub fn tokens<'s, Tok: TokenKind<'s>>(expected: &'s [Tok]) -> impl Parser<'s, Tok> {
    let expect_eof = expected.is_empty();
    move |mut builder: Builder<'s, '_, Tok>| match (builder.peek_token(), expect_eof) {
        (Some(tok), true) => builder.error(format!("Expected EOF, found {}", tok)),
        (None, false) => builder.error(format!("{} but found EOF`", Expected::new(expected))),
        (Some(tok), false) if !expected.contains(&tok) => {
            builder.error(format!("{} but found {}", Expected::new(expected), tok))
        }
        _ => builder.name("token").token(),
    }
}

pub fn token<'s, Tok: TokenKind<'s>>(expected: impl Into<Option<Tok>>) -> impl Parser<'s, Tok> {
    let expected = expected.into();
    move |mut builder: Builder<'s, '_, Tok>| match (builder.peek_token(), expected) {
        (Some(tok), None) => builder.error(format!("Expected EOF, found {}", tok)),
        (None, Some(expected)) => builder.error(format!("Expected {} but found EOF`", expected)),
        (Some(tok), Some(expected)) if tok != expected => {
            builder.error(format!("Expected {} but found {}", expected, tok))
        }
        _ => builder.name("token").token(),
    }
}

struct Expected<'s, Tok: TokenKind<'s>>{
    expected: &'s [Tok],
}

impl <'s, Tok> Expected<'s, Tok>
    where Tok: TokenKind<'s>
{
    fn new(expected: &'s [Tok]) -> Self {
        Self {
            expected,
        }
    }
}

impl<'s, Tok> Display for Expected<'s, Tok>
    where Tok: TokenKind<'s>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected ")?;
        let last = self.expected.len() - 1;
        if last > 0 {
            write!(f, "one of ")?;
        }
        let iter = self.expected.iter();
        for (i, token) in iter.enumerate() {
            if i == 0 {
                write!(f, "{}", token)?;
            } else {
                write!(f, ", {}", token)?;
            }
        }
        Ok(())
    }
}
