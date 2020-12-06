use crate::{Context, State, TokenKind};
use microtree::Green;

pub trait Parser<Tok: TokenKind> {
    fn parse(&self, state: State<Tok>, context: &Context<Tok>) -> (Option<Green>, State<Tok>);
}
