use crate::lexer::tests::Ownership::{Borrowed, Owned, NA};
use crate::lexer::{Error, Tokeniser};
use crate::metadata::Metadata;
use crate::symbols::SymbolTable;
use crate::token::ListDelimiter::{Brace, Bracket};
use crate::token::Token::{
    Boolean, Character, Decimal, ExtendedSymbol, Ident, Left, Right, Symbol,
};
use crate::token::{Ascii, AsciiSlice, ListDelimiter, Token};
use std::borrow::Cow;
use ListDelimiter::Paren;
use Token::{Integer, Newline, Text};
use crate::token;

fn tok_ok(str: &str) -> (Vec<Token>, Vec<Metadata>) {
    let tok_with_metadata = Tokeniser::new(str, SymbolTable::default())
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    let tokens = tok_with_metadata
        .iter()
        .cloned()
        .map(|(token, _)| token)
        .collect();
    let metadata = tok_with_metadata
        .into_iter()
        .map(|(_, metadata)| metadata)
        .collect();
    (tokens, metadata)
}

fn tok_err(str: &str) -> Box<Error> {
    Tokeniser::new(str, SymbolTable::default())
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
    NA,
}

fn is_owned(tokens: Vec<Token>) -> Vec<Ownership> {
    tokens
        .iter()
        .map(|token| match token {
            Text(str) | Ident(str) => {
                if matches!(str, Cow::Owned(_)) {
                    Owned
                } else {
                    Borrowed
                }
            }
            _ => NA,
        })
        .collect()
}

#[test]
fn error_terminates_tokeniser() {
    let str = r#"\n"#;
    let mut tokens = Tokeniser::new(str, SymbolTable::default());
    assert!(tokens.next().unwrap().is_err());
    assert!(tokens.next().is_none());
}

#[test]
fn text_unescaped() {
    let str = r#""hello world"
"hi""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Text("hello world".into()),
            Newline,
            Text("hi".into()),
            Newline
        ],
        tokens
    );
    assert_eq!(vec![Borrowed, NA, Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 13),
            Metadata::bounds(1, 14, 2, 0),
            Metadata::bounds(2, 1, 2, 4),
            Metadata::bounds(2, 5, 3, 0)
        ],
        metadata
    );
}

#[test]
fn text_unescaped_with_utf8() {
    let str = r#""hello µℝ💣 world"
"hiµℝ💣""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Text("hello µℝ💣 world".into()),
            Newline,
            Text("hiµℝ💣".into()),
            Newline
        ],
        tokens
    );
    assert_eq!(vec![Borrowed, NA, Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 17),
            Metadata::bounds(1, 18, 2, 0),
            Metadata::bounds(2, 1, 2, 7),
            Metadata::bounds(2, 8, 3, 0)
        ],
        metadata
    );
}

