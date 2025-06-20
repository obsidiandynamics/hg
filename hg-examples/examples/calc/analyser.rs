use crate::ast::{DynEval, Number, Product};
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
    NoExpression
}

#[derive(Debug)]
enum Element {
    Expr(DynEval, Metadata),
    Symbol(Ascii, Metadata)
}

impl Element {
    fn into_expr(self) -> Option<(DynEval, Metadata)> {
        match self {
            Element::Expr(eval, metadata) => Some((eval, metadata)),
            Element::Symbol(_, _) => None,
        }
    }

    fn into_symbol(self) -> Option<(Ascii, Metadata)> {
        match self {
            Element::Expr(_, _) => None,
            Element::Symbol(ascii, metadata) => Some((ascii, metadata))
        }
    }
}

pub fn analyse(verse: Verse) -> Result<DynEval, Error> {
    let node_iter = flatten([verse]);
    let elements = process_elements(node_iter);
    let root = fold_elements(elements)?;
    Ok(root)
}

fn flatten<'a, I: IntoIterator<Item = Verse<'a>>>(into_iter: I) -> impl Iterator<Item = Node<'a>> {
    into_iter.into_iter().flat_map(Verse::flatten)
}

fn process_elements<'a, I: Iterator<Item = Node<'a>>>(iter: I) -> impl Iterator<Item = Result<Element, Error>> {
    iter.map(|node| {
        let element = match node {
            Node::Raw(Token::Integer(uint), metadata) => {
                let (eval, metadata) = convert_integer(uint, metadata)?;
                Element::Expr(eval, metadata)
            }
            Node::Raw(Token::Decimal(decimal), metadata) => {
                Element::Expr(Box::new(Number::Float(f64::from(decimal))), metadata)
            }
            Node::Raw(Token::Symbol(Ascii(byte)), metadata) => {
                match byte {
                    b'+' => Element::Symbol(Ascii(byte), metadata),
                    _ => Err(Error::UnexpectedSymbol(Ascii(byte), metadata))?
                }
            }
            Node::List(verses, metadata) => {
                let list_elements = process_elements(flatten(verses));
                let folded_list = fold_elements(list_elements)?;
                Element::Expr(folded_list, metadata)
            }
            Node::Prefix(_, _, _) => {todo!()}
            other => {
                Err(Error::UnexpectedNode(other.metadata().clone()))?
            }
        };
        Ok(element)
    })
}

fn fold_elements<I: Iterator<Item = Result<Element, Error>>>(iter: I) -> Result<DynEval, Error> {
    let sans_products = fold_products(iter)?;

    // should be no symbols remaining, just one top-level expression
    let mut iter = sans_products.into_iter();
    match iter.next() {
        None => {
            Err(Error::NoExpression)
        }
        Some(Element::Expr(eval, _)) => {
            match iter.next() {
                None => Ok(eval),
                Some(element) => {
                    let (ascii, metadata) = element.into_symbol().unwrap();
                    Err(Error::StrayInfixOperator(ascii, metadata))
                }
            }
        }
        Some(Element::Symbol(ascii, metadata)) => {
            Err(Error::StrayInfixOperator(ascii, metadata))
        }
    }
}

