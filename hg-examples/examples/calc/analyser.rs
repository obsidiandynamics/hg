use crate::ast::{Add, Div, Expression, Mult, Number, Sub};
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

    #[error("stray operator '{0}' at {1}")]
    StrayOperator(Ascii, Metadata),

    #[error("stray expression at {0}")]
    StrayExpression(Metadata),

    #[error("no expression")]
    NoExpression,
    
    #[error("unexpected line separator")]
    MultiplePhrases,

    #[error("unexpected comma separator")]
    MultipleVerses
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
    let node_iter = flatten([verse])?;
    let elements = process_elements(node_iter);
    let root = fold_elements(elements)?;
    Ok(root)
}

fn flatten<'a, I: IntoIterator<Item = Verse<'a>>>(verses: I) -> Result<impl Iterator<Item = Node<'a>>, Error> {
    let mut verses = verses.into_iter();
    match verses.next() {
        None => {
            Ok(vec![].into_iter())
        }
        Some(first_verse) => {
            match verses.next() {
                None => {
                    let mut phrases = first_verse.into_phrases().into_iter();
                    let first_phrase = phrases.next().unwrap();
                    match phrases.next() {
                        None => {
                            Ok(first_phrase.into_nodes().into_iter())
                        }
                        Some(_) => {
                            Err(Error::MultiplePhrases)
                        }
                    }
                }
                Some(_) => {
                    Err(Error::MultipleVerses)
                }
            }
        }
    }
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
            Node::Raw(Token::Decimal(decimal), metadata) => Element::Expression(
                Expression::from(Number::Float(f64::from(decimal))),
                metadata,
            ),
            Node::Raw(Token::Symbol(Ascii(byte)), metadata) => match byte {
                b'+' | b'-' | b'*' | b'/' => Element::Operator(Ascii(byte), metadata),
                _ => Err(Error::UnexpectedSymbol(Ascii(byte), metadata))?,
            },
            Node::List(verses, metadata) => {
                let list_elements = process_elements(flatten(verses)?);
                let folded = fold_elements(list_elements)?;
                Element::Expression(folded, metadata)
            }
            other => Err(Error::UnexpectedNode(other.metadata().clone()))?,
        };
        Ok(element)
    })
}

fn fold_elements<I: Iterator<Item = Result<Element, Error>>>(iter: I) -> Result<Expression, Error> {
    let mut elements = {
        let (min_elements, _) = iter.size_hint();
        Vec::with_capacity(min_elements)
    };
    for element in iter {
        elements.push(element?);
    }
    let elements = fold_prefix_sub(elements)?;
    let elements = fold_div(elements)?;
    let elements = fold_mult(elements)?;
    let elements = fold_infix_sub(elements)?;
    let elements = fold_add(elements)?;

    // should be no symbols remaining, just one top-level expression
    let mut iter = elements.into_iter();
    match iter.next() {
        None => Err(Error::NoExpression),
        Some(Element::Expression(eval, _)) => match iter.next() {
            None => Ok(eval),
            Some(element) => {
                let (ascii, metadata) = element.into_symbol().unwrap();
                Err(Error::StrayOperator(ascii, metadata))
            }
        },
        Some(Element::Operator(ascii, metadata)) => Err(Error::StrayOperator(ascii, metadata)),
    }
}

fn fold_prefix_sub<I: IntoIterator<Item = Element>>(elements: I) -> Result<Vec<Element>, Error> {
    fold_prefix(elements, |rhs| Expression::from(Sub(Box::new(Expression::from(Number::Integer(0))), rhs)), b'-')
}

fn fold_div<I: IntoIterator<Item = Element>>(elements: I) -> Result<Vec<Element>, Error> {
    fold_infix(elements, |lhs, rhs| Expression::from(Div(lhs, rhs)), b'/')
}

fn fold_mult<I: IntoIterator<Item = Element>>(elements: I) -> Result<Vec<Element>, Error> {
    fold_infix(elements, |lhs, rhs| Expression::from(Mult(lhs, rhs)), b'*')
}

fn fold_infix_sub<I: IntoIterator<Item = Element>>(elements: I) -> Result<Vec<Element>, Error> {
    fold_infix(elements, |lhs, rhs| Expression::from(Sub(lhs, rhs)), b'-')
}

fn fold_add<I: IntoIterator<Item = Element>>(elements: I) -> Result<Vec<Element>, Error> {
    fold_infix(elements, |lhs, rhs| Expression::from(Add(lhs, rhs)), b'+')
}

fn fold_prefix<
    I: IntoIterator<Item = Element>,
    C: Fn(Box<Expression>) -> Expression,
>(
    elements: I,
    combiner: C,
    operator: u8,
) -> Result<Vec<Element>, Error> {
    let mut refined = vec![];
    for element in elements {
        //println!("prefix refined: {refined:#?}");
        match element {
            Element::Expression(expr, metadata) => {
                let last = take_last(&mut refined);
                match last {
                    None => {
                        refined.push(Element::Expression(expr, metadata));
                    }
                    Some(Element::Expression(_, _)) => return Err(Error::StrayExpression(metadata)),
                    Some(last @ Element::Operator(Ascii(symbol), _)) => {
                        if symbol == operator {
                            let before_last = refined.last();
                            match before_last {
                                None | Some(Element::Operator(_, _)) => {
                                    let combined = combiner( Box::new(expr));
                                    refined.push(Element::Expression(
                                        combined,
                                        metadata,
                                    ));
                                }
                                Some(Element::Expression(_, _)) => {
                                    refined.push(last);
                                    refined.push(Element::Expression(expr, metadata));
                                }
                            }
                        } else {
                            refined.push(last);
                            refined.push(Element::Expression(expr, metadata));
                        }
                    }
                }
            },
            Element::Operator(ascii, metadata) => {
                refined.push(Element::Operator(ascii, metadata));
            },
        }
    }
    Ok(refined)
}

fn fold_infix<
    I: IntoIterator<Item = Element>,
    C: Fn(Box<Expression>, Box<Expression>) -> Expression,
>(
    elements: I,
    combiner: C,
    operator: u8,
) -> Result<Vec<Element>, Error> {
    let mut refined = vec![];
    for element in elements {
        // println!("refined: {refined:#?}");
        match element {
            Element::Expression(expr, metadata) => {
                let last = take_last(&mut refined);
                match last {
                    None => {
                        refined.push(Element::Expression(expr, metadata));
                    }
                    Some(Element::Expression(_, _)) => return Err(Error::StrayExpression(metadata)),
                    Some(last @ Element::Operator(Ascii(symbol), _)) => {
                        if symbol == operator {
                            let before_last = take_last(&mut refined).unwrap();
                            let (lhs_expr, lhs_metadata) = before_last.into_expression().unwrap();
                            let combined = combiner(Box::new(lhs_expr), Box::new(expr));
                            refined.push(Element::Expression(
                                combined,
                                Metadata {
                                    start: lhs_metadata.start,
                                    end: metadata.end,
                                },
                            ));
                        } else {
                            refined.push(last);
                            refined.push(Element::Expression(expr, metadata));
                        }
                    }
                }
            },
            Element::Operator(ascii, metadata) => {
                let last = take_last(&mut refined);
                match last {
                    None | Some(Element::Operator(_, _)) => {
                        return Err(Error::StrayOperator(ascii, metadata))
                    },
                    Some(last @ Element::Expression(_, _)) => {
                        refined.push(last);
                        refined.push(Element::Operator(ascii, metadata));
                    }
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
