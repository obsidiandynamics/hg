use crate::token::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Raw(Token),
    List(Vec<Node>),
    Container(Vec<Node>),
    Pair(Token, Vec<Node>)
}