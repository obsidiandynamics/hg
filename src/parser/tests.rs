use crate::parser::parse;
use crate::token::Token;
use crate::token::Token::{Dash, Ident, LeftBrace, Newline, RightBrace, Text};
use crate::tree::Node;
use crate::tree::Node::{Container, Raw};

fn parse_ok(tokens: Vec<Token>) -> Vec<Node> {
    parse(tokens.into()).unwrap()
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