#[test]
fn text_escaped_newline() {
    let str = r#""hel\nlo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\nlo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_nul() {
    let str = r#""hel\0lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\0lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_hex() {
    let str = r#""hel\x7elo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel~lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 11),
            Metadata::bounds(1, 12, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unicode_fixed() {
    let str = r#""hel\u2764lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel❤lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 13),
            Metadata::bounds(1, 14, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unicode_fixed_ascii() {
    let str = r#""hel\u007elo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel~lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 13),
            Metadata::bounds(1, 14, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unicode_variable_24() {
    let str = r#""hel\u{1f4af}lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel💯lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 16),
            Metadata::bounds(1, 17, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unicode_variable_16() {
    let str = r#""hel\u{2764}lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel❤lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 15),
            Metadata::bounds(1, 16, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unicode_variable_ascii() {
    let str = r#""hel\u{007e}lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel~lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 15),
            Metadata::bounds(1, 16, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unicode_variable_out_of_range_err() {
    let str = r#""hel\u{ffffffff}lo""#;
    let err = tok_err(str);
    assert_eq!(
        "invalid codepoint \"ffffffff\" (codepoint out of range) at line 1, column 16",
        err.to_string()
    );
}

#[test]
fn text_escaped_hex_unparsable_err() {
    let str = r#""hel\xfglo""#;
    let err = tok_err(str);
    assert_eq!(
        "invalid codepoint \"fg\" (invalid digit found in string) at line 1, column 8",
        err.to_string()
    );
}

#[test]
fn text_escaped_newline_with_utf8() {
    let str = r#""hel\nµℝ💣""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\nµℝ💣".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 10),
            Metadata::bounds(1, 11, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_carriage_return() {
    let str = r#""hel\rlo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\rlo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_tab() {
    let str = r#""hel\tlo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\tlo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_quote() {
    let str = r#""hel\"lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\"lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_backslash() {
    let str = r#""hel\\lo""#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Text("hel\\lo".into()), Newline], tokens);
    assert_eq!(vec![Owned, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}

#[test]
fn text_escaped_unknown_utf8_err() {
    let str = r#""hel\µ""#;
    let err = tok_err(str);
    assert_eq!(
        "unknown escape sequence \"µ\" at line 1, column 6",
        err.to_string()
    );
}

#[test]
fn text_escaped_unterminated_unicode_err() {
    let str = r#""hel\u"#;
    let err = tok_err(str);
    assert_eq!(
        "unknown escape sequence \"\n\" at line 1, column 7",
        err.to_string()
    );
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
    assert_eq!(
        "unknown escape sequence \"s\" at line 1, column 8",
        err.to_string()
    );
}

#[test]
fn character_unescaped() {
    let str = r#"  'a'
'b'"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Character('a'), Newline, Character('b'), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 3, 1, 5),
            Metadata::bounds(1, 6, 2, 0),
            Metadata::bounds(2, 1, 2, 3),
            Metadata::bounds(2, 4, 3, 0),
        ],
        metadata
    );
}

#[test]
fn character_unescaped_with_unicode() {
    let str = r#"'💣'
'µ'"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Character('💣'), Newline, Character('µ'), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 3),
            Metadata::bounds(1, 4, 2, 0),
            Metadata::bounds(2, 1, 2, 3),
            Metadata::bounds(2, 4, 3, 0),
        ],
        metadata
    );
}

#[test]
fn character_escaped_newline() {
    let str = r#"'\n'"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Character('\n'), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 4),
            Metadata::bounds(1, 5, 2, 0),
        ],
        metadata
    );
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
    assert_eq!(
        "unexpected character 'j' at line 1, column 3",
        err.to_string()
    );
}

#[test]
fn character_empty_err() {
    let str = r#"''"#;
    let err = tok_err(str);
    assert_eq!(
        "empty character literal at line 1, column 2",
        err.to_string()
    );
}

#[test]
fn character_unknown_escape_err() {
    let str = r#"'\s
        "#;
    let err = tok_err(str);
    assert_eq!(
        "unknown escape sequence \"s\" at line 1, column 3",
        err.to_string()
    );
}

#[test]
fn escape_during_whitespace_err() {
    let str = r#"\n
        "#;
    let err = tok_err(str);
    assert_eq!(
        "unexpected character '\\' at line 1, column 1",
        err.to_string()
    );
}

#[test]
fn left_and_right_paren() {
    let str = r#"(( ))"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Paren),
            Left(Paren),
            Right(Paren),
            Right(Paren),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 6, 2, 0),
        ],
        metadata
    );
}

#[test]
fn left_and_right_paren_around_text() {
    let str = r#"("a string"
"another string")"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Paren),
            Text("a string".into()),
            Newline,
            Text("another string".into()),
            Right(Paren),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 11),
            Metadata::bounds(1, 12, 2, 0),
            Metadata::bounds(2, 1, 2, 16),
            Metadata::bounds(2, 17, 2, 17),
            Metadata::bounds(2, 18, 3, 0),
        ],
        metadata
    );
}

#[test]
fn left_and_right_brace() {
    let str = r#"{{ }}"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Brace),
            Left(Brace),
            Right(Brace),
            Right(Brace),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 6, 2, 0),
        ],
        metadata
    );
}

#[test]
fn dash() {
    let str = r#" - -- -"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Symbol(Ascii(b'-')),
            ExtendedSymbol(AsciiSlice(&[b'-', b'-'])),
            Symbol(Ascii(b'-')),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 4, 1, 5),
            Metadata::bounds(1, 7, 1, 7),
            Metadata::bounds(1, 8, 2, 0),
        ],
        metadata
    );
}

