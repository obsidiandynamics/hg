use crate::metadata::{Location, Metadata};
use crate::parser::{parse, Error};
use crate::token::ListDelimiter::{Brace, Paren};
use crate::token::Token::{Decimal, Ident, Integer, Left, Newline, Right, Symbol, Text};
use crate::token::{Ascii, Token};
use crate::tree::Node::{Cons, List, Prefix, Raw};
use crate::tree::Verse;
use crate::{lexer, phrase, verse};
use std::iter::{Enumerate, Map};
use std::vec::IntoIter;

fn map_metadata(tokens: Vec<Token>) -> Map<Enumerate<IntoIter<Token>>, fn((usize, Token)) -> Result<(Token, Metadata), Box<lexer::Error>>> {
    tokens
        .into_iter()
        .enumerate()
        .map(|(index, token)| {
            Ok((token, Metadata {start: Some(Location { line: 1, column: index as u32 * 2 + 1}), end: Some(Location { line: 1, column: index as u32 * 2 + 2})}))
        })
}

fn parse_ok(tokens: Vec<Token>) -> Verse {
    parse(map_metadata(tokens)).unwrap()
}

fn parse_err(tokens: Vec<Token>) -> Error {
    parse(map_metadata(tokens)).unwrap_err()
}

#[test]
fn flat_sequence_of_tokens() {
    let verse = parse_ok(vec![Ident("hello".into()), Text("world".into()), Newline, Integer(42), Newline]);
    assert_eq!(verse![
        phrase![
            Raw(Ident("hello".into()), Metadata::bounds(1, 1, 1, 2)),
            Raw(Text("world".into()), Metadata::bounds(1, 3, 1, 4)),
        ],
        phrase![
            Raw(Integer(42), Metadata::bounds(1, 7, 1, 8)),
        ]
    ], verse);
}

#[test]
fn unterminated_phrase_err() {
    let err = parse_err(vec![Ident("hello".into()), Text("world".into())]);
    assert_eq!("unterminated phrase", err.to_string());
}

#[test]
fn unexpected_token_err() {
    let err = parse_err(vec![Symbol(Ascii(b','))]);
    assert_eq!("unexpected token Symbol(Ascii(b','))", err.to_string());
}

#[test]
fn brace_list_empty() {
    let verse = parse_ok(vec![Left(Brace), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![], Metadata::bounds(1, 1, 1, 4)),
        ]
    ], verse);
}

#[test]
fn brace_list_nested_empty() {
    let verse = parse_ok(vec![Left(Brace), Left(Brace), Right(Brace), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        List(vec![], Metadata::bounds(1, 3, 1, 6))
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 8)),
        ]
    ], verse);
}

#[test]
fn brace_list_around_paren_list() {
    let verse = parse_ok(vec![Left(Brace), Left(Paren), Right(Paren), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        List(vec![], Metadata::bounds(1, 3, 1, 6))
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 8)),
        ]
    ], verse);
}

#[test]
fn brace_list_flat() {
    let verse = parse_ok(vec![Left(Brace), Ident("hello".into()), Text("world".into()), Newline, Right(Brace), Integer(42), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Raw(Ident("hello".into()), Metadata::bounds(1, 3, 1, 4)),
                        Raw(Text("world".into()), Metadata::bounds(1, 5, 1, 6)),
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 10)),
            Raw(Integer(42), Metadata::bounds(1, 11, 1, 12)),
        ]
    ], verse);
}

#[test]
fn brace_list_nested() {
    let verse = parse_ok(vec![Left(Brace), Ident("hello".into()), Left(Brace), Text("world".into()), Newline, Right(Brace), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Raw(Ident("hello".into()), Metadata::bounds(1, 3, 1, 4)),
                        List(
                            vec![
                                verse![
                                    phrase![
                                        Raw(Text("world".into()), Metadata::bounds(1, 7, 1, 8)),
                                    ]
                                ]
                            ],
                            Metadata::bounds(1, 5, 1, 12)
                        )
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 14)),
        ]
    ], verse);
}

#[test]
fn brace_list_unterminated_err() {
    let err = parse_err(vec![Left(Brace), Ident("hello".into()), Newline]);
    assert_eq!("unterminated list", err.to_string());
}

#[test]
fn brace_list_expected_token_err() {
    let err = parse_err(vec![Left(Brace), Ident("hello".into()), Right(Paren)]);
    assert_eq!("unexpected token Right(Paren)", err.to_string());
}

#[test]
fn paren_list_empty() {
    let verse = parse_ok(vec![Left(Paren), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![], Metadata::bounds(1, 1, 1, 4)),
        ]
    ], verse);
}

