use hg::lexer::Tokeniser;
use hg::parser::parse;
use hg::token::Token;
use hg::token::Token::{Boolean, Dash, Decimal, Ident, Integer, Text};
use hg::tree::Node::{Cons, List, Prefix, Raw};
use hg::tree::{Node, Verse};
use hg::{phrase, verse};

fn tok_ok(str: &str) -> Vec<Token> {
    Tokeniser::new(str).map(Result::unwrap).collect()
}

fn parse_ok(tokens: Vec<Token>) -> Verse {
    parse(tokens.into_iter().map(Ok)).unwrap()
}

fn string(value: &str) -> Node {
    Raw(Text(value.into()))
}

fn integer(value: u128) -> Node<'static> {
    Raw(Integer(value))
}

fn decimal(whole: u128, fractional: u128, scale: u8) -> Node<'static> {
    Raw(Decimal(whole, fractional, scale))
}

fn boolean(value: bool) -> Node<'static> {
    Raw(Boolean(value))
}

fn null() -> Node<'static> {
    Raw(Ident("null".into()))
}

fn negative(value: impl Into<Node<'static>>) -> Node<'static> {
    Prefix(Dash, Box::new(value.into()))
}

fn key_value(key: &'static str, value: Node<'static>) -> Node<'static> {
    Cons(
        Box::new(Raw(Text(key.into()))),
        phrase![value]
    )
}

struct ArrayBuilder(Vec<Node<'static>>);

impl ArrayBuilder {
    fn with(mut self, element: impl Into<Node<'static>>) -> Self {
        self.0.push(element.into());
        self
    }
}

impl From<ArrayBuilder> for Node<'static> {
    fn from(array_builder: ArrayBuilder) -> Self {
        let verses = array_builder.0.into_iter().map(|element| verse![phrase![element]]).collect();
        List(verses)
    }
}

fn array() -> ArrayBuilder {
    ArrayBuilder(vec![])
}

struct ObjectBuilder(Vec<(&'static str, Node<'static>)>);

impl ObjectBuilder {
    fn key(self, key: &'static str) -> ObjectKeyValueBuilder {
        ObjectKeyValueBuilder(self, key)
    }
}

impl From<ObjectBuilder> for Node<'static> {
    fn from(object_builder: ObjectBuilder) -> Self {
        let verses = object_builder.0.into_iter().map(|(key, value)| verse![phrase![key_value(key, value)]]).collect();
        List(verses)
    }
}

fn object() -> ObjectBuilder {
    ObjectBuilder(vec![])
}

struct ObjectKeyValueBuilder(ObjectBuilder, &'static str);

impl ObjectKeyValueBuilder {
    fn value(self, value: impl Into<Node<'static>>) -> ObjectBuilder {
        let (mut object_builder, key) = (self.0, self.1);
        object_builder.0.push((key, value.into()));
        object_builder
    }
}

fn root(node: impl Into<Node<'static>>) -> Verse<'static> {
    verse![phrase![node.into()]]
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