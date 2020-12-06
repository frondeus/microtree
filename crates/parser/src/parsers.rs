use crate::{Builder, Parser, TokenKind};

pub fn any_token<Tok: TokenKind>() -> impl Parser<Tok> {
    |builder: Builder<Tok>| builder.name("token").token()
}

pub fn error<Tok: TokenKind>(desc: impl ToString + Clone) -> impl Parser<Tok> {
    move |builder: Builder<Tok>| builder.error(desc.clone())
}

pub fn tokens<Tok: TokenKind>(expected: &[Tok]) -> impl Parser<Tok> + '_ {
    let expect_eof = expected.is_empty();
    move |mut builder: Builder<Tok>| match (builder.peek_token(), expect_eof) {
        (Some(tok), true) => builder.error(format!("Expected EOF, found {}", tok)),
        (None, false) => builder.error(format!("{} but found EOF`", Expected(expected))),
        (Some(tok), false) if !expected.contains(&tok) => {
            builder.error(format!("{} but found {}", Expected(expected), tok))
        }
        _ => builder.name("token").token(),
    }
}

pub fn token<Tok: TokenKind>(expected: impl Into<Option<Tok>>) -> impl Parser<Tok> {
    let expected = expected.into();
    move |mut builder: Builder<Tok>| match (builder.peek_token(), expected) {
        (Some(tok), None) => builder.error(format!("Expected EOF, found {}", tok)),
        (None, Some(expected)) => builder.error(format!("Expected {} but found EOF`", expected)),
        (Some(tok), Some(expected)) if tok != expected => {
            builder.error(format!("Expected {} but found {}", expected, tok))
        }
        _ => builder.name("token").token(),
    }
}

struct Expected<'a, Tok: TokenKind>(&'a [Tok]);

impl<'a, Tok: TokenKind> std::fmt::Display for Expected<'a, Tok> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected ")?;
        let last = self.0.len() - 1;
        if last > 0 {
            write!(f, "one of ")?;
        }
        let iter = self.0.iter();
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
