use hg::lexer::Tokeniser;
use hg::metadata::Metadata;
use hg::parser::parse;
use hg::symbols::SymbolTable;
use hg::token::Token::{Boolean, Decimal, Ident, Integer, Symbol, Text};
use hg::token::{Ascii, Token};
use hg::tree::Node::{Cons, List, Raw};
use hg::tree::{Node, Phrase, Verse};
use hg::{lexer, phrase, token, verse};
use std::iter::Map;
use std::vec::IntoIter;

fn tok_ok(str: &str) -> Vec<Token> {
    Tokeniser::new(str, SymbolTable::default()).map(Result::unwrap).map(|(token, _)| token).collect()
}

fn without_metadata(tokens: Vec<Token>) -> Map<IntoIter<Token>, fn(Token) -> Result<(Token, Metadata), Box<lexer::Error>>> {
    tokens.into_iter().map(|token| Ok((token, Metadata::unspecified())))
}

fn parse_ok(tokens: Vec<Token>) -> Verse {
    parse(without_metadata(tokens)).unwrap()
}

fn string(value: &str) -> Vec<Node> {
    vec![Raw(Text(value.into()), Metadata::unspecified())]
}

fn integer(value: u128) -> Vec<Node<'static>> {
    vec![Raw(Integer(value), Metadata::unspecified())]
}

fn decimal(whole: u128, fractional: u128, scale: u8) -> Vec<Node<'static>> {
    vec![Raw(Decimal(token::Decimal(whole, fractional, scale)), Metadata::unspecified())]
}

fn boolean(value: bool) -> Vec<Node<'static>> {
    vec![Raw(Boolean(value), Metadata::unspecified())]
}

fn null() -> Vec<Node<'static>> {
    vec![Raw(Ident("null".into()), Metadata::unspecified())]
}

fn negative(value: impl Into<Vec<Node<'static>>>) -> Vec<Node<'static>> {
    let value = value.into();
    let mut concat = Vec::with_capacity(value.len() + 1);
    concat.push(Raw(Symbol(Ascii(b'-')), Metadata::unspecified()));
    for node in value {
        concat.push(node);
    }
    concat
}

fn key_value(key: &'static str, value: Vec<Node<'static>>) -> Vec<Node<'static>> {
    vec![Cons(
        Box::new(Raw(Text(key.into()), Metadata::unspecified())),
        Phrase(value), 
        Metadata::unspecified()
    )]
}

struct ArrayBuilder(Vec<Vec<Node<'static>>>);

impl ArrayBuilder {
    fn with(mut self, element: impl Into<Vec<Node<'static>>>) -> Self {
        self.0.push(element.into());
        self
    }
}

impl From<ArrayBuilder> for Vec<Node<'static>> {
    fn from(array_builder: ArrayBuilder) -> Self {
        let verses = array_builder.0.into_iter().map(|element| verse![Phrase(element)]).collect();
        vec![List(verses, Metadata::unspecified())]
    }
}

fn array() -> ArrayBuilder {
    ArrayBuilder(vec![])
}

struct ObjectBuilder(Vec<(&'static str, Vec<Node<'static>>)>);

impl ObjectBuilder {
    fn key(self, key: &'static str) -> ObjectKeyValueBuilder {
        ObjectKeyValueBuilder(self, key)
    }
}

impl From<ObjectBuilder> for Vec<Node<'static>> {
    fn from(object_builder: ObjectBuilder) -> Self {
        let verses = object_builder.0.into_iter().map(|(key, value)| verse![Phrase(key_value(key, value))]).collect();
        vec![List(verses, Metadata::unspecified())]
    }
}

fn object() -> ObjectBuilder {
    ObjectBuilder(vec![])
}

struct ObjectKeyValueBuilder(ObjectBuilder, &'static str);

impl ObjectKeyValueBuilder {
    fn value(self, value: impl Into<Vec<Node<'static>>>) -> ObjectBuilder {
        let (mut object_builder, key) = (self.0, self.1);
        object_builder.0.push((key, value.into()));
        object_builder
    }
}

fn root(node: impl Into<Vec<Node<'static>>>) -> Verse<'static> {
    verse![Phrase(node.into())]
}

#[test]
fn multilevel_json() {
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
            }
        ]
    }"#;
    let tokens = tok_ok(str);
    let verse = parse_ok(tokens);
    assert_eq!(root(
        object()
            .key("key1").value(string("value1"))
            .key("key2").value(integer(1234))
            .key("key3").value(decimal(1234, 5678, 4))
            .key("key4").value(negative(integer(345)))
            .key("key5").value(boolean(true))
            .key("key6").value(null())
            .key("emptyArray").value(array())
            .key("employees").value(array()
                .with(object()
                    .key("id").value(integer(1))
                    .key("details").value(object()
                        .key("name").value(string("John Wick"))
                        .key("age").value(integer(42))
                        .key("dogOwner").value(boolean(true))
                    )
                )
                .with(object()
                    .key("id").value(integer(2))
                    .key("details").value(object()
                        .key("name").value(string("Max Payne"))
                        .key("age").value(integer(39))
                        .key("dogOwner").value(boolean(false))
                    )
                )
            )
    ), verse);
}