#[test]
fn paren_list_nested_empty() {
    let verse = parse_ok(vec![Left(Paren), Left(Paren), Right(Paren), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![List(vec![], Metadata::bounds(1, 3, 1, 6))]
                ]
            ], Metadata::bounds(1, 1, 1, 8)),
        ]
    ], verse);
}

#[test]
fn paren_list_around_brace_list() {
    let verse = parse_ok(vec![Left(Paren), Left(Brace), Right(Brace), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![List(vec![], Metadata::bounds(1, 3, 1, 6))]
                ]
            ], Metadata::bounds(1, 1, 1, 8)),
        ]
    ], verse);
}

#[test]
fn paren_list_with_one_verse_and_phrase_with_one_node() {
    let verse = parse_ok(vec![Left(Paren), Integer(1), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Raw(Integer(1), Metadata::bounds(1, 3, 1, 4))
                    ] 
                ]
            ], Metadata::bounds(1, 1, 1, 6)),
        ]
    ], verse);
}

#[test]
fn paren_list_with_one_verse_trailing_comma() {
    let verse = parse_ok(vec![Left(Paren), Integer(1), Symbol(Ascii(b',')), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![Raw(Integer(1), Metadata::bounds(1, 3, 1, 4))]
                ]
            ], Metadata::bounds(1, 1, 1, 8)),
        ]
    ], verse);
}

#[test]
fn paren_list_with_one_verse_and_phrase_with_many_nodes() {
    let verse = parse_ok(vec![Left(Paren), Integer(1), Integer(2), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![Raw(Integer(1), Metadata::bounds(1, 3, 1, 4)), Raw(Integer(2), Metadata::bounds(1, 5, 1, 6))]
                ]
            ], Metadata::bounds(1, 1, 1, 8)),
        ]
    ], verse);
}

#[test]
fn paren_list_with_many_verses() {
    let verse = parse_ok(vec![Left(Paren), Integer(1), Integer(2), Symbol(Ascii(b',')), Integer(3), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![Raw(Integer(1), Metadata::bounds(1, 3, 1, 4)), Raw(Integer(2), Metadata::bounds(1, 5, 1, 6))],
                ],
                verse![
                    phrase![Raw(Integer(3), Metadata::bounds(1, 9, 1, 10))]
                ]
            ], Metadata::bounds(1, 1, 1, 12)),
        ]
    ], verse);
}

#[test]
fn list_empty_verse_err() {
    let err = parse_err(vec![Left(Paren), Integer(1), Symbol(Ascii(b',')), Newline, Newline, Symbol(Ascii(b',')), Right(Paren)]);
    assert_eq!("empty verse", err.to_string());
}

#[test]
fn list_unterminated_err() {
    let err = parse_err(vec![Left(Paren), Ident("hello".into()), Newline]);
    assert_eq!("unterminated list", err.to_string());
}

#[test]
fn paren_list_expected_brace_token_err() {
    let err = parse_err(vec![Left(Paren), Ident("hello".into()), Right(Brace)]);
    assert_eq!("unexpected token Right(Brace)", err.to_string());
}

#[test]
fn cons_single() {
    let verse = parse_ok(vec![Integer(1), Symbol(Ascii(b':')), Integer(2), Newline]);
    assert_eq!(verse![
        phrase![
            Cons(
                Box::new(Raw(Integer(1), Metadata::bounds(1, 1, 1, 2))), 
                phrase![Raw(Integer(2), Metadata::bounds(1, 5, 1, 6))], 
                Metadata::bounds(1, 1, 1, 6)
            ),
        ]
    ], verse);
}

#[test]
fn cons_single_long_tail() {
    let verse = parse_ok(vec![Integer(1), Symbol(Ascii(b':')), Integer(2), Integer(3), Newline]);
    assert_eq!(verse![
        phrase![
            Cons(
                Box::new(Raw(Integer(1), Metadata::bounds(1, 1, 1, 2))), 
                phrase![Raw(Integer(2), Metadata::bounds(1, 5, 1, 6)), Raw(Integer(3), Metadata::bounds(1, 7, 1, 8))], 
                Metadata::bounds(1, 1, 1, 8)
            ),
        ],
    ], verse);
}

