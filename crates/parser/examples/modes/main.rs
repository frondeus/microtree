use microtree::{Ast, Red};

mod generated;

use generated::*;
use microtree_parser::State;

mod str_parser {
    use microtree_parser::{parsers::*, Builder, Parser, SmolStr, TokenKind};

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Token {
        DQuote,
        OpenI,
        CloseI,
        Text,
    }

    impl std::fmt::Display for Token {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Token::DQuote => "`\"`",
                    Token::OpenI => "`${`",
                    Token::CloseI => "`}`",
                    Token::Text => "text",
                }
            )
        }
    }

    impl TokenKind for Token {
        type Extra = ();

        fn is_mergeable(self, other: Self) -> bool {
            self == Token::Text && (other == Token::Text || other == Token::CloseI)
        }

        fn lex(lexer: &mut microtree_parser::Lexer<Self>) -> Option<(Self, SmolStr)> {
            let input = lexer.input_mut();
            let i = input.as_ref();
            let peeked = i.chars().next()?;

            if peeked == '"' {
                return Some((Token::DQuote, input.chomp(1)));
            }

            if i.starts_with("${") {
                return Some((Token::OpenI, input.chomp(2)));
            }

            if peeked == '}' {
                return Some((Token::CloseI, input.chomp(1)));
            }

            Some((Token::Text, input.chomp(1)))
        }
    }

    pub fn interp() -> impl Parser<Token> {
        |builder: Builder<Token>| {
            builder
                .node()
                .parse(any_token()) // `${`
                .parse_mode(crate::parser::value())
                .parse(token(Token::CloseI))
                .finish()
        }
    }

    pub fn inner_string() -> impl Parser<Token> {
        |builder: Builder<Token>| {
            let mut builder = builder.name("StringVal").node();

            loop {
                builder = match builder.peek_token() {
                    Some(Token::Text) => builder.parse(|b: Builder<Token>| b.name("text").token()),
                    Some(Token::OpenI) => builder.parse(interp()),
                    Some(Token::DQuote) => break builder.finish(),
                    Some(Token::CloseI) => {
                        builder.parse(error("Found `}` which is... unexpected. ICE!"))
                    }
                    None => builder.parse(tokens(&[
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
    use microtree_parser::{parsers::*, Builder, Context, Parser, SmolStr, TokenKind};

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Token {
        Error,
        OpenP,
        CloseP,
        Dot,
        Atom,
        Whitespace,
        DQuote,
    }

    impl std::fmt::Display for Token {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Token::Error => "error",
                    Token::Atom => "atom",
                    Token::Whitespace => "whitespace",
                    Token::OpenP => "`(`",
                    Token::CloseP => "`)`",
                    Token::DQuote => "`\"`",
                    Token::Dot => "`.`",
                }
            )
        }
    }

    pub type Lexer<T = Token> = microtree_parser::Lexer<T>;

    impl TokenKind for Token {
        type Extra = ();

        fn is_mergeable(self, other: Self) -> bool {
            self == Token::Error && other == Token::Error
        }

        fn lex(lexer: &mut microtree_parser::Lexer<Self>) -> Option<(Self, SmolStr)> {
            let input = lexer.input_mut();
            let i = input.as_ref();
            let peeked = i.chars().next()?;

            if peeked.is_whitespace() {
                let rest = i.chars().take_while(|c| c.is_whitespace()).count();

                return Some((Token::Whitespace, input.chomp(rest)));
            }

            if peeked == '(' {
                return Some((Token::OpenP, input.chomp(1)));
            }

            if peeked == ')' {
                return Some((Token::CloseP, input.chomp(1)));
            }

            if peeked == '.' {
                return Some((Token::Dot, input.chomp(1)));
            }

            if peeked == '"' {
                return Some((Token::DQuote, input.chomp(1)));
            }

            let is_atom = |c: &char| c.is_ascii_alphanumeric() || *c == '_';

            if is_atom(&peeked) {
                let rest = i.chars().take_while(is_atom).count();

                return Some((Token::Atom, input.chomp(rest)));
            }

            Some((Token::Error, input.chomp(1)))
        }
    }

    pub fn trivia() -> impl Parser<Token> {
        |mut builder: Builder<Token>| match builder.peek_token() {
            Some(Token::Whitespace) => builder.name("trivia").parse(any_token()),
            _ => builder.none(),
        }
    }

    pub fn string() -> impl Parser<Token> {
        |builder: Builder<Token>| {
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

    pub fn sexp() -> impl Parser<Token> {
        |builder: Builder<Token>| {
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

    pub fn value() -> impl Parser<Token> {
        |builder: Builder<Token>| {
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

    let value = dbg!(act(r#"(a "  foo ${(1 2 3)} bar  " c)"#)).unwrap();

    println!("`{}`", value.red().green());
    println!("VALUES: ");
    for (idx, value) in value.as_list().unwrap().values().enumerate() {
        println!("{}. {}", idx, value.red().green());
    }
}