#[test]
fn colon() {
    let str = r#" : :: :"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Symbol(Ascii(b':')),
            ExtendedSymbol(AsciiSlice(&[b':', b':'])),
            Symbol(Ascii(b':')),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 4, 1, 5),
            Metadata::bounds(1, 7, 1, 7),
            Metadata::bounds(1, 8, 2, 0),
        ],
        metadata
    );
}

#[test]
fn comma() {
    let str = r#" , ,, ,"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Symbol(Ascii(b',')),
            Symbol(Ascii(b',')),
            Symbol(Ascii(b',')),
            Symbol(Ascii(b',')),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 7, 1, 7),
            Metadata::bounds(1, 8, 2, 0),
        ],
        metadata
    );
}

#[test]
fn integer_newline_terminated() {
    let str = r#"1234567890"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Integer(1234567890), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 10),
            Metadata::bounds(1, 11, 2, 0),
        ],
        metadata
    );
}

#[test]
fn integer_zero_newline_terminated() {
    let str = r#"0"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Integer(0), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 2, 0),
        ],
        metadata
    );
}

#[test]
fn integer_colon_terminated() {
    let str = r#"1_234_567_890:"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Integer(1234567890), Symbol(Ascii(b':')), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 13),
            Metadata::bounds(1, 14, 1, 14),
            Metadata::bounds(1, 15, 2, 0),
        ],
        metadata
    );
}

#[test]
fn integer_dash_terminated() {
    let str = r#"123-456"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Integer(123), Symbol(Ascii(b'-')), Integer(456), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 3),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 7),
            Metadata::bounds(1, 8, 2, 0),
        ],
        metadata
    );
}

#[test]
fn integer_comma_terminated() {
    let str = r#"123,456"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Integer(123), Symbol(Ascii(b',')), Integer(456), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 3),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 7),
            Metadata::bounds(1, 8, 2, 0),
        ],
        metadata
    );
}

#[test]
fn integer_too_large_err() {
    let str = r#"1234567890123456789012345678901234567890:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable integer 1234567890123456789012345678901234567890 (number too large to fit in target type) at line 1, column 41",
        err.to_string()
    );
}

#[test]
fn integer_invalid_err() {
    let str = r#"1k1:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable integer 1k1 (invalid digit found in string) at line 1, column 4",
        err.to_string()
    );
}

#[test]
fn integer_invalid_due_to_utf8_err() {
    let str = r#"1💣1:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable integer 1💣1 (invalid digit found in string) at line 1, column 4",
        err.to_string()
    );
}

#[test]
fn decimal_newline_terminated() {
    let str = r#"1234567890.0123456789"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Decimal(token::Decimal(1234567890, 123456789, 10)), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 21),
            Metadata::bounds(1, 22, 2, 0),
        ],
        metadata
    );
}

#[test]
fn decimal_small() {
    let str = r#"1234567890.0001"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Decimal(token::Decimal(1234567890, 1, 4)), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 15),
            Metadata::bounds(1, 16, 2, 0),
        ],
        metadata
    );
}

#[test]
fn decimal_implied_leading_zero() {
    let str = r#".123"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Decimal(token::Decimal(0, 123, 3)), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 4),
            Metadata::bounds(1, 5, 2, 0),
        ],
        metadata
    );
}

#[test]
fn symbol_and_decimal() {
    let str = r#". .123"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Symbol(Ascii(b'.')), Decimal(token::Decimal(0, 123, 3)), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 3, 1, 6),
            Metadata::bounds(1, 7, 2, 0),
        ],
        metadata
    );
}

#[test]
fn decimal_colon_terminated() {
    let str = r#"1_234_567_890.0_123_456_789:"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Decimal(token::Decimal(1234567890, 123456789, 10)),
            Symbol(Ascii(b':')),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 27),
            Metadata::bounds(1, 28, 1, 28),
            Metadata::bounds(1, 29, 2, 0),
        ],
        metadata
    );
}

#[test]
fn decimal_comma_terminated() {
    let str = r#"1_234_567_890.0_123_456_789,12.34"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Decimal(token::Decimal(1234567890, 123456789, 10)),
            Symbol(Ascii(b',')),
            Decimal(token::Decimal(12, 34, 2)),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 27),
            Metadata::bounds(1, 28, 1, 28),
            Metadata::bounds(1, 29, 1, 33),
            Metadata::bounds(1, 34, 2, 0),
        ],
        metadata
    );
}

