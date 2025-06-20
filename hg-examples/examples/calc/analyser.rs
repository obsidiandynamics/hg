use crate::ast::{EvalKind, Number, Product};
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
    Expr(EvalKind, Metadata),
    Symbol(Ascii, Metadata),
}

impl Element {
    fn into_expr(self) -> Option<(EvalKind, Metadata)> {
        match self {
            Element::Expr(eval, metadata) => Some((eval, metadata)),
            Element::Symbol(_, _) => None,
        }
    }

    fn into_symbol(self) -> Option<(Ascii, Metadata)> {
        match self {
            Element::Expr(_, _) => None,
            Element::Symbol(ascii, metadata) => Some((ascii, metadata)),
        }
    }
}

pub fn analyse(verse: Verse) -> Result<EvalKind, Error> {
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
                Element::Expr(eval, metadata)
            }
            Node::Raw(Token::Decimal(decimal), metadata) => {
                Element::Expr(EvalKind::from(Number::Float(f64::from(decimal))), metadata)
            }
            Node::Raw(Token::Symbol(Ascii(byte)), metadata) => match byte {
                b'+' => Element::Symbol(Ascii(byte), metadata),
                _ => Err(Error::UnexpectedSymbol(Ascii(byte), metadata))?,
            },
            Node::List(verses, metadata) => {
                let list_elements = process_elements(flatten(verses));
                let folded_list = fold_elements(list_elements)?;
                Element::Expr(folded_list, metadata)
            }
            Node::Prefix(_, _, _) => {
                todo!()
            }
            other => Err(Error::UnexpectedNode(other.metadata().clone()))?,
        };
        Ok(element)
    })
}

