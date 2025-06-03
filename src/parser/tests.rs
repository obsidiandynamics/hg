use crate::parser::{parse, Error};
use crate::token::Token;
use crate::token::Token::{Colon, Comma, Dash, Ident, Integer, LeftBrace, LeftParen, Newline, RightBrace, RightParen, Text};
use crate::tree::Node;
use crate::tree::Node::{Cons, Container, List, Raw};

fn parse_ok(tokens: Vec<Token>) -> Vec<Node> {
    parse(tokens.into()).unwrap()
}

fn parse_err(tokens: Vec<Token>) -> Error {
    parse(tokens.into()).unwrap_err()
}

#[test]
fn flat_sequence_of_tokens() {
    let nodes = parse_ok(vec![Ident("hello".into()), Text("world".into()), Newline, Dash, Newline]);
    assert_eq!(vec![
        Raw(Ident("hello".into())),
        Raw(Text("world".into())),
        Raw(Newline),
        Raw(Dash),
        Raw(Newline)
    ], nodes);
}

#[test]
fn unexpected_token_err() {
    let err = parse_err(vec![Comma]);
    assert_eq!("unexpected token Comma", err.to_string());
}

#[test]
fn container_empty() {
    let nodes = parse_ok(vec![LeftBrace, RightBrace]);
    assert_eq!(vec![
        Container(vec![
        ]),
    ], nodes);
}

#[test]
fn container_nested_empty() {
    let nodes = parse_ok(vec![LeftBrace, LeftBrace, RightBrace, RightBrace]);
    assert_eq!(vec![
        Container(vec![
            Container(vec![])
        ]),
    ], nodes);
}

#[test]
fn container_around_list() {
    let nodes = parse_ok(vec![LeftBrace, LeftParen, RightParen, RightBrace]);
    assert_eq!(vec![
        Container(vec![
            List(vec![])
        ]),
    ], nodes);
}

#[test]
fn container_flat() {
    let nodes = parse_ok(vec![LeftBrace, Ident("hello".into()), Text("world".into()), Newline, RightBrace, Dash, Newline]);
    assert_eq!(vec![
        Container(vec![
            Raw(Ident("hello".into())),
            Raw(Text("world".into())),
            Raw(Newline),
        ]),
        Raw(Dash),
        Raw(Newline)
    ], nodes);
}

#[test]
fn container_nested() {
    let nodes = parse_ok(vec![LeftBrace, Ident("hello".into()), LeftBrace, Text("world".into()), Newline, RightBrace, RightBrace]);
    assert_eq!(vec![
        Container(vec![
            Raw(Ident("hello".into())),
            Container(
                vec![
                    Raw(Text("world".into())),
                    Raw(Newline),
                ]
            )
        ]),
    ], nodes);
}

#[test]
fn container_unterminated_err() {
    let err = parse_err(vec![LeftBrace, Ident("hello".into()), Newline]);
    assert_eq!("unterminated container", err.to_string());
}

#[test]
fn container_expected_token_err() {
    let err = parse_err(vec![LeftBrace, Ident("hello".into()), RightParen]);
    assert_eq!("unexpected token RightParen", err.to_string());
}

#[test]
fn list_empty() {
    let nodes = parse_ok(vec![LeftParen, RightParen]);
    assert_eq!(vec![
        List(vec![
        ]),
    ], nodes);
}

#[test]
fn list_nested_empty() {
    let nodes = parse_ok(vec![LeftParen, LeftParen, RightParen, RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![List(vec![])]
        ]),
    ], nodes);
}

#[test]
fn list_around_container() {
    let nodes = parse_ok(vec![LeftParen, LeftBrace, RightBrace, RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![Container(vec![])]
        ]),
    ], nodes);
}

#[test]
fn list_with_one_item_single() {
    let nodes = parse_ok(vec![LeftParen, Integer(1), RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![Raw(Integer(1))]
        ]),
    ], nodes);
}

#[test]
fn list_with_one_item_single_trailing_comma() {
    let nodes = parse_ok(vec![LeftParen, Integer(1), Comma, RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![Raw(Integer(1))]
        ]),
    ], nodes);
}

#[test]
fn list_with_one_item_sequence() {
    let nodes = parse_ok(vec![LeftParen, Integer(1), Integer(2), RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![Raw(Integer(1)), Raw(Integer(2))]
        ]),
    ], nodes);
}

