use crate::{Context, State, TokenKind};
use microtree::Green;

pub trait Parser<'source, Tok: TokenKind<'source>> {
    fn parse<'ctx>(&self, state: State<'source, Tok>,
             context: &Context<'source, 'ctx, Tok>) -> (Option<Green>, State<'source, Tok>);
}