fn fold_products<I: Iterator<Item = Result<Element, Error>>>(iter: I) -> Result<Vec<Element>, Error> {
    let mut refined = vec![];
    for result in iter {
        println!("refined: {refined:?}");
        let element = result?;
        match element {
            Element::Expr(eval, metadata) => {
                match take_last(&mut refined) {
                    None => {
                        refined.push(Element::Expr(eval, metadata));
                    }
                    Some(Element::Expr(_, _)) => {
                        return Err(Error::StrayExpression(metadata))
                    }
                    Some(Element::Symbol(Ascii(b'*'), _)) => {
                        let before_last = take_last(&mut refined).unwrap();
                        let (lhs_eval, lhs_metadata) = before_last.into_expr().unwrap();
                        let product = Product(vec![lhs_eval, eval]);
                        refined.push(Element::Expr(Box::new(product), Metadata {
                            start: lhs_metadata.start,
                            end: metadata.end,
                        }));
                    }
                    Some(other @ Element::Symbol(_, _)) => {
                        refined.push(other)
                    }
                }
            }
            Element::Symbol(ascii, metadata) => {
                match take_last(&mut refined) {
                    None => {
                        return Err(Error::StrayInfixOperator(ascii, metadata))
                    }
                    Some(last @ Element::Expr(_, _)) => {
                        refined.push(last);
                        refined.push(Element::Symbol(ascii, metadata));
                    }
                    Some(Element::Symbol(_, _)) => {
                        return Err(Error::StrayInfixOperator(ascii, metadata))
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

// pub fn analyse_elements<'a>(elements: Vec<Element<'a>>) -> Result<DynEval, Error> {
//     let mut old_elements = elements;
//     let mut new_elements = vec![];
//     loop {
//         let mut has_pending = false;
//         for current in old_elements {
//             let analysed = match current {
//                 Element::Expr(eval) => eval,
//                 Element::Pending(pending) => {
//                     has_pending = true;
//                     match pending {
//                         Node::Raw(Token::Integer(uint), metadata) => {
//                             convert_integer(uint, metadata)?
//                         }
//                         Node::Raw(Token::Decimal(decimal), _) => {
//                             Box::new(Number::Float(f64::from(decimal)))
//                         }
//                         Node::Raw(Token::Symbol(Ascii(b'+')), metadata) => {
//                             todo!()
//                         }
//                         Node::List(verses, _) => {
//                             analyse_elements(flatten(verses).collect())?
//                         }
//                         Node::Prefix(_, _, _) => {todo!()}
//                         other => {
//                             Err(Error::UnexpectedNode(other.metadata().clone()))?
//                         }
//                     }
//                 }
//             };
//             new_elements.push(Element::Expr(analysed));
//         }
//
//         old_elements = mem::take(&mut new_elements);
//         if !has_pending {
//             break;
//         }
//     }
//     assert_eq!(1, old_elements.len());
//     todo!()
// }

fn convert_integer(uint: u128, metadata: Metadata) -> Result<(DynEval, Metadata), Error> {
    match i64::try_from(uint) {
        Ok(int) => {
            Ok((Box::new(Number::Integer(int)), metadata))
        }
        Err(_) => {
            Err(Error::InvalidInteger(uint, metadata))?
        }
    }
}

#[cfg(test)]
mod tests {
    use hg::metadata::Metadata;
    use hg::token::Ascii;
    use crate::analyser::{analyse, fold_products, take_last, Element, Error};
    use crate::ast::{DynEval, Number};

    #[test]
    fn vec_take_last() {
        let mut vec = vec![10];
        assert_eq!(Some(10), take_last(&mut vec));
        assert!(vec.is_empty());
        assert_eq!(None, take_last(&mut vec));
    }
    
    impl From<i32> for Element {
        fn from(int: i32) -> Self {
            Element::Expr(Box::new(Number::Integer(int as i64)), Metadata::unspecified())
        }
    }

    impl From<u8> for Element {
        fn from(byte: u8) -> Self {
            Element::Symbol(Ascii(byte), Metadata::unspecified())
        }
    }
    
    fn sans_error<I: IntoIterator<Item = Element>>(into_iter: I) -> impl Iterator<Item = Result<Element, Error>> {
        into_iter.into_iter().map(Ok)
    }
    
    fn fold_products_ok<I: IntoIterator<Item = Element>>(into_iter: I) -> DynEval {
        let folded = fold_products(sans_error(into_iter)).unwrap();
        assert_eq!(1, folded.len());
        let (expr, _) = folded.into_iter().next().unwrap().into_expr().unwrap();
        expr
    }
    
    #[test]
    fn fold_products_2() {
        // let elements = vec![Element::Expr(Box::new(Number::Integer(5)), Metadata::unspecified()), Element::Symbol(Ascii(b'*'), Metadata::unspecified()), Element::Expr(Box::new(Number::Integer(7)), Metadata::unspecified())];
        let elements = [Element::from(3), Element::from(b'*'), Element::from(4)];
        let expr = fold_products_ok(elements);
        assert_eq!(12.0, expr.eval());
    }

    #[test]
    fn fold_products_3() {
        let elements = [Element::from(3), Element::from(b'*'), Element::from(4), Element::from(b'*'), Element::from(5)];
        let expr = fold_products_ok(elements);
        assert_eq!(60.0, expr.eval());
    }
}