use crate::parser::{parse, Error};
use crate::sentence;
use crate::token::Token;
use crate::token::Token::{Colon, Comma, Dash, Decimal, Ident, Integer, LeftBrace, LeftParen, Newline, RightBrace, RightParen, Text};
use crate::tree::Node::{Cons, Container, List, Prefix, Raw};
use crate::tree::Sentence;

fn parse_ok(tokens: Vec<Token>) -> Vec<Sentence> {
    parse(tokens.into()).unwrap()
}

fn parse_err(tokens: Vec<Token>) -> Error {
    parse(tokens.into()).unwrap_err()
}

#[test]
fn flat_sequence_of_tokens() {
    let stanza = parse_ok(vec![Ident("hello".into()), Text("world".into()), Newline, Integer(42), Newline]);
    assert_eq!(vec![
        sentence![
            Raw(Ident("hello".into())),
            Raw(Text("world".into())),
        ],
        sentence![
            Raw(Integer(42)),
        ]
    ], stanza);
}

#[test]
fn unexpected_token_err() {
    let err = parse_err(vec![Comma]);
    assert_eq!("unexpected token Comma", err.to_string());
}

#[test]
fn container_empty() {
    let stanza = parse_ok(vec![LeftBrace, RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Container(vec![
            ]),
        ]
    ], stanza);
}

#[test]
fn container_nested_empty() {
    let stanza = parse_ok(vec![LeftBrace, LeftBrace, RightBrace, RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Container(vec![
                Container(vec![])
            ]),
        ]
    ], stanza);
}

#[test]
fn container_around_list() {
    let stanza = parse_ok(vec![LeftBrace, LeftParen, RightParen, RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Container(vec![
                List(vec![])
            ]),
        ]
    ], stanza);
}

#[test]
fn container_flat() {
    let stanza = parse_ok(vec![LeftBrace, Ident("hello".into()), Text("world".into()), Newline, RightBrace, Integer(42), Newline]);
    assert_eq!(vec![
        sentence![
            Container(vec![
                Raw(Ident("hello".into())),
                Raw(Text("world".into())),
                Raw(Newline),
            ]),
            Raw(Integer(42)),
        ]
    ], stanza);
}

#[test]
fn container_nested() {
    let stanza = parse_ok(vec![LeftBrace, Ident("hello".into()), LeftBrace, Text("world".into()), Newline, RightBrace, RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Container(vec![
                Raw(Ident("hello".into())),
                Container(
                    vec![
                        Raw(Text("world".into())),
                        Raw(Newline),
                    ]
                )
            ]),
        ]
    ], stanza);
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
    let stanza = parse_ok(vec![LeftParen, RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
            ]),
        ]
    ], stanza);
}

#[test]
fn list_nested_empty() {
    let stanza = parse_ok(vec![LeftParen, LeftParen, RightParen, RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![List(vec![])]
            ]),
        ]
    ], stanza);
}

#[test]
fn list_around_container() {
    let stanza = parse_ok(vec![LeftParen, LeftBrace, RightBrace, RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![Container(vec![])]
            ]),
        ]
    ], stanza);
}

#[test]
fn list_with_one_item_single() {
    let stanza = parse_ok(vec![LeftParen, Integer(1), RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![Raw(Integer(1))]
            ]),
        ]
    ], stanza);
}

#[test]
fn list_with_one_item_single_trailing_comma() {
    let stanza = parse_ok(vec![LeftParen, Integer(1), Comma, RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![Raw(Integer(1))]
            ]),
        ]
    ], stanza);
}

#[test]
fn list_with_one_item_sequence() {
    let stanza = parse_ok(vec![LeftParen, Integer(1), Integer(2), RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![Raw(Integer(1)), Raw(Integer(2))]
            ]),
        ]
    ], stanza);
}

#[test]
fn list_with_many_items() {
    let stanza = parse_ok(vec![LeftParen, Integer(1), Integer(2), Comma, Integer(3), RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![Raw(Integer(1)), Raw(Integer(2))],
                vec![Raw(Integer(3))]
            ]),
        ]
    ], stanza);
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
    let stanza = parse_ok(vec![Integer(1), Colon, Integer(2), Newline]);
    assert_eq!(vec![
        sentence![
            Cons(Box::new(Raw(Integer(1))), sentence![Raw(Integer(2))]),
        ]
    ], stanza);
}

