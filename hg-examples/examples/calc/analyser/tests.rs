use crate::analyser::{fold_products, take_last, Element, Error};
use crate::ast::{Eval, Expression, Number, Product};
use hg::metadata::{Location, Metadata};
use hg::token::Ascii;

#[test]
fn vec_take_last() {
    let mut vec = vec![10];
    assert_eq!(Some(10), take_last(&mut vec));
    assert!(vec.is_empty());
    assert_eq!(None, take_last(&mut vec));
}

fn metadata_bounds(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Metadata {
    debug_assert!(start_line <= end_line);
    debug_assert!(
        start_line == end_line && start_column <= end_column || start_line + 1 == end_line
    );
    Metadata {
        start: Some(Location {
            line: start_line,
            column: start_column,
        }),
        end: Some(Location {
            line: end_line,
            column: end_column,
        }),
    }
}

impl From<i32> for Element {
    fn from(int: i32) -> Self {
        Element::Expression(
            Expression::from(Number::Integer(int as i64)),
            Metadata::unspecified(),
        )
    }
}

impl From<u8> for Element {
    fn from(byte: u8) -> Self {
        Element::Operator(Ascii(byte), Metadata::unspecified())
    }
}

fn with_metadata<I: IntoIterator<Item = Element>>(
    into_iter: I,
) -> impl Iterator<Item = Result<Element, Error>> {
    into_iter.into_iter().enumerate().map(|(index, element)| {
        let metadata = Metadata {
            start: Some(Location { line: 1, column: (index * 2 + 1) as u32 }),
            end: Some(Location { line: 1, column: (index * 2 + 2) as u32 }),
        };
        match element {
            Element::Expression(expression, _) => {
                Element::Expression(expression, metadata)
            }
            Element::Operator(ascii, _) => {
                Element::Operator(ascii, metadata)
            }
        }
    }).map(Ok)
}

fn fold_products_ok<I: IntoIterator<Item = Element>>(into_iter: I) -> Vec<Element> {
    fold_products(with_metadata(into_iter)).unwrap()
}

fn fold_products_err<I: IntoIterator<Item = Element>>(into_iter: I) -> Error {
    fold_products(with_metadata(into_iter)).unwrap_err()
}

#[test]
fn fold_products_2() {
    let elements = [Element::from(3), Element::from(b'*'), Element::from(4)];
    let folded = fold_products_ok(elements);
    assert_eq!(1, folded.len());
    let (expr, _) = folded.into_iter().next().unwrap().into_expression().unwrap();
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
    let (expr, _) = folded.into_iter().next().unwrap().into_expression().unwrap();
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
    assert_eq!(
        vec![
            Element::Expression(
                Expression::from(Product(
                    Box::new(Expression::from(Number::Integer(3))),
                    Box::new(Expression::from(Number::Integer(4)))
                )),
                metadata_bounds(1, 1, 1, 6)
            ),
            Element::Operator(Ascii(b'+'), metadata_bounds(1, 7, 1, 8)),
            Element::Expression(Expression::from(Number::Integer(5)), metadata_bounds(1, 9, 1, 10))
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
    assert_eq!(
        vec![
            Element::Expression(
                Expression::from(Product(
                    Box::new(Expression::from(Number::Integer(3))),
                    Box::new(Expression::from(Number::Integer(4)))
                )),
                metadata_bounds(1, 1, 1, 6)
            ),
            Element::Operator(Ascii(b'+'), metadata_bounds(1, 7, 1, 8)),
            Element::Expression(
                Expression::from(Product(
                    Box::new(Expression::from(Number::Integer(5))),
                    Box::new(Expression::from(Number::Integer(6)))
                )),
                metadata_bounds(1, 9, 1, 14)
            ),
        ],
        folded
    );
}

#[test]
fn fold_products_stray_leading_operator_err() {
    let elements = [
        Element::from(b'*'),
    ];
    let err = fold_products_err(elements);
    assert_eq!("stray infix operator '*' at line 1, columns 1 to 2", err.to_string());
}

#[test]
fn fold_products_stray_mid_operator_err() {
    let elements = [
        Element::from(5),
        Element::from(b'*'),
        Element::from(6),
        Element::from(b'*'),
        Element::from(b'*'),
    ];
    let err = fold_products_err(elements);
    assert_eq!("stray infix operator '*' at line 1, columns 9 to 10", err.to_string());
}