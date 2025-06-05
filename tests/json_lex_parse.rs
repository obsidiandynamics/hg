use std::io::BufReader;
use hg::{lexer, phrase, verse};
use hg::lexer::tokenise;
use hg::parser::parse;
use hg::token::Token;
use hg::token::Token::{Boolean, Dash, Decimal, Ident, Integer, Text};
use hg::tree::Node::{Cons, List, Prefix, Raw};
use hg::tree::{Node, Verse};

fn tok_ok(str: &str) -> Vec<Token> {
    tokenise(BufReader::with_capacity(10, str.as_bytes())).unwrap().into()
}

fn parse_ok(tokens: Vec<Token>) -> Verse {
    parse(tokens.into()).unwrap()
}

fn string(value: &str) -> Node {
    Raw(Text(value.into()))
}

fn key_value(key: &str, value: Node) -> Node {
    Cons(
        Box::new(Raw(Text(key.into()))),
        phrase![value]
    )
}

#[test]
fn small_json() {
    let str = r#"{
        "key1": "value1",
        "key2": 1234,
        "key3": 1234.5678,
        "key4": -345,
        "key5": true,
        "key6": null,
        "emptyArray": [
        ],
        "employees": [
            {
                "id": 1,
                "details": {"name": "John Wick", "age": 42, "dogOwner": true}
            },
            {
                "id": 2,
                "details": {"name": "Max Payne", "age": 39, "dogOwner": false}
            },
        ]
    }"#;
    let tokens = tok_ok(str);
    let verse = parse_ok(tokens);
    assert_eq!(verse![
        phrase![
            List(vec![
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("key1".into()))), 
                            phrase![Raw(Text("value1".into()))]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("key2".into()))), 
                            phrase![Raw(Integer(1234))]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("key3".into()))), 
                            phrase![Raw(Decimal(1234, 5678, 4))]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("key4".into()))), 
                            phrase![Prefix(Dash, Box::new(Raw(Integer(345))))]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("key5".into()))), 
                            phrase![Raw(Boolean(true))]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("key6".into()))), 
                            phrase![Raw(Ident("null".into()))]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("emptyArray".into()))), 
                            phrase![List(vec![])]
                        )
                    ]
                ],
                verse![
                    phrase![
                        Cons(
                            Box::new(Raw(Text("employees".into()))), 
                            phrase![List(vec![
                                verse![
                                    phrase![
                                        List(
                                            vec![
                                                verse![
                                                    phrase![
                                                        Cons(
                                                            Box::new(Raw(Text("id".into()))), 
                                                            phrase![Raw(Integer(1))]
                                                        )
                                                    ]
                                                ],
                                                verse![
                                                    phrase![
                                                        Cons(
                                                            Box::new(Raw(Text("details".into()))), 
                                                            phrase![
                                                                List(vec![
                                                                    verse![
                                                                        phrase![
                                                                            Cons(
                                                                                Box::new(Raw(Text("name".into()))), 
                                                                                phrase![Raw(Text("John Wick".into()))]
                                                                            )
                                                                        ]
                                                                    ],
                                                                    verse![
                                                                        phrase![
                                                                            Cons(
                                                                                Box::new(Raw(Text("age".into()))), 
                                                                                phrase![Raw(Integer(42))]
                                                                            )
                                                                        ]
                                                                    ],
                                                                    verse![
                                                                        phrase![
                                                                            Cons(
                                                                                Box::new(Raw(Text("dogOwner".into()))), 
                                                                                phrase![Raw(Boolean(true))]
                                                                            )
                                                                        ]
                                                                    ],
                                                                ])
                                                            ]
                                                        )
                                                    ]
                                                ]
                                            ]
                                        )
                                    ]
                                ],
                                verse![
                                    phrase![
                                        List(
                                            vec![
                                                verse![
                                                    phrase![
                                                        Cons(
                                                            Box::new(Raw(Text("id".into()))), 
                                                            phrase![Raw(Integer(2))]
                                                        )
                                                    ]
                                                ],
                                                verse![
                                                    phrase![
                                                        Cons(
                                                            Box::new(Raw(Text("details".into()))), 
                                                            phrase![
                                                                List(vec![
                                                                    verse![
                                                                        phrase![
                                                                            Cons(
                                                                                Box::new(Raw(Text("name".into()))), 
                                                                                phrase![Raw(Text("Max Payne".into()))]
                                                                            )
                                                                        ]
                                                                    ],
                                                                    verse![
                                                                        phrase![
                                                                            Cons(
                                                                                Box::new(Raw(Text("age".into()))), 
                                                                                phrase![Raw(Integer(39))]
                                                                            )
                                                                        ]
                                                                    ],
                                                                    verse![
                                                                        phrase![
                                                                            Cons(
                                                                                Box::new(Raw(Text("dogOwner".into()))), 
                                                                                phrase![Raw(Boolean(false))]
                                                                            )
                                                                        ]
                                                                    ],
                                                                ])
                                                            ]
                                                        )
                                                    ]
                                                ]
                                            ]
                                        )
                                    ]
                                ]
                            ])]
                        )
                    ]
                ],
            ])
        ]
    ], verse);
}