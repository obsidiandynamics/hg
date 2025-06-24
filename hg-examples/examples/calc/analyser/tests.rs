use crate::analyser::{Element, Error, analyse, fold_mult, take_last};
use crate::ast::{Eval, Expression, Mult, Number};
use hg::lexer::Tokeniser;
use hg::metadata::{Location, Metadata};
use hg::parser::parse;
use hg::symbols::SymbolTable;
use hg::token::Ascii;
use hg_examples::testing::metadata_bounds;

#[test]
fn vec_take_last() {
    let mut vec = vec![10];
    assert_eq!(Some(10), take_last(&mut vec));
    assert!(vec.is_empty());
    assert_eq!(None, take_last(&mut vec));
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

fn with_metadata<I: IntoIterator<Item = Element>>(elements: I) -> impl Iterator<Item = Element> {
    elements.into_iter().enumerate().map(|(index, element)| {
        let metadata = Metadata {
            start: Some(Location {
                line: 1,
                column: (index * 2 + 1) as u32,
            }),
            end: Some(Location {
                line: 1,
                column: (index * 2 + 2) as u32,
            }),
        };
        match element {
            Element::Expression(expression, _) => Element::Expression(expression, metadata),
            Element::Operator(ascii, _) => Element::Operator(ascii, metadata),
        }
    })
}

fn fold_mult_ok<I: IntoIterator<Item = Element>>(elements: I) -> Vec<Element> {
    fold_mult(with_metadata(elements)).unwrap()
}

fn fold_mult_err<I: IntoIterator<Item = Element>>(elements: I) -> Error {
    fold_mult(with_metadata(elements)).unwrap_err()
}

#[test]
fn fold_mult_x2() {
    let elements = [Element::from(3), Element::from(b'*'), Element::from(4)];
    let folded = fold_mult_ok(elements);
    assert_eq!(1, folded.len());
    let (expr, _) = folded
        .into_iter()
        .next()
        .unwrap()
        .into_expression()
        .unwrap();
    assert_eq!(12.0, expr.eval());
}

#[test]
fn fold_mult_x3() {
    let elements = [
        Element::from(3),
        Element::from(b'*'),
        Element::from(4),
        Element::from(b'*'),
        Element::from(5),
    ];
    let folded = fold_mult_ok(elements);
    assert_eq!(1, folded.len());
    let (expr, _) = folded
        .into_iter()
        .next()
        .unwrap()
        .into_expression()
        .unwrap();
    assert_eq!(60.0, expr.eval());
}

#[test]
fn fold_mult_with_trailing_sum() {
    let elements = [
        Element::from(3),
        Element::from(b'*'),
        Element::from(4),
        Element::from(b'+'),
        Element::from(5),
    ];
    let folded = fold_mult_ok(elements);
    assert_eq!(
        vec![
            Element::Expression(
                Expression::from(Mult(
                    Box::new(Expression::from(Number::Integer(3))),
                    Box::new(Expression::from(Number::Integer(4)))
                )),
                metadata_bounds(1, 1, 1, 6)
            ),
            Element::Operator(Ascii(b'+'), metadata_bounds(1, 7, 1, 8)),
            Element::Expression(
                Expression::from(Number::Integer(5)),
                metadata_bounds(1, 9, 1, 10)
            )
        ],
        folded
    );
}

#[test]
fn fold_mult_with_mid_sum() {
    let elements = [
        Element::from(3),
        Element::from(b'*'),
        Element::from(4),
        Element::from(b'+'),
        Element::from(5),
        Element::from(b'*'),
        Element::from(6),
    ];
    let folded = fold_mult_ok(elements);
    assert_eq!(
        vec![
            Element::Expression(
                Expression::from(Mult(
                    Box::new(Expression::from(Number::Integer(3))),
                    Box::new(Expression::from(Number::Integer(4)))
                )),
                metadata_bounds(1, 1, 1, 6)
            ),
            Element::Operator(Ascii(b'+'), metadata_bounds(1, 7, 1, 8)),
            Element::Expression(
                Expression::from(Mult(
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
fn fold_mult_stray_leading_operator_err() {
    let elements = [Element::from(b'*')];
    let err = fold_mult_err(elements);
    assert_eq!(
        "stray operator '*' at line 1, columns 1 to 2",
        err.to_string()
    );
}

#[test]
fn fold_mult_stray_mid_operator_err() {
    let elements = [
        Element::from(5),
        Element::from(b'*'),
        Element::from(6),
        Element::from(b'*'),
        Element::from(b'*'),
    ];
    let err = fold_mult_err(elements);
    assert_eq!(
        "stray operator '*' at line 1, columns 9 to 10",
        err.to_string()
    );
}

fn analyse_ok(str: &str) -> f64 {
    let evaluate = || -> Result<_, Box<dyn std::error::Error>> {
        let tok = Tokeniser::new(str, SymbolTable::default());
        let root = parse(tok)?;
        let expr = analyse(root)?;
        Ok(expr.eval())
    };
    evaluate().unwrap()
}

#[test]
fn analyse_success() {
    for (input, expect) in [
        ("1", 1.0),
        ("-1", -1.0),
        ("-1 + 2", 1.0),
        ("2 + -1", 1.0),
        ("-2 + -1", -3.0),
        ("-2 - -1", -1.0),
    ] {
        println!("testing {input}");
        let actual = analyse_ok(input);
        assert_eq!(actual, expect, "for input {input}");
    }
}
