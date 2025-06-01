use std::io::BufReader;
use Token::{Newline, Text};
use crate::lexer::{tokenise, Error};
use crate::token::Token;
use crate::token::Token::{LeftBrace, LeftParen, RightBrace, RightParen};

fn tok_ok(str: &str) -> Vec<Token> {
    tokenise(BufReader::with_capacity(10, str.as_bytes())).unwrap()
}

fn tok_err(str: &str) -> Error {
    tokenise(BufReader::with_capacity(10, str.as_bytes())).unwrap_err()
}

#[test]
fn text_unescaped() {
    let str = r#""hello world"
        "hi""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hello world".into()), Newline, Text("hi".into()), Newline], tokens);
}

#[test]
fn text_escaped_newline() {
    let str = r#""hel\nlo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\nlo".into()), Newline], tokens);
}

#[test]
fn text_escaped_carriage_return() {
    let str = r#""hel\rlo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\rlo".into()), Newline], tokens);
}

#[test]
fn text_escaped_tab() {
    let str = r#""hel\tlo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\tlo".into()), Newline], tokens);
}

#[test]
fn text_escaped_quote() {
    let str = r#""hel\"lo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\"lo".into()), Newline], tokens);
}

#[test]
fn text_escaped_backslash() {
    let str = r#""hel\\lo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\\lo".into()), Newline], tokens);
}

#[test]
fn text_unterminated_err() {
    let str = r#""hello
        "#;
    let err = tok_err(str);
    assert_eq!("unterminated text literal at line 1, column 7", err.to_string());
}

#[test]
fn text_unknown_escape_err() {
    let str = r#""hello\s
        "#;
    let err = tok_err(str);
    assert_eq!("unknown escape sequence 's' at line 1, column 8", err.to_string());
}

#[test]
fn escape_during_whitespace_err() {
    let str = r#"\n
        "#;
    let err = tok_err(str);
    assert_eq!("unexpected character '\\' at line 1, column 1", err.to_string());
}

#[test]
fn left_and_right_paren() {
    let str = r#"(( ))"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![LeftParen, LeftParen, RightParen, RightParen, Newline], tokens);
}

#[test]
fn left_and_right_paren_around_text() {
    let str = r#"("a string"
    "another string")"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![LeftParen, Text("a string".into()), Newline, Text("another string".into()), RightParen, Newline], tokens);
}

#[test]
fn left_and_right_brace() {
    let str = r#"{{ }}"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![LeftBrace, LeftBrace, RightBrace, RightBrace, Newline], tokens);
}