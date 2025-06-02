use std::io::BufReader;
use Token::{Integer, Newline, Text};
use crate::lexer::{tokenise, Error};
use crate::token::Token;
use crate::token::Token::{Boolean, Character, Colon, Comma, Dash, Decimal, Ident, LeftBrace, LeftParen, RightBrace, RightParen};

fn tok_ok(str: &str) -> Vec<Token> {
    tokenise(BufReader::with_capacity(10, str.as_bytes())).unwrap().into()
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
    assert_eq!("unterminated literal at line 1, column 7", err.to_string());
}

#[test]
fn text_unknown_escape_err() {
    let str = r#""hello\s
        "#;
    let err = tok_err(str);
    assert_eq!("unknown escape sequence 's' at line 1, column 8", err.to_string());
}

#[test]
fn character_unescaped() {
    let str = r#"'a'
        'b'"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Character('a'), Newline, Character('b'), Newline], tokens);
}

#[test]
fn character_escaped_newline() {
    let str = r#"'\n'"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Character('\n'), Newline], tokens);
}

#[test]
fn character_unterminated_err() {
    let str = r#"'h
        "#;
    let err = tok_err(str);
    assert_eq!("unterminated literal at line 1, column 3", err.to_string());
}

#[test]
fn character_unknown_escape_err() {
    let str = r#"'\s
        "#;
    let err = tok_err(str);
    assert_eq!("unknown escape sequence 's' at line 1, column 3", err.to_string());
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

#[test]
fn dash() {
    let str = r#" - -- -"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Dash, Dash, Dash, Dash, Newline], tokens);
}

#[test]
fn colon() {
    let str = r#" : :: :"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Colon, Colon, Colon, Colon, Newline], tokens);
}

#[test]
fn comma() {
    let str = r#" , ,, ,"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Comma, Comma, Comma, Comma, Newline], tokens);
}

#[test]
fn integer_newline_terminated() {
    let str = r#"1234567890"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Integer(1234567890), Newline], tokens);
}

#[test]
fn integer_zero_newline_terminated() {
    let str = r#"0"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Integer(0), Newline], tokens);
}

#[test]
fn integer_colon_terminated() {
    let str = r#"1_234_567_890:"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Integer(1234567890), Colon, Newline], tokens);
}

#[test]
fn integer_dash_terminated() {
    let str = r#"123-456"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Integer(123), Dash, Integer(456), Newline], tokens);
}

#[test]
fn integer_too_large_err() {
    let str = r#"1234567890123456789012345678901234567890:"#;
    let err = tok_err(str);
    assert_eq!("unparsable integer 1234567890123456789012345678901234567890 (number too large to fit in target type) at line 1, column 41", err.to_string());
}

#[test]
fn integer_invalid_err() {
    let str = r#"1k1:"#;
    let err = tok_err(str);
    assert_eq!("unparsable integer 1k1 (invalid digit found in string) at line 1, column 4", err.to_string());
}

#[test]
fn decimal_newline_terminated() {
    let str = r#"1234567890.0123456789"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Decimal(1234567890, 123456789, 10), Newline], tokens);
}

#[test]
fn decimal_small() {
    let str = r#"1234567890.0001"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Decimal(1234567890, 1, 4), Newline], tokens);
}

#[test]
fn decimal_colon_terminated() {
    let str = r#"1_234_567_890.0_123_456_789:"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Decimal(1234567890, 123456789, 10), Colon, Newline], tokens);
}

#[test]
fn decimal_whole_too_large_err() {
    let str = r#"1234567890123456789012345678901234567890.:"#;
    let err = tok_err(str);
    assert_eq!("unparsable integer 1234567890123456789012345678901234567890 (number too large to fit in target type) at line 1, column 41", err.to_string());
}

#[test]
fn decimal_fractional_too_large_err() {
    let str = r#"1234567890.1234567890123456789012345678901234567890:"#;
    let err = tok_err(str);
    assert_eq!("unparsable decimal 1234567890.1234567890123456789012345678901234567890 (number too large to fit in target type) at line 1, column 52", err.to_string());
}

#[test]
fn decimal_whole_invalid_err() {
    let str = r#"1k1."#;
    let err = tok_err(str);
    assert_eq!("unparsable integer 1k1 (invalid digit found in string) at line 1, column 4", err.to_string());
}

#[test]
fn decimal_fractional_invalid_err() {
    let str = r#"1234567890.1k1:"#;
    let err = tok_err(str);
    assert_eq!("unparsable decimal 1234567890.1k1 (invalid digit found in string) at line 1, column 15", err.to_string());
}

#[test]
fn ident() {
    let str = r#"first second
    third"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Ident("first".into()), Ident("second".into()), Newline, Ident("third".into()), Newline], tokens);
}

#[test]
fn ident_colon_terminated() {
    let str = r#"first:second"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Ident("first".into()), Colon, Ident("second".into()), Newline], tokens);
}

#[test]
fn boolean() {
    let str = r#"true false"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Boolean(true), Boolean(false), Newline], tokens);
}