#[test]
fn cons_single_long_tail() {
    let stanza = parse_ok(vec![Integer(1), Colon, Integer(2), Integer(3), Newline]);
    assert_eq!(vec![
        sentence![
            Cons(Box::new(Raw(Integer(1))), sentence![Raw(Integer(2)), Raw(Integer(3))]),
        ],
    ], stanza);
}

#[test]
fn cons_multiple() {
    let stanza = parse_ok(vec![Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), Newline]);
    assert_eq!(vec![
        sentence![
            Cons(Box::new(Cons(Box::new(Raw(Integer(1))), sentence![Raw(Integer(2)), Raw(Integer(3))])), sentence![Raw(Integer(4))]),
        ],
    ], stanza);
}

#[test]
fn cons_multiple_trailing_empty_segment() {
    let stanza = parse_ok(vec![Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), Colon, Newline]);
    assert_eq!(vec![
        sentence![
            Cons(Box::new(Cons(Box::new(Cons(Box::new(Raw(Integer(1))), sentence![Raw(Integer(2)), Raw(Integer(3))])), sentence![Raw(Integer(4))])), sentence![]),
        ],
    ], stanza);
}

#[test]
fn cons_with_container_tail() {
    let stanza = parse_ok(vec![Integer(1), Colon, LeftBrace, Integer(2), Newline, Integer(3), RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Cons(Box::new(Raw(Integer(1))), sentence![Container(vec![Raw(Integer(2)), Raw(Newline), Raw(Integer(3))])]),
        ],
    ], stanza);
}

#[test]
fn cons_with_list_tail() {
    let stanza = parse_ok(vec![Integer(1), Colon, LeftParen, Integer(2), Comma, Integer(3), Integer(4), RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            Cons(Box::new(Raw(Integer(1))), sentence![List(vec![vec![Raw(Integer(2))], vec![Raw(Integer(3)), Raw(Integer(4))]])]),
        ],
    ], stanza);
}

#[test]
fn cons_inside_container() {
    let stanza = parse_ok(vec![LeftBrace, Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Container(vec![
                Cons(Box::new(Cons(Box::new(Raw(Integer(1))), sentence![Raw(Integer(2)), Raw(Integer(3))])), sentence![Raw(Integer(4))]),
            ])
        ]
    ], stanza);
}

#[test]
fn cons_inside_list() {
    let stanza = parse_ok(vec![LeftParen, Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            List(vec![
                vec![
                    Cons(Box::new(Cons(Box::new(Raw(Integer(1))), sentence![Raw(Integer(2)), Raw(Integer(3))])), sentence![Raw(Integer(4))]),
                ]
            ])
        ]
    ], stanza);
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

#[test]
fn prefix_with_integer() {
    let stanza = parse_ok(vec![Dash, Integer(1), Newline]);
    assert_eq!(vec![
        sentence![
            Prefix(Dash, Box::new(Raw(Integer(1))))
        ]
    ], stanza);
}

#[test]
fn prefix_with_decimal() {
    let stanza = parse_ok(vec![Dash, Decimal(10, 5, 2), Newline]);
    assert_eq!(vec![
        sentence![  
            Prefix(Dash, Box::new(Raw(Decimal(10, 5, 2))))
        ]
    ], stanza);
}

#[test]
fn prefix_with_container() {
    let stanza = parse_ok(vec![Dash, LeftBrace, Integer(1), Newline, Integer(2), RightBrace, Newline]);
    assert_eq!(vec![
        sentence![
            Prefix(Dash, Box::new(Container(vec![Raw(Integer(1)), Raw(Newline), Raw(Integer(2))])))
        ]
    ], stanza);
}

#[test]
fn prefix_with_list() {
    let stanza = parse_ok(vec![Dash, LeftParen, Integer(1), Newline, Integer(2), Comma, Integer(3), RightParen, Newline]);
    assert_eq!(vec![
        sentence![
            Prefix(Dash, Box::new(List(vec![vec![Raw(Integer(1)), Raw(Newline), Raw(Integer(2))], vec![Raw(Integer(3))]])))
        ]
    ], stanza);
}

#[test]
fn prefix_unterminated_err() {
    let err = parse_err(vec![Dash]);
    assert_eq!("unterminated prefix", err.to_string());
}

#[test]
fn prefix_unexpected_token_err() {
    let err = parse_err(vec![Dash, Dash]);
    assert_eq!("unexpected token Dash", err.to_string());
}