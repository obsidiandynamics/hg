use crate::ast::{Expression, Number, Product};
use hg::metadata::Metadata;
use hg::token::{Ascii, Token};
use hg::tree::{Node, Verse};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected node at {0}")]
    UnexpectedNode(Metadata),

    #[error("invalid 64-bit signed integer {0} at {1}")]
    InvalidInteger(u128, Metadata),

    #[error("unexpected symbol '{0}' at {1}")]
    UnexpectedSymbol(Ascii, Metadata),

    #[error("stray infix operator '{0}' at {1}")]
    StrayInfixOperator(Ascii, Metadata),

    #[error("stray expression at {0}")]
    StrayExpression(Metadata),

    #[error("no expression")]
    NoExpression,
}

#[derive(Debug, PartialEq)]
enum Element {
    Expression(Expression, Metadata),
    Operator(Ascii, Metadata),
}

impl Element {
    fn into_expression(self) -> Option<(Expression, Metadata)> {
        match self {
            Element::Expression(eval, metadata) => Some((eval, metadata)),
            Element::Operator(_, _) => None,
        }
    }

    fn into_symbol(self) -> Option<(Ascii, Metadata)> {
        match self {
            Element::Expression(_, _) => None,
            Element::Operator(ascii, metadata) => Some((ascii, metadata)),
        }
    }
}

pub fn analyse(verse: Verse) -> Result<Expression, Error> {
    let node_iter = flatten([verse]);
    let elements = process_elements(node_iter);
    let root = fold_elements(elements)?;
    Ok(root)
}

fn flatten<'a, I: IntoIterator<Item = Verse<'a>>>(into_iter: I) -> impl Iterator<Item = Node<'a>> {
    into_iter.into_iter().flat_map(Verse::flatten)
}

fn process_elements<'a, I: Iterator<Item = Node<'a>>>(
    iter: I,
) -> impl Iterator<Item = Result<Element, Error>> {
    iter.map(|node| {
        let element = match node {
            Node::Raw(Token::Integer(uint), metadata) => {
                let (eval, metadata) = convert_integer(uint, metadata)?;
                Element::Expression(eval, metadata)
            }
            Node::Raw(Token::Decimal(decimal), metadata) => {
                Element::Expression(Expression::from(Number::Float(f64::from(decimal))), metadata)
            }
            Node::Raw(Token::Symbol(Ascii(byte)), metadata) => match byte {
                b'+' => Element::Operator(Ascii(byte), metadata),
                _ => Err(Error::UnexpectedSymbol(Ascii(byte), metadata))?,
            },
            Node::List(verses, metadata) => {
                let list_elements = process_elements(flatten(verses));
                let folded_list = fold_elements(list_elements)?;
                Element::Expression(folded_list, metadata)
            }
            Node::Prefix(_, _, _) => {
                todo!()
            }
            other => Err(Error::UnexpectedNode(other.metadata().clone()))?,
        };
        Ok(element)
    })
}

fn fold_elements<I: Iterator<Item = Result<Element, Error>>>(iter: I) -> Result<Expression, Error> {
    let sans_products = fold_products(iter)?;

    // should be no symbols remaining, just one top-level expression
    let mut iter = sans_products.into_iter();
    match iter.next() {
        None => Err(Error::NoExpression),
        Some(Element::Expression(eval, _)) => match iter.next() {
            None => Ok(eval),
            Some(element) => {
                let (ascii, metadata) = element.into_symbol().unwrap();
                Err(Error::StrayInfixOperator(ascii, metadata))
            }
        },
        Some(Element::Operator(ascii, metadata)) => Err(Error::StrayInfixOperator(ascii, metadata)),
    }
}

fn fold_products<I: Iterator<Item = Result<Element, Error>>>(
    iter: I,
) -> Result<Vec<Element>, Error> {
    let mut refined = vec![];
    for result in iter {
        println!("refined: {refined:#?}");
        let element = result?;
        match element {
            Element::Expression(eval, metadata) => match take_last(&mut refined) {
                None => {
                    refined.push(Element::Expression(eval, metadata));
                }
                Some(Element::Expression(_, _)) => return Err(Error::StrayExpression(metadata)),
                Some(Element::Operator(Ascii(b'*'), _)) => {
                    let before_last = take_last(&mut refined).unwrap();
                    let (lhs_eval, lhs_metadata) = before_last.into_expression().unwrap();
                    let product = Product(Box::new(lhs_eval), Box::new(eval));
                    refined.push(Element::Expression(
                        Expression::from(product),
                        Metadata {
                            start: lhs_metadata.start,
                            end: metadata.end,
                        },
                    ));
                }
                Some(other @ Element::Operator(_, _)) => {
                    refined.push(other);
                    refined.push(Element::Expression(eval, metadata));
                }
            },
            Element::Operator(ascii, metadata) => match take_last(&mut refined) {
                None => return Err(Error::StrayInfixOperator(ascii, metadata)),
                Some(last @ Element::Expression(_, _)) => {
                    refined.push(last);
                    refined.push(Element::Operator(ascii, metadata));
                }
                Some(Element::Operator(_, _)) => {
                    return Err(Error::StrayInfixOperator(ascii, metadata));
                }
            },
        }
    }
    Ok(refined)
}

fn take_last<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        None
    } else {
        let last = vec.remove(vec.len() - 1);
        Some(last)
    }
}

fn convert_integer(uint: u128, metadata: Metadata) -> Result<(Expression, Metadata), Error> {
    match i64::try_from(uint) {
        Ok(int) => Ok((Expression::from(Number::Integer(int)), metadata)),
        Err(_) => Err(Error::InvalidInteger(uint, metadata))?,
    }
}

#[cfg(test)]
mod tests;
