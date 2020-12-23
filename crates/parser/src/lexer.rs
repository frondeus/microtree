use std::ops::{Deref, DerefMut};

use crate::{Input, PeekableIterator, SmolStr, TextRange, TextSize, Token};
pub use logos::Logos;
use logos::Lexer as Inner;


pub trait TokenKind<'source> : Logos<'source, Source = str, Extras: Clone>
    + std::fmt::Display + PartialEq
    + Clone
    + Copy
    + Send
    + Sync
{
    fn mergeable(self, _other: Self) -> bool { false }
}

pub struct Lexer<'source, Tok: TokenKind<'source>> {
    #[allow(clippy::option_option)]
    peeked: Option<Option<(Inner<'source, Tok>, Token<Tok>)>>,
    inner: Inner<'source, Tok>
}


impl<'source, Tok: TokenKind<'source>> Lexer<'source, Tok> {
    pub fn new(source: &'source Tok::Source) -> Self {
        Self {
            inner: Inner::new(source),
            peeked: None
        }
    }

    pub fn morph<Tok2>(self) -> Lexer<'source, Tok2>
    where Tok2: TokenKind<'source>,
        Tok::Extras: Into<Tok2::Extras>,
    {
        Lexer {
            peeked: None,
            inner: self.inner.morph()
        }
    }

    pub fn span(&self) -> TextRange {
        let range = self.inner.span();
        TextRange::new((range.start as u32).into(), (range.end as u32).into())
    }
}

impl<'source, Tok> PeekableIterator for Lexer<'source, Tok>
where Tok: TokenKind<'source>
{
    fn peek(&mut self) -> Option<&Self::Item> {
        if self.peeked.is_none() {
            let saved = self.inner.clone();
            let token = self.next();
            let original = std::mem::replace(&mut self.inner, saved);
            self.peeked = Some(token.map(|token| (original, token)));
        }

        self.peeked.as_ref().and_then(|t| t.as_ref())
            .map(|(_, t)| t)
    }
}

impl<'source, Tok: TokenKind<'source>> Iterator for Lexer<'source, Tok> {
    type Item = Token<Tok>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut first = self.lex()?;

        loop {
            match self.peek_one() {
                Some(token) if first.token.mergeable(token.token) => {
                    let from = first.range.start();
                    let len: TextSize = ((first.value.len() + token.value.len()) as u32).into();
                    let range = TextRange::at(from, len);
                    first.range = range;
                    let new_value = &self.inner.source()[range];
                    first.value = new_value.into();
                    self.lex();
                },
                _ => break Some(first)
            }
        }
    }
}

impl<'source, Tok: TokenKind<'source>> Lexer<'source, Tok> {
    fn lex(&mut self) -> Option<Token<Tok>> {
        if let Some(peeked) = self.peeked.take() {
            if let Some((original, peeked)) = peeked {
                self.inner = original;
                return Some(peeked);
            }
            return None;
        }
        let token = self.inner.next()?;
        let value = self.inner.slice().into();
        let range = self.span();
        Some(Token {
            token, range, value
        })
    }

    fn peek_one(&mut self) -> Option<&Token<Tok>> {
            if self.peeked.is_none() {
                let saved = self.inner.clone();
                let token = self.lex();
                let original = std::mem::replace(&mut self.inner, saved);
                self.peeked = Some(token.map(|token| (original, token)));
            }

            self.peeked.as_ref().and_then(|t| t.as_ref())
                .map(|(_, t)| t)
        }
}