#[test]
fn cons_multiple() {
    let verse = parse_ok(vec![Integer(1), Symbol(Ascii(b':')), Integer(2), Integer(3), Symbol(Ascii(b':')), Integer(4), Newline]);
    assert_eq!(verse![
        phrase![
            Cons(
                Box::new(
                    Cons(
                        Box::new(Raw(Integer(1), Metadata::bounds(1, 1, 1, 2))), 
                        phrase![Raw(Integer(2), Metadata::bounds(1, 5, 1, 6)), Raw(Integer(3), Metadata::bounds(1, 7, 1, 8))], 
                        Metadata::bounds(1, 1, 1, 8))
                    ), 
                phrase![Raw(Integer(4), Metadata::bounds(1, 11, 1, 12))], 
                Metadata::bounds(1, 1, 1, 12)
            ),
        ],
    ], verse);
}

#[test]
fn cons_multiple_trailing_empty_segment() {
    let verse = parse_ok(vec![Integer(1), Symbol(Ascii(b':')), Integer(2), Integer(3), Symbol(Ascii(b':')), Integer(4), Symbol(Ascii(b':')), Newline]);
    assert_eq!(verse![
        phrase![
            Cons(
                Box::new(
                    Cons(
                        Box::new(
                            Cons(
                                Box::new(Raw(Integer(1), Metadata::bounds(1, 1, 1, 2))), 
                                phrase![Raw(Integer(2), Metadata::bounds(1, 5, 1, 6)), Raw(Integer(3), Metadata::bounds(1, 7, 1, 8))], 
                                Metadata::bounds(1, 1, 1, 8)
                            )
                        ), 
                        phrase![Raw(Integer(4), Metadata::bounds(1, 11, 1, 12))], 
                        Metadata::bounds(1, 1, 1, 12)
                    )
                ), 
                phrase![], 
                Metadata::bounds(1, 1, 1, 14)
            ),
        ],
    ], verse);
}

#[test]
fn cons_with_list_tail() {
    let verse = parse_ok(vec![Integer(1), Symbol(Ascii(b':')), Left(Brace), Integer(2), Newline, Integer(3), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            Cons(
                Box::new(Raw(Integer(1), Metadata::bounds(1, 1, 1, 2))), 
                phrase![
                    List(vec![
                        verse![
                            phrase![
                                Raw(Integer(2), Metadata::bounds(1, 7, 1, 8))
                            ],
                            phrase![
                                Raw(Integer(3), Metadata::bounds(1, 11, 1, 12))
                            ]
                        ]
                    ], Metadata::bounds(1, 5, 1, 14))
                ], 
                Metadata::bounds(1, 1, 1, 14)
            ),
        ],
    ], verse);
}

#[test]
fn cons_inside_brace_list() {
    let verse = parse_ok(vec![Left(Brace), Integer(1), Symbol(Ascii(b':')), Integer(2), Integer(3), Symbol(Ascii(b':')), Integer(4), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Cons(
                            Box::new(
                                Cons(
                                    Box::new(Raw(Integer(1), Metadata::bounds(1, 3, 1, 4))), 
                                    phrase![Raw(Integer(2), Metadata::bounds(1, 7, 1, 8)), Raw(Integer(3), Metadata::bounds(1, 9, 1, 10))], 
                                    Metadata::bounds(1, 3, 1, 10)
                                )
                            ), 
                            phrase![Raw(Integer(4), Metadata::bounds(1, 13, 1, 14))], 
                            Metadata::bounds(1, 3, 1, 14)
                        ),
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 16))
        ]
    ], verse);
}

#[test]
fn cons_inside_list() {
    let verse = parse_ok(vec![Left(Paren), Integer(1), Symbol(Ascii(b':')), Integer(2), Integer(3), Symbol(Ascii(b':')), Integer(4), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Cons(
                            Box::new(
                                Cons(
                                    Box::new(Raw(Integer(1), Metadata::bounds(1, 3, 1, 4))), 
                                    phrase![Raw(Integer(2), Metadata::bounds(1, 7, 1, 8)), Raw(Integer(3), Metadata::bounds(1, 9, 1, 10))], 
                                    Metadata::bounds(1, 3, 1, 10)
                                )
                            ), 
                            phrase![Raw(Integer(4), Metadata::bounds(1, 13, 1, 14))], 
                            Metadata::bounds(1, 3, 1, 14)
                        ),
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 16))
        ]
    ], verse);
}

#[test]
fn cons_empty_starting_segment_err() {
    let err = parse_err(vec![Symbol(Ascii(b':')), Integer(2)]);
    assert_eq!("empty cons segment", err.to_string());
}

#[test]
fn cons_empty_intermediate_segment_err() {
    let err = parse_err(vec![Integer(1), Symbol(Ascii(b':')), Integer(2), Symbol(Ascii(b':')), Symbol(Ascii(b':'))]);
    assert_eq!("empty cons segment", err.to_string());
}