#[test]
fn list_with_many_items() {
    let nodes = parse_ok(vec![LeftParen, Integer(1), Integer(2), Comma, Integer(3), RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![Raw(Integer(1)), Raw(Integer(2))],
            vec![Raw(Integer(3))]
        ]),
    ], nodes);
}

#[test]
fn list_empty_segment_err() {
    let err = parse_err(vec![LeftParen, Integer(1), Comma, Comma]);
    assert_eq!("empty list segment", err.to_string());
}

#[test]
fn list_unterminated_err() {
    let err = parse_err(vec![LeftParen, Ident("hello".into()), Newline]);
    assert_eq!("unterminated list", err.to_string());
}

#[test]
fn list_expected_token_err() {
    let err = parse_err(vec![LeftParen, Ident("hello".into()), RightBrace]);
    assert_eq!("unexpected token RightBrace", err.to_string());
}

#[test]
fn cons_single() {
    let nodes = parse_ok(vec![Integer(1), Colon, Integer(2), Newline]);
    assert_eq!(vec![
        Cons(Box::from(Raw(Integer(1))), vec![Raw(Integer(2))]),
        Raw(Newline),
    ], nodes);
}

#[test]
fn cons_single_long_tail() {
    let nodes = parse_ok(vec![Integer(1), Colon, Integer(2), Integer(3), Newline]);
    assert_eq!(vec![
        Cons(Box::from(Raw(Integer(1))), vec![Raw(Integer(2)), Raw(Integer(3))]),
        Raw(Newline),
    ], nodes);
}

#[test]
fn cons_multiple() {
    let nodes = parse_ok(vec![Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), Newline]);
    assert_eq!(vec![
        Cons(Box::from(Cons(Box::from(Raw(Integer(1))), vec![Raw(Integer(2)), Raw(Integer(3))])), vec![Raw(Integer(4))]),
        Raw(Newline),
    ], nodes);
}

#[test]
fn cons_multiple_trailing_empty_segment() {
    let nodes = parse_ok(vec![Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), Colon, Newline]);
    assert_eq!(vec![
        Cons(Box::from(Cons(Box::from(Cons(Box::from(Raw(Integer(1))), vec![Raw(Integer(2)), Raw(Integer(3))])), vec![Raw(Integer(4))])), vec![]),
        Raw(Newline),
    ], nodes);
}

#[test]
fn cons_with_container_tail() {
    let nodes = parse_ok(vec![Integer(1), Colon, LeftBrace, Integer(2), Newline, Integer(3), RightBrace, Newline]);
    assert_eq!(vec![
        Cons(Box::from(Raw(Integer(1))), vec![Container(vec![Raw(Integer(2)), Raw(Newline), Raw(Integer(3))])]),
        Raw(Newline),
    ], nodes);
}

#[test]
fn cons_with_list_tail() {
    let nodes = parse_ok(vec![Integer(1), Colon, LeftParen, Integer(2), Comma, Integer(3), Integer(4), RightParen, Newline]);
    assert_eq!(vec![
        Cons(Box::from(Raw(Integer(1))), vec![List(vec![vec![Raw(Integer(2))], vec![Raw(Integer(3)), Raw(Integer(4))]])]),
        Raw(Newline),
    ], nodes);
}

#[test]
fn cons_inside_container() {
    let nodes = parse_ok(vec![LeftBrace, Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), RightBrace]);
    assert_eq!(vec![
        Container(vec![
            Cons(Box::from(Cons(Box::from(Raw(Integer(1))), vec![Raw(Integer(2)), Raw(Integer(3))])), vec![Raw(Integer(4))]),
        ])
    ], nodes);
}

#[test]
fn cons_inside_list() {
    let nodes = parse_ok(vec![LeftParen, Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), RightParen]);
    assert_eq!(vec![
        List(vec![
            vec![
                Cons(Box::from(Cons(Box::from(Raw(Integer(1))), vec![Raw(Integer(2)), Raw(Integer(3))])), vec![Raw(Integer(4))]),
            ]
        ])
    ], nodes);
}

#[test]
fn cons_empty_starting_segment_err() {
    let err = parse_err(vec![Colon, Integer(2)]);
    assert_eq!("empty cons segment", err.to_string());
}

#[test]
fn cons_empty_intermediate_segment_err() {
    let err = parse_err(vec![Integer(1), Colon, Integer(2), Colon, Colon]);
    assert_eq!("empty cons segment", err.to_string());
}

#[test]
fn cons_unterminated_err() {
    let err = parse_err(vec![Integer(1), Colon, Integer(2)]);
    assert_eq!("unterminated cons", err.to_string());
}