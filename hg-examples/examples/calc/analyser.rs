use hg::metadata::Metadata;
use hg::tree::{Node, Verse};
use crate::ast::DynEval;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected node at {0}")]
    UnexpectedNode(Metadata)
}

pub fn analyse(verse: Verse) -> Result<DynEval, Error> {
    let mut previous = None;
    for current in verse.flatten() {
        match current {
            Node::Raw(_, _) => {}
            Node::List(_, _) => {}
            Node::Cons(_, _, metadata) => {
                return Err(Error::UnexpectedNode(metadata))
            }
            Node::Prefix(_, _, _) => {}
        }
        previous = Some(current);
    }
    todo!()
}