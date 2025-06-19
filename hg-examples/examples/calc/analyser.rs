use std::mem;
use hg::metadata::Metadata;
use hg::token::{Ascii, Token};
use hg::tree::{Node, Verse};
use crate::ast::{DynEval, Number};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected node at {0}")]
    UnexpectedNode(Metadata),
    
    #[error("invalid 64-bit signed integer {0} at {1}")]
    InvalidInteger(u128, Metadata)
}

enum Element<'a> {
    Analysed(DynEval),
    Pending(Node<'a>)
}

fn flatten<'a, I: IntoIterator<Item = Verse<'a>>>(into_iter: I) -> Vec<Element<'a>> {
    into_iter.into_iter().flat_map(Verse::flatten).map(Element::Pending).collect()
}

pub fn analyse_elements<'a>(elements: Vec<Element<'a>>) -> Result<DynEval, Error> {
    let mut old_elements = elements;
    let mut new_elements = vec![];
    loop {
        let mut has_pending = false;
        for current in old_elements {
            let analysed = match current {
                Element::Analysed(eval) => eval,
                Element::Pending(pending) => {
                    has_pending = true;
                    match pending {
                        Node::Raw(Token::Integer(uint), metadata) => {
                            convert_integer(uint, metadata)?
                        }
                        Node::Raw(Token::Decimal(decimal), _) => {
                            Box::new(Number::Float(f64::from(decimal)))
                        }
                        Node::Raw(Token::Symbol(Ascii(b'+')), metadata) => {
                            todo!()
                        }
                        Node::List(verses, _) => {
                            analyse_elements(flatten(verses))?
                        }
                        Node::Prefix(_, _, _) => {todo!()}
                        other => {
                            Err(Error::UnexpectedNode(other.metadata().clone()))?
                        }
                    }
                }
            };
            new_elements.push(Element::Analysed(analysed));
        }

        old_elements = mem::take(&mut new_elements);
        if !has_pending {
            break;
        }
    }
    assert_eq!(1, old_elements.len());
    todo!()
}

fn convert_integer(uint: u128, metadata: Metadata) -> Result<DynEval, Error> {
    match i64::try_from(uint) {
        Ok(int) => {
            Ok(Box::new(Number::Integer(int)))
        }
        Err(_) => {
            Err(Error::InvalidInteger(uint, metadata))?
        }
    }
}

#[cfg(test)]
mod tests {
}