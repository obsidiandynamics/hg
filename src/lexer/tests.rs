use std::borrow::Cow;
use ListDelimiter::Paren;
use Token::{Integer, Newline, Text};
use crate::lexer::{Error, Tokeniser};
use crate::lexer::tests::Ownership::{Borrowed, NA, Owned};
use crate::token::{ListDelimiter, Token};
use crate::token::ListDelimiter::Brace;
use crate::token::Token::{Boolean, Character, Colon, Comma, Dash, Decimal, Ident, Left, Right};

fn tok_ok(str: &str) -> Vec<Token> {
    Tokeniser::new(str).map(Result::unwrap).collect()
}

fn tok_err(str: &str) -> Box<Error> {
     Tokeniser::new(str)
        .map(Result::err)
        .skip_while(Option::is_none)
        .map(Option::unwrap)
        .next()
        .unwrap()
}

#[derive(Debug, PartialEq)]
enum Ownership {
    Owned,
    Borrowed,
    NA
}

fn is_owned(tokens: Vec<Token>) -> Vec<Ownership> {
    tokens.iter().map(|token| {
        match token {
            Text(str) | Ident(str) => {
                if matches!(str, Cow::Owned(_)) { Owned } else { Borrowed }
            }
            _ => NA,
        }
    }).collect()
}

#[test]
fn error_terminates_tokeniser() {
    let str = r#"\n"#;
    let mut tokens = Tokeniser::new(str);
    assert!(tokens.next().unwrap().is_err());
    assert!(tokens.next().is_none());
}

#[test]
fn text_unescaped() {
    let str = r#""hello world"
        "hi""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hello world".into()), Newline, Text("hi".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, NA, Borrowed, NA], is_owned(tokens));
}

#[test]
fn text_unescaped_with_utf8() {
    let str = r#""hello µℝ💣 world"
        "hiµℝ💣""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hello µℝ💣 world".into()), Newline, Text("hiµℝ💣".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, NA, Borrowed, NA], is_owned(tokens));
}

#[test]
fn text_escaped_newline() {
    let str = r#""hel\nlo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\nlo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
}

#[test]
fn text_escaped_newline_with_utf8() {
    let str = r#""hel\nµℝ💣""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\nµℝ💣".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
}

#[test]
fn text_escaped_carriage_return() {
    let str = r#""hel\rlo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\rlo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
}

#[test]
fn text_escaped_tab() {
    let str = r#""hel\tlo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\tlo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
}

#[test]
fn text_escaped_quote() {
    let str = r#""hel\"lo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\"lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
}

