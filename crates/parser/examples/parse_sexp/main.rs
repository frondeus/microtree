use microtree::{Ast, Red};

mod generated;

use generated::*;
use microtree_parser::State;

mod parser {
    use microtree_parser::{parsers::*, Builder, Context, Parser, Logos, TokenKind};

    #[derive(Logos, Debug, PartialEq, Clone, Copy)]
    pub enum Token {
        #[error]
        Error,
        #[token("(")]
        OpenP,
        #[token(")")]
        CloseP,
        #[token(".")]
        Dot,
        #[regex("[0-9a-zA-Z_]+")]
        Atom,
        #[regex(r#"[ \t\n]+"#)]
        Whitespace,
    }

    impl TokenKind<'_> for Token {}

    impl std::fmt::Display for Token {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Token::Error => "error",
                    Token::OpenP => "`(`",
                    Token::CloseP => "`)`",
                    Token::Dot => "`.`",
                    Token::Atom => "atom",
                    Token::Whitespace => "whitespace",
                }
            )
        }
    }

    pub type Lexer<'s, T = Token> = microtree_parser::Lexer<'s, T>;

    pub fn trivia<'s>() -> impl Parser<'s, Token> {
        |mut builder: Builder<'s, '_, Token>| match builder.peek_token() {
            Some(Token::Whitespace) => builder.name("trivia").parse(any_token()),
            _ => builder.none(),
        }
    }

    pub fn sexp<'s>() -> impl Parser<'s, Token> {
        |builder: Builder<'s, '_, Token>| {
            let mut builder = builder.node().parse(any_token()); //'('

            match builder.peek_token() {
                Some(Token::CloseP) => builder.name("Nil").parse(any_token()),
                _ => {
                    let mut builder = builder.parse(value());

                    match builder.peek_token() {
                        Some(Token::Dot) => {
                            builder
                                .name("Cons")
                                .parse(any_token()) //'.'
                                .parse(value())
                        }
                        _ => {
                            let mut builder = builder.name("List");
                            loop {
                                match builder.peek_token() {
                                    None => break builder,
                                    Some(Token::CloseP) => break builder,
                                    _ => builder = builder.parse(value()),
                                }
                            }
                        }
                    }
                    .parse(token(Token::CloseP))
                }
            }
            .finish()
        }
    }

    pub fn value<'s>() -> impl Parser<'s, Token> {
        |builder: Builder<'s, '_, Token>| {
            let trivia = trivia();
            let ctx = Context::new(&trivia);
            let mut builder = builder.name("Value").set_ctx(&ctx);
            match builder.peek_token() {
                Some(Token::OpenP) => builder.parse(sexp()),
                Some(Token::Atom) => builder.name("atom").token(),
                _ => builder.parse(tokens(&[Token::OpenP, Token::Atom])),
            }
        }
    }
}

fn main() {
    fn act(input: &str) -> Option<Value> {
        let lexer = parser::Lexer::new(input);
        let parsed = State::parse(lexer, parser::value());

        dbg!(&parsed.errors);

        Value::new(Red::root(parsed.root?))
    }

    dbg!(act("(a b c)"));
    dbg!(act("(a b c d"));
    dbg!(act("(a . b)"));
    dbg!(act("a"));
    dbg!(act("(   )"));
    dbg!(act("(a b . c)"));
}
