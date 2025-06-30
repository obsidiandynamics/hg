use crate::analyser::{Element, Error, analyse, fold_mult, take_last, flatten};
use crate::ast::{Eval, Expression, Mult, Number};
use hg::lexer::Tokeniser;
use hg::metadata::{Location, Metadata};
use hg::parser::parse;
use hg::symbols::SymbolTable;
use hg::token::{Ascii, Token};
use hg::{phrase, verse};
use hg::token::Token::Symbol;
use hg::tree::Node;
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

#[test]
fn flatten_one_verse_one_phrase() {
    let a = Node::Raw(Token::Symbol(Ascii(b'-')), Metadata::unspecified());
    let b = Node::Raw(Token::Symbol(Ascii(b'?')), Metadata::unspecified());
    let verse = verse![
        phrase![
            a.clone(), b.clone()
        ]
    ];
    assert_eq!(vec![a, b], flatten([verse]).unwrap().collect::<Vec<_>>());
}

#[test]
fn flatten_two_verses_err() {
    let verses = vec![verse![
        phrase![Node::Raw(Symbol(Ascii(b'-')), Metadata::unspecified())]
    ], verse![
        phrase![Node::Raw(Symbol(Ascii(b'-')), Metadata::unspecified())]
    ]];
    assert_eq!("unexpected comma separator", flatten(verses).map(|_|()).unwrap_err().to_string());
}

#[test]
fn flatten_two_phrases_err() {
    let verse = verse![
        phrase![Node::Raw(Symbol(Ascii(b'-')), Metadata::unspecified())],
        phrase![Node::Raw(Symbol(Ascii(b'-')), Metadata::unspecified())]
    ];
    assert_eq!("unexpected line separator", flatten([verse]).map(|_|()).unwrap_err().to_string());
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

fn evaluate(str: &'static str) -> Result<f64, Box<dyn std::error::Error>> {
    let tok = Tokeniser::new(str, SymbolTable::default());
    let root = parse(tok)?.ok_or("no expression")?;
    let expr = analyse(root)?;
    Ok(expr.eval())
}

#[test]
fn lex_parse_analyse() {
    for (input, expect) in [
        ("1", 1.0),
        ("-1", -1.0),
        ("-1 + 2", 1.0),
        ("2 + -1", 1.0),
        ("-2 + -1", -3.0),
        ("-2 - -1", -1.0),
        ("(1)", 1.0),
        ("(-1)", -1.0),
        ("-(-1)", 1.0),
        ("-(-(-1))", -1.0),
        ("5 - 4 - 3", -2.0),
        ("(5 - 4 - 3)", -2.0),
        ("(5 - 4) - 3", -2.0),
        ("-(5 - 4) - 3", -4.0),
        ("((5 - 4) - 3)", -2.0),
        ("5 - (4 - 3)", 4.0),
        ("(5 - (4 - 3))", 4.0),
        ("5 - -(4 - 3)", 6.0),
        ("5 - (-4 - 3)", 12.0),
        ("5 - (-4 - -3)", 6.0),
        ("2 * 3", 6.0),
        ("2 * 3 * 4", 24.0),
        ("2 * (3)", 6.0),
        ("(2) * 3", 6.0),
        ("2 * -3", -6.0),
        ("-2 * 3", -6.0),
        ("1 + 2 * 3", 7.0),
        ("1 + -2 * 3", -5.0),
        ("2 * 3 + 4 * 5", 26.0),
        ("2 * 3 - 4 * 5", -14.0),
        ("(2 * 3) - (4 * 5)", -14.0),
        ("2 * 3 + -4 * 5", -14.0),
        ("2 * 3 + -(4 * 5)", -14.0),
        ("2 * (3 + 4) * 5", 70.0),
        ("2 * -(3 + 4) * 5", -70.0),
        ("2 + 3 * 4 + 5", 19.0),
        ("8 / 4", 2.0),
        ("8 / -4", -2.0),
        ("-8 / -4", 2.0),
        ("8 / 4 / 2", 1.0),
        ("(8 / 4) / 2", 1.0),
        ("8 / (4 / 2)", 4.0),
        ("8 / -(4 / 2)", -4.0),
        ("16 / 2 * 4 / 2", 16.0),
        ("16 / (2 * 4) / 2", 1.0),
        ("(16 / 2) * (4 / 2)", 16.0),
        ("16 / 4 + 6 - 2", 8.0),
        ("16 / (4 + 4) - 3", -1.0),
        ("16 / (4 + 6 - 2)", 2.0),
        ("16 / -(4 + 6 - 2)", -2.0),
    ] {
        let actual = evaluate(input);
        match actual {
            Ok(actual) => {
                assert_eq!(expect, actual, "for input {input}");
            }
            Err(err) => {
                panic!("unexpected error \"{err}\" for input {input}")
            }
        }
    }
}

#[test]
fn lex_parse_analyse_err() {
    for (input, expect) in [
        ("", "no expression"),
        ("+", "stray operator '+' at line 1, columns 1 to 1"),
        ("1 1", "stray expression at line 1, columns 3 to 3"),
        ("1 + + 1", "stray operator '+' at line 1, columns 5 to 5"),
        ("1, 2", "unexpected token Symbol(Ascii(b','))"),
        ("1 + (2,3)", "unexpected comma separator"),
        ("1 + (2\n3)", "unexpected line separator"),
    ] {
        let actual = evaluate(input);
        match actual {
            Ok(actual) => {
                panic!("unexpected result {actual} for input {input}")
            }
            Err(err) => {
                assert_eq!(expect, err.to_string(), "for input {input}")
            }
        }
    }
}