#[test]
fn cons_unterminated_err() {
    let err = parse_err(vec![Integer(1), Symbol(Ascii(b':')), Integer(2)]);
    assert_eq!("unterminated cons", err.to_string());
}

#[test]
fn prefix_with_integer() {
    let verse = parse_ok(vec![Symbol(Ascii(b'-')), Integer(1), Newline]);
    assert_eq!(verse![
        phrase![
            Prefix(
                Symbol(Ascii(b'-')), 
                Box::new(Raw(Integer(1), Metadata::bounds(1, 3, 1, 4))), 
                Metadata::bounds(1, 1, 1, 4)
            )
        ]
    ], verse);
}

#[test]
fn prefix_with_decimal() {
    let verse = parse_ok(vec![Symbol(Ascii(b'-')), Decimal(10, 5, 2), Newline]);
    assert_eq!(verse![
        phrase![  
            Prefix(
                Symbol(Ascii(b'-')), 
                Box::new(Raw(Decimal(10, 5, 2), Metadata::bounds(1, 3, 1, 4))), 
                Metadata::bounds(1, 1, 1, 4)
            )
        ]
    ], verse);
}

#[test]
fn prefix_with_brace_list() {
    let verse = parse_ok(vec![Symbol(Ascii(b'-')), Left(Brace), Integer(1), Newline, Integer(2), Right(Brace), Newline]);
    assert_eq!(verse![
        phrase![
            Prefix(
                Symbol(Ascii(b'-')), 
                Box::new(
                    List(vec![
                        verse![
                            phrase![
                                Raw(Integer(1), Metadata::bounds(1, 5, 1, 6))
                            ],
                            phrase![
                                Raw(Integer(2), Metadata::bounds(1, 9, 1, 10))
                            ]
                        ]
                    ], Metadata::bounds(1, 3, 1, 12))
                ), 
                Metadata::bounds(1, 1, 1, 12))
        ]
    ], verse);
}

#[test]
fn prefix_with_list() {
    let verse = parse_ok(vec![Symbol(Ascii(b'-')), Left(Paren), Integer(1), Newline, Integer(2), Symbol(Ascii(b',')), Integer(3), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            Prefix(
                Symbol(Ascii(b'-')), 
                Box::new(List(
                    vec![
                        verse![
                            phrase![Raw(Integer(1), Metadata::bounds(1, 5, 1, 6))], 
                            phrase![Raw(Integer(2), Metadata::bounds(1, 9, 1, 10))]
                        ],
                        verse![
                            phrase![Raw(Integer(3), Metadata::bounds(1, 13, 1, 14))]
                        ]
                    ], Metadata::bounds(1, 3, 1, 16))
                ), 
                Metadata::bounds(1, 1, 1, 16)
            )
        ]
    ], verse);
}

#[test]
fn prefix_inside_of_list() {
    let verse = parse_ok(vec![Left(Paren), Symbol(Ascii(b'-')), Integer(42), Right(Paren), Newline]);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Prefix(
                            Symbol(Ascii(b'-')),
                            Box::new(Raw(Integer(42), Metadata::bounds(1, 5, 1, 6))),
                            Metadata::bounds(1, 3, 1, 6)
                        )
                    ]
                ]
            ], Metadata::bounds(1, 1, 1, 8))
        ]
    ], verse);
}

#[test]
fn prefix_inside_of_cons() {
    let verse = parse_ok(vec![Text("key".into()), Symbol(Ascii(b':')), Symbol(Ascii(b'-')), Integer(42), Newline]);
    assert_eq!(verse![
        phrase![
            Cons(
                Box::new(Raw(Text("key".into()), Metadata::bounds(1, 1, 1, 2))),
                phrase![                 
                    Prefix(
                        Symbol(Ascii(b'-')),
                        Box::new(Raw(Integer(42), Metadata::bounds(1, 7, 1, 8))),
                        Metadata::bounds(1, 5, 1, 8)
                    )
                ], 
                Metadata::bounds(1, 1, 1, 8)
            )
        ]
    ], verse);
}

#[test]
fn prefix_unterminated_err() {
    let err = parse_err(vec![Symbol(Ascii(b'-'))]);
    assert_eq!("unterminated prefix", err.to_string());
}

#[test]
fn prefix_unexpected_token_err() {
    let err = parse_err(vec![Symbol(Ascii(b'-')), Symbol(Ascii(b'-'))]);
    assert_eq!("unexpected token Symbol(Ascii(b'-'))", err.to_string());
}