#[test]
fn decimal_whole_too_large_err() {
    let str = r#"1234567890123456789012345678901234567890.:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable integer 1234567890123456789012345678901234567890 (number too large to fit in target type) at line 1, column 41",
        err.to_string()
    );
}

#[test]
fn decimal_fractional_too_large_err() {
    let str = r#"1234567890.1234567890123456789012345678901234567890:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable decimal 1234567890.1234567890123456789012345678901234567890 (number too large to fit in target type) at line 1, column 52",
        err.to_string()
    );
}

#[test]
fn decimal_whole_invalid_due_to_utf8_err() {
    let str = r#"1💣1."#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable integer 1💣1 (invalid digit found in string) at line 1, column 4",
        err.to_string()
    );
}

#[test]
fn decimal_fractional_invalid_err() {
    let str = r#"1234567890.1k1:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable decimal 1234567890.1k1 (invalid digit found in string) at line 1, column 15",
        err.to_string()
    );
}

#[test]
fn decimal_fractional_invalid_due_to_utf8_err() {
    let str = r#"1234567890.1💣1:"#;
    let err = tok_err(str);
    assert_eq!(
        "unparsable decimal 1234567890.1💣1 (invalid digit found in string) at line 1, column 15",
        err.to_string()
    );
}

#[test]
fn ident() {
    let str = r#"first second
third"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Ident("first".into()),
            Ident("second".into()),
            Newline,
            Ident("third".into()),
            Newline
        ],
        tokens
    );
    assert_eq!(vec![Borrowed, Borrowed, NA, Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 5),
            Metadata::bounds(1, 7, 1, 12),
            Metadata::bounds(1, 13, 2, 0),
            Metadata::bounds(2, 1, 2, 5),
            Metadata::bounds(2, 6, 3, 0),
        ],
        metadata
    );
}

#[test]
fn ident_with_mid_and_trailing_digits() {
    let str = r#"alpha123tail456"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Ident("alpha123tail456".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 15),
            Metadata::bounds(1, 16, 2, 0),
        ],
        metadata
    );
}

#[test]
fn ident_with_underscores() {
    let str = r#"__alpha_bravo"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Ident("__alpha_bravo".into()), Newline], tokens);
    assert_eq!(vec![Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 13),
            Metadata::bounds(1, 14, 2, 0),
        ],
        metadata
    );
}

#[test]
fn ident_starts_with_unicode() {
    let str = r#"first µℝ💣second
third"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Ident("first".into()),
            Ident("µℝ💣second".into()),
            Newline,
            Ident("third".into()),
            Newline
        ],
        tokens
    );
    assert_eq!(vec![Borrowed, Borrowed, NA, Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 5),
            Metadata::bounds(1, 7, 1, 15),
            Metadata::bounds(1, 16, 2, 0),
            Metadata::bounds(2, 1, 2, 5),
            Metadata::bounds(2, 6, 3, 0),
        ],
        metadata
    );
}

#[test]
fn ident_ends_with_unicode() {
    let str = r#"first second_µℝ💣
third"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Ident("first".into()),
            Ident("second_µℝ💣".into()),
            Newline,
            Ident("third".into()),
            Newline
        ],
        tokens
    );
    assert_eq!(vec![Borrowed, Borrowed, NA, Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 5),
            Metadata::bounds(1, 7, 1, 16),
            Metadata::bounds(1, 17, 2, 0),
            Metadata::bounds(2, 1, 2, 5),
            Metadata::bounds(2, 6, 3, 0),
        ],
        metadata
    );
}

#[test]
fn ident_colon_terminated() {
    let str = r#"first:second"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Ident("first".into()),
            Symbol(Ascii(b':')),
            Ident("second".into()),
            Newline
        ],
        tokens
    );
    assert_eq!(vec![Borrowed, NA, Borrowed, NA], is_owned(tokens));
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 5),
            Metadata::bounds(1, 6, 1, 6),
            Metadata::bounds(1, 7, 1, 12),
            Metadata::bounds(1, 13, 2, 0),
        ],
        metadata
    );
}

