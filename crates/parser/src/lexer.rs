use crate::{Input, PeekableIterator, SmolStr, TextRange, TextSize, Token};

pub trait TokenKind:
    Clone + Copy + std::fmt::Debug + std::fmt::Display + PartialEq + Send + Sync
{
    type Extra: Default + Clone;
    fn is_mergeable(self, other: Self) -> bool;
    fn lex(lexer: &mut Lexer<Self>) -> Option<(Self, SmolStr)>;
}

#[derive(Debug)]
pub struct Lexer<Tok: TokenKind> {
    input: Input,
    #[allow(clippy::option_option)]
    peeked: Option<Option<Token<Tok>>>,
    pub extra: Tok::Extra,
}

impl<Tok: TokenKind> Lexer<Tok> {
    pub fn new(i: &str) -> Self {
        Self {
            input: Input::from(i),
            peeked: None,
            extra: Default::default(),
        }
    }

    pub fn transform<Tok2>(self) -> Lexer<Tok2>
    where
        Tok2: TokenKind,
        Tok::Extra: Into<Tok2::Extra>,
    {
        let Self { input, extra, .. } = self;
        Lexer {
            input,
            peeked: None,
            extra: extra.into(),
        }
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    fn lex(&mut self) -> Option<Token<Tok>> {
        if let Some(peeked) = self.peeked.take() {
            if let Some(peeked) = peeked.as_ref() {
                self.input.set_cursor(peeked.end());
            }
            return peeked;
        }

        let offset = self.input.cursor();
        let (kind, value) = Tok::lex(self)?;
        Some(Token {
            kind,
            value,
            offset,
        })
    }

    fn peek_one(&mut self) -> Option<&Token<Tok>> {
        if self.peeked.is_none() {
            let i = self.input.cursor();
            self.peeked = Some(self.lex());
            self.input.set_cursor(i);
        }

        self.peeked.as_ref().and_then(|i| i.as_ref())
    }
}

impl<Tok> PeekableIterator for Lexer<Tok>
where
    Tok: TokenKind,
{
    fn peek(&mut self) -> Option<&Self::Item> {
        if self.peeked.is_none() {
            let i = self.input.cursor();
            self.peeked = Some(self.next());
            self.input.set_cursor(i);
        }

        self.peeked.as_ref().and_then(|i| i.as_ref())
    }
}

impl<Tok> Iterator for Lexer<Tok>
where
    Tok: TokenKind,
{
    type Item = Token<Tok>;

    fn next(&mut self) -> Option<Token<Tok>> {
        let mut first = self.lex()?;

        loop {
            match self.peek_one() {
                Some(token) if first.kind.is_mergeable(token.kind) => {
                    let to: TextSize = ((first.value.len() + token.value.len()) as u32).into();
                    let to = first.offset + to;
                    first.value = self.input.str_for_range(TextRange::new(first.offset, to));
                    self.lex();
                }
                _ => break,
            }
        }
        Some(first)
    }
}

pub trait OptionExt<T> {
    fn as_kind(&self) -> Option<T>;
}

impl<T: Copy> OptionExt<T> for Option<Token<T>> {
    fn as_kind(&self) -> Option<T> {
        self.as_ref().map(|t| t.kind)
    }
}

impl<T: Copy> OptionExt<T> for Option<&Token<T>> {
    fn as_kind(&self) -> Option<T> {
        self.as_ref().map(|t| t.kind)
    }
}
