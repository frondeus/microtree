use microtree::{Ast, Red};

mod generated;

use generated::*;
use microtree_parser::State;

mod str_parser {
    use microtree_parser::{parsers::*, Builder, Parser, TokenKind, Logos};
    use derive_more::Display;


    #[derive(Debug, PartialEq, Clone, Copy, Logos, Display)]
    pub enum Token {
        #[display("`\"`")]
        #[token("\"")]
        DQuote,
        #[display("`${`")]
        #[token("${")]
        OpenI,
        #[display("`}`")]
        #[token("}")]
        CloseI,
        #[display("text")]
        #[error]
        Text
    }

    impl TokenKind<'_> for Token {
        fn mergeable(self, other: Self) -> bool {
            self == Self::Text && (other == Self::Text || other == Self::CloseI)
        }
    }

    pub fn interp<'s>() -> impl Parser<'s, Token> {
        |builder: Builder<'s, '_, Token>| {
            builder
                .name("interp")
                .node()
                .parse(any_token()) // `${`
                .parse_mode(crate::parser::value())
                .parse(token(Token::CloseI))
                .finish()
        }
    }

    pub fn inner_string<'s>() -> impl Parser<'s, Token> {
        |builder: Builder<'s, '_, Token>| {
            let mut builder = builder.name("StringVal").node();

            loop {
                builder = match builder.peek_token() {
                    Some(Token::Text) => builder.parse(|b: Builder<'s, '_, Token>| b.name("text").token()),
                    Some(Token::OpenI) => builder.parse(interp()),
                    Some(Token::DQuote) => break builder.finish(),
                    Some(Token::CloseI) => {
                        builder.parse(error("Found `}` which is... unexpected. ICE!"))
                    }
                    _ => builder.parse(tokens(&[
                        Token::Text,
                        Token::OpenI,
                        Token::CloseI,
                        Token::DQuote,
                    ])),
                }
            }
        }
    }
}
mod parser {
    use microtree_parser::{parsers::*, Builder, Context, Parser, Logos, TokenKind};
    use derive_more::Display;

    #[derive(Debug, PartialEq, Clone, Copy, Logos, Display)]
    pub enum Token {
        #[error] #[display("error")]
        Error,
        #[token("(")] #[display("`(`")]
        OpenP,
        #[token(")")] #[display("`)`")]
        CloseP,
        #[token(".")] #[display("`.`")]
        Dot,
        #[regex("[0-9a-zA-Z_]+")] #[display("atom")]
        Atom,
        #[regex(r#"[ \t\n]+"#)] #[display("whitespace")]
        Whitespace,
        #[token("\"")] #[display("`\"`")]
        DQuote,
    }

    impl TokenKind<'_> for Token {}

    pub type Lexer<'s, T = Token> = microtree_parser::Lexer<'s, T>;

    pub fn trivia<'s>() -> impl Parser<'s, Token> {
        |mut builder: Builder<'s, '_, Token>| match builder.peek_token() {
            Some(Token::Whitespace) => builder.name("trivia").parse(any_token()),
            _ => builder.none(),
        }
    }

    pub fn string<'s>() -> impl Parser<'s, Token> {
        |builder: Builder<'s, '_, Token>| {
            let trivia = trivia();
            let ctx = Context {
                leading_trivia: Some(&trivia),
                ..Default::default()
            };
            let prev_ctx = builder.get_ctx();
            builder
                .name("String")
                .set_ctx(&ctx)
                .node()
                .parse(any_token()) // `"`
                .parse_mode(crate::str_parser::inner_string())
                .set_ctx(prev_ctx)
                .parse(token(Token::DQuote))
                .finish()
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
                Some(Token::DQuote) => builder.parse(string()),
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

    let value = dbg!(act(r#"(a "  foo ${(1 2 3)} bar $ baz }  " c)"#)).unwrap();

    println!("`{}`", value.red().green());
    println!("VALUES: ");
    for (idx, value) in value.as_list().unwrap().values().enumerate() {
        println!("{}. {}", idx, value.red().green());
    }
}