#[test]
fn boolean() {
    let str = r#"true false"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(vec![Boolean(true), Boolean(false), Newline], tokens);
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 4),
            Metadata::bounds(1, 6, 1, 10),
            Metadata::bounds(1, 11, 2, 0),
        ],
        metadata
    );
}

#[test]
fn boolean_comma_terminated() {
    let str = r#"true false,"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![Boolean(true), Boolean(false), Symbol(Ascii(b',')), Newline],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 4),
            Metadata::bounds(1, 6, 1, 10),
            Metadata::bounds(1, 11, 1, 11),
            Metadata::bounds(1, 12, 2, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_flat_sequence_of_tokens() {
    let str = r#"hello "world"
42"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Ident("hello".into()),
            Text("world".into()),
            Newline,
            Integer(42),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 5),
            Metadata::bounds(1, 7, 1, 13),
            Metadata::bounds(1, 14, 2, 0),
            Metadata::bounds(2, 1, 2, 2),
            Metadata::bounds(2, 3, 3, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_list_around_list() {
    let str = r#"{([])}"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Brace),
            Left(Paren),
            Left(Bracket),
            Right(Bracket),
            Right(Paren),
            Right(Brace),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 3, 1, 3),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 6, 1, 6),
            Metadata::bounds(1, 7, 2, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_list_nested() {
    let str = r#"{hello {"world"
}}"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Brace),
            Ident("hello".into()),
            Left(Brace),
            Text("world".into()),
            Newline,
            Right(Brace),
            Right(Brace),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 6),
            Metadata::bounds(1, 8, 1, 8),
            Metadata::bounds(1, 9, 1, 15),
            Metadata::bounds(1, 16, 2, 0),
            Metadata::bounds(2, 1, 2, 1),
            Metadata::bounds(2, 2, 2, 2),
            Metadata::bounds(2, 3, 3, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_list_with_one_item_trailing_comma() {
    let str = r#"(1,)"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Paren),
            Integer(1),
            Symbol(Ascii(b',')),
            Right(Paren),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 3, 1, 3),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 2, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_list_with_many_items() {
    let str = r#"(1 2, 3)"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Paren),
            Integer(1),
            Integer(2),
            Symbol(Ascii(b',')),
            Integer(3),
            Right(Paren),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 7, 1, 7),
            Metadata::bounds(1, 8, 1, 8),
            Metadata::bounds(1, 9, 2, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_cons_single_long_tail() {
    let str = r#"1:2 3"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Integer(1),
            Symbol(Ascii(b':')),
            Integer(2),
            Integer(3),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 3, 1, 3),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 6, 2, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_cons_multiple() {
    let str = r#"1:2 3:4"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Integer(1),
            Symbol(Ascii(b':')),
            Integer(2),
            Integer(3),
            Symbol(Ascii(b':')),
            Integer(4),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 3, 1, 3),
            Metadata::bounds(1, 5, 1, 5),
            Metadata::bounds(1, 6, 1, 6),
            Metadata::bounds(1, 7, 1, 7),
            Metadata::bounds(1, 8, 2, 0),
        ],
        metadata
    );
}

#[test]
fn mixed_cons_inside_list() {
    let str = r#"{1:2 3:4}"#;
    let (tokens, metadata) = tok_ok(str);
    assert_eq!(
        vec![
            Left(Brace),
            Integer(1),
            Symbol(Ascii(b':')),
            Integer(2),
            Integer(3),
            Symbol(Ascii(b':')),
            Integer(4),
            Right(Brace),
            Newline
        ],
        tokens
    );
    assert_eq!(
        vec![
            Metadata::bounds(1, 1, 1, 1),
            Metadata::bounds(1, 2, 1, 2),
            Metadata::bounds(1, 3, 1, 3),
            Metadata::bounds(1, 4, 1, 4),
            Metadata::bounds(1, 6, 1, 6),
            Metadata::bounds(1, 7, 1, 7),
            Metadata::bounds(1, 8, 1, 8),
            Metadata::bounds(1, 9, 1, 9),
            Metadata::bounds(1, 10, 2, 0),
        ],
        metadata
    );
}