#[test]
fn text_escaped_backslash() {
    let str = r#""hel\\lo""#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Text("hel\\lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
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
fn character_unescaped_with_utf8() {
    let str = r#"'💣'
        'µ'"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Character('💣'), Newline, Character('µ'), Newline], tokens);
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
fn character_too_long_err() {
    let str = r#"'hj'"#;
    let err = tok_err(str);
    assert_eq!("unexpected character 'j' at line 1, column 3", err.to_string());
}

#[test]
fn character_empty_err() {
    let str = r#"''"#;
    let err = tok_err(str);
    assert_eq!("empty character literal at line 1, column 2", err.to_string());
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
    assert_eq!(vec![Left(Paren), Left(Paren), Right(Paren), Right(Paren), Newline], tokens);
}

#[test]
fn left_and_right_paren_around_text() {
    let str = r#"("a string"
    "another string")"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Left(Paren), Text("a string".into()), Newline, Text("another string".into()), Right(Paren), Newline], tokens);
}

#[test]
fn left_and_right_brace() {
    let str = r#"{{ }}"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Left(Brace), Left(Brace), Right(Brace), Right(Brace), Newline], tokens);
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
fn integer_comma_terminated() {
    let str = r#"123,456"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Integer(123), Comma, Integer(456), Newline], tokens);
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
fn integer_invalid_due_to_utf8_err() {
    let str = r#"1💣1:"#;
    let err = tok_err(str);
    assert_eq!("unparsable integer 1💣1 (invalid digit found in string) at line 1, column 4", err.to_string());
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
fn decimal_implied_leading_zero() {
    let str = r#".123"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Decimal(0, 123, 3), Newline], tokens);
}

#[test]
fn decimal_colon_terminated() {
    let str = r#"1_234_567_890.0_123_456_789:"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Decimal(1234567890, 123456789, 10), Colon, Newline], tokens);
}

#[test]
fn decimal_comma_terminated() {
    let str = r#"1_234_567_890.0_123_456_789,12.34"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Decimal(1234567890, 123456789, 10), Comma, Decimal(12, 34, 2), Newline], tokens);
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
fn decimal_whole_invalid_due_to_utf8_err() {
    let str = r#"1💣1."#;
    let err = tok_err(str);
    assert_eq!("unparsable integer 1💣1 (invalid digit found in string) at line 1, column 4", err.to_string());
}

#[test]
fn decimal_fractional_invalid_err() {
    let str = r#"1234567890.1k1:"#;
    let err = tok_err(str);
    assert_eq!("unparsable decimal 1234567890.1k1 (invalid digit found in string) at line 1, column 15", err.to_string());
}

#[test]
fn decimal_fractional_invalid_due_to_utf8_err() {
    let str = r#"1234567890.1💣1:"#;
    let err = tok_err(str);
    assert_eq!("unparsable decimal 1234567890.1💣1 (invalid digit found in string) at line 1, column 15", err.to_string());
}

#[test]
fn ident() {
    let str = r#"first second
    third"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Ident("first".into()), Ident("second".into()), Newline, Ident("third".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, Borrowed, NA, Borrowed, NA], is_owned(tokens));
}

#[test]
fn ident_starts_with_utf8() {
    let str = r#"first µℝ💣second
    third"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Ident("first".into()), Ident("µℝ💣second".into()), Newline, Ident("third".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, Borrowed, NA, Borrowed, NA], is_owned(tokens));
}

#[test]
fn ident_ends_with_utf8() {
    let str = r#"first secondµℝ💣
    third"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Ident("first".into()), Ident("secondµℝ💣".into()), Newline, Ident("third".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, Borrowed, NA, Borrowed, NA], is_owned(tokens));
}

#[test]
fn ident_colon_terminated() {
    let str = r#"first:second"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Ident("first".into()), Colon, Ident("second".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, NA, Borrowed, NA], is_owned(tokens));
}

#[test]
fn boolean() {
    let str = r#"true false"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Boolean(true), Boolean(false), Newline], tokens);
}

#[test]
fn boolean_comma_terminated() {
    let str = r#"true false,"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![Boolean(true), Boolean(false), Comma, Newline], tokens);
}

#[test]
fn mixed_flat_sequence_of_tokens() {
    let str = r#"hello "world"
    42"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Ident("hello".into()),
        Text("world".into()),
        Newline,
        Integer(42),
        Newline
    ], tokens);
}

#[test]
fn mixed_container_around_list() {
    let str = r#"{()}"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Left(Brace), Left(Paren), Right(Paren), Right(Brace), Newline
    ], tokens);
}

#[test]
fn mixed_container_nested() {
    let str = r#"{hello {"world"
    }}"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Left(Brace), Ident("hello".into()), Left(Brace), Text("world".into()), Newline, Right(Brace), Right(Brace), Newline
    ], tokens);
}

#[test]
fn mixed_list_with_one_item_trailing_comma() {
    let str = r#"(1,)"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Left(Paren), Integer(1), Comma, Right(Paren), Newline
    ], tokens);
}

#[test]
fn mixed_list_with_many_items() {
    let str = r#"(1 2, 3)"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Left(Paren), Integer(1), Integer(2), Comma, Integer(3), Right(Paren), Newline
    ], tokens);
}

#[test]
fn mixed_cons_single_long_tail() {
    let str = r#"1:2 3"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Integer(1), Colon, Integer(2), Integer(3), Newline
    ], tokens);
}

#[test]
fn mixed_cons_multiple() {
    let str = r#"1:2 3:4"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), Newline
    ], tokens);
}

#[test]
fn mixed_cons_inside_container() {
    let str = r#"{1:2 3:4}"#;
    let tokens = tok_ok(str);
    assert_eq!(vec![
        Left(Brace), Integer(1), Colon, Integer(2), Integer(3), Colon, Integer(4), Right(Brace), Newline
    ], tokens);
}