fn fold_elements<I: Iterator<Item = Result<Element, Error>>>(iter: I) -> Result<EvalKind, Error> {
    let sans_products = fold_products(iter)?;

    // should be no symbols remaining, just one top-level expression
    let mut iter = sans_products.into_iter();
    match iter.next() {
        None => Err(Error::NoExpression),
        Some(Element::Expr(eval, _)) => match iter.next() {
            None => Ok(eval),
            Some(element) => {
                let (ascii, metadata) = element.into_symbol().unwrap();
                Err(Error::StrayInfixOperator(ascii, metadata))
            }
        },
        Some(Element::Symbol(ascii, metadata)) => Err(Error::StrayInfixOperator(ascii, metadata)),
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
            Element::Expr(eval, metadata) => match take_last(&mut refined) {
                None => {
                    refined.push(Element::Expr(eval, metadata));
                }
                Some(Element::Expr(_, _)) => return Err(Error::StrayExpression(metadata)),
                Some(Element::Symbol(Ascii(b'*'), _)) => {
                    let before_last = take_last(&mut refined).unwrap();
                    let (lhs_eval, lhs_metadata) = before_last.into_expr().unwrap();
                    let product = Product(Box::new(lhs_eval), Box::new(eval));
                    refined.push(Element::Expr(
                        EvalKind::from(product),
                        Metadata {
                            start: lhs_metadata.start,
                            end: metadata.end,
                        },
                    ));
                }
                Some(other @ Element::Symbol(_, _)) => {
                    refined.push(other);
                    refined.push(Element::Expr(eval, metadata));
                }
            },
            Element::Symbol(ascii, metadata) => match take_last(&mut refined) {
                None => return Err(Error::StrayInfixOperator(ascii, metadata)),
                Some(last @ Element::Expr(_, _)) => {
                    refined.push(last);
                    refined.push(Element::Symbol(ascii, metadata));
                }
                Some(Element::Symbol(_, _)) => {
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

fn convert_integer(uint: u128, metadata: Metadata) -> Result<(EvalKind, Metadata), Error> {
    match i64::try_from(uint) {
        Ok(int) => Ok((EvalKind::from(Number::Integer(int)), metadata)),
        Err(_) => Err(Error::InvalidInteger(uint, metadata))?,
    }
}

#[cfg(test)]
mod tests {
    use crate::analyser::{Element, Error, fold_products, take_last};
    use crate::ast::{Eval, EvalKind, Number, Product, Sum};
    use hg::metadata::Metadata;
    use hg::token::Ascii;

    #[test]
    fn vec_take_last() {
        let mut vec = vec![10];
        assert_eq!(Some(10), take_last(&mut vec));
        assert!(vec.is_empty());
        assert_eq!(None, take_last(&mut vec));
    }

    impl From<i32> for Element {
        fn from(int: i32) -> Self {
            Element::Expr(
                EvalKind::from(Number::Integer(int as i64)),
                Metadata::unspecified(),
            )
        }
    }

    impl From<u8> for Element {
        fn from(byte: u8) -> Self {
            Element::Symbol(Ascii(byte), Metadata::unspecified())
        }
    }

    fn sans_error<I: IntoIterator<Item = Element>>(
        into_iter: I,
    ) -> impl Iterator<Item = Result<Element, Error>> {
        into_iter.into_iter().map(Ok)
    }

    fn fold_products_ok<I: IntoIterator<Item = Element>>(into_iter: I) -> Vec<Element> {
        fold_products(sans_error(into_iter)).unwrap()
    }

    #[test]
    fn fold_products_2() {
        // let elements = vec![Element::Expr(Box::new(Number::Integer(5)), Metadata::unspecified()), Element::Symbol(Ascii(b'*'), Metadata::unspecified()), Element::Expr(Box::new(Number::Integer(7)), Metadata::unspecified())];
        let elements = [Element::from(3), Element::from(b'*'), Element::from(4)];
        let folded = fold_products_ok(elements);
        assert_eq!(1, folded.len());
        let (expr, _) = folded.into_iter().next().unwrap().into_expr().unwrap();
        assert_eq!(12.0, expr.eval());
    }

    #[test]
    fn fold_products_3() {
        let elements = [
            Element::from(3),
            Element::from(b'*'),
            Element::from(4),
            Element::from(b'*'),
            Element::from(5),
        ];
        let folded = fold_products_ok(elements);
        assert_eq!(1, folded.len());
        let (expr, _) = folded.into_iter().next().unwrap().into_expr().unwrap();
        assert_eq!(60.0, expr.eval());
    }

    #[test]
    fn fold_products_with_trailing_sum() {
        let elements = [
            Element::from(3),
            Element::from(b'*'),
            Element::from(4),
            Element::from(b'+'),
            Element::from(5),
        ];
        let folded = fold_products_ok(elements);
        assert_eq!(3, folded.len());
        assert_eq!(
            vec![
                Element::Expr(
                    EvalKind::from(Product(
                        Box::from(EvalKind::from(Number::Integer(3))),
                        Box::from(EvalKind::from(Number::Integer(4)))
                    )),
                    Metadata::unspecified()
                ),
                Element::Symbol(Ascii(b'+'), Metadata::unspecified()),
                Element::Expr(EvalKind::from(Number::Integer(5)), Metadata::unspecified())
            ],
            folded
        );
    }

    #[test]
    fn fold_products_with_mid_sum() {
        let elements = [
            Element::from(3),
            Element::from(b'*'),
            Element::from(4),
            Element::from(b'+'),
            Element::from(5),
            Element::from(b'*'),
            Element::from(6),
        ];
        let folded = fold_products_ok(elements);
        assert_eq!(3, folded.len());
        assert_eq!(
            vec![
                Element::Expr(
                    EvalKind::from(Product(
                        Box::from(EvalKind::from(Number::Integer(3))),
                        Box::from(EvalKind::from(Number::Integer(4)))
                    )),
                    Metadata::unspecified()
                ),
                Element::Symbol(Ascii(b'+'), Metadata::unspecified()),
                Element::Expr(
                    EvalKind::from(Product(
                        Box::from(EvalKind::from(Number::Integer(5))),
                        Box::from(EvalKind::from(Number::Integer(6)))
                    )),
                    Metadata::unspecified()
                ),
            ],
            folded
        );
    }
}
