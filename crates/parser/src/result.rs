use crate::Error;
use microtree::Green;

#[derive(Debug)]
pub struct ParseResult {
    pub root: Option<Green>,
    pub errors: Vec<Error>,